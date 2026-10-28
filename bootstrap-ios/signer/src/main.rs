use clap::Parser;
use isideload::{
    anisette::remote_v3::RemoteV3AnisetteProvider,
    auth::apple_account::{AppleAccount, TwoFactorCallbackParams, TwoFactorCallbackResponse},
    dev::{
        certificates::DevelopmentCertificate,
        developer_session::DeveloperSession,
        devices::DevicesApi,
        teams::{DeveloperTeam, TeamsApi},
    },
    sideload::{
        SideloaderBuilder, TeamSelection,
        builder::{ExtensionsBehavior, ExtensionsBehaviorSelection, MaxCertsBehavior},
    },
    util::{
        callbacks::{ExtensionsBehaviorCallbackBox, MaxCertsCallbackBox},
        fs_storage::FsStorage,
    },
};
use serde_json::json;
use std::{
    fs,
    io::{self, Write},
    os::unix::fs::{PermissionsExt, symlink},
    path::{Path, PathBuf},
    process::Command,
};
use tempfile::TempDir;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use zeroize::Zeroizing;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Apple Account email. The password is always requested interactively.
    #[arg(long)]
    apple_id: String,

    /// UDID captured locally from the iPhone.
    #[arg(long)]
    udid: String,

    /// Unsigned IPA to sign.
    #[arg(long)]
    ipa: PathBuf,

    /// Destination for the signed IPA.
    #[arg(long)]
    output: PathBuf,

    /// Friendly device name registered with Apple.
    #[arg(long, default_value = "iPhone iSH bootstrap")]
    device_name: String,

    /// Developer team to use when the account belongs to more than one team.
    #[arg(long)]
    team_id: Option<String>,

    /// Private directory used for the certificate and Anisette state.
    #[arg(long, default_value = ".ios-bootstrap-state")]
    state_dir: PathBuf,
}

fn read_line(prompt: &str) -> io::Result<String> {
    print!("{prompt}");
    io::stdout().flush()?;
    let mut value = String::new();
    io::stdin().read_line(&mut value)?;
    Ok(value.trim().to_owned())
}

async fn two_factor_callback(
    params: TwoFactorCallbackParams,
) -> Result<TwoFactorCallbackResponse, rootcause::Report> {
    if params.unknown {
        println!("La méthode 2FA précédente a échoué.");
    } else if params.sms {
        println!("Un code peut être envoyé par SMS.");
    } else {
        println!("Saisis le code affiché sur ton appareil Apple.");
    }

    if !params.numbers.is_empty() {
        for number in &params.numbers {
            println!("  p{} : {}", number.id, number.number_with_dial_code);
        }
    }
    println!("Commandes : d = appareils Apple, r = renvoyer, pN = téléphone N");

    let value = read_line("Code ou commande 2FA : ")?;
    if value == "d" {
        return Ok(TwoFactorCallbackResponse::SendToDevices);
    }
    if value == "r" {
        return Ok(TwoFactorCallbackResponse::ResendCode);
    }
    if let Some(number) = value.strip_prefix('p') {
        let id = number.parse::<u32>()?;
        return Ok(TwoFactorCallbackResponse::SendSms(id));
    }
    Ok(TwoFactorCallbackResponse::SubmitCode(value))
}

fn select_team(
    teams: Vec<DeveloperTeam>,
    requested: Option<&str>,
) -> Result<DeveloperTeam, String> {
    if let Some(team_id) = requested {
        return teams
            .into_iter()
            .find(|team| team.team_id == team_id)
            .ok_or_else(|| format!("Équipe Apple introuvable : {team_id}"));
    }

    if teams.len() == 1 {
        return teams
            .into_iter()
            .next()
            .ok_or_else(|| "Aucune équipe Apple".to_string());
    }

    println!("Équipes disponibles :");
    for (index, team) in teams.iter().enumerate() {
        println!(
            "  {}. {} ({})",
            index + 1,
            team.name.as_deref().unwrap_or("Sans nom"),
            team.team_id
        );
    }
    let selected = read_line("Numéro de l'équipe : ")
        .map_err(|error| error.to_string())?
        .parse::<usize>()
        .map_err(|_| "Numéro d'équipe invalide".to_string())?;
    teams
        .into_iter()
        .nth(selected.saturating_sub(1))
        .ok_or_else(|| "Numéro d'équipe invalide".to_string())
}

fn certificate_prompt() -> MaxCertsCallbackBox {
    Box::new(|certificates: Vec<DevelopmentCertificate>| {
        Box::pin(async move {
            println!("Apple refuse un nouveau certificat : la limite est atteinte.");
            println!("Révoquer un certificat invalide les applications qui l'utilisent.");
            for (index, certificate) in certificates.iter().enumerate() {
                println!(
                    "  {}. {} — {}",
                    index + 1,
                    certificate.name.as_deref().unwrap_or("Sans nom"),
                    certificate
                        .machine_name
                        .as_deref()
                        .unwrap_or("Machine inconnue")
                );
            }
            let answer = read_line(
                "Numéros à révoquer, séparés par des virgules, ou Entrée pour annuler : ",
            )?;
            if answer.is_empty() {
                return Ok(None);
            }
            let serials = answer
                .split(',')
                .filter_map(|item| item.trim().parse::<usize>().ok())
                .filter_map(|index| certificates.get(index.saturating_sub(1)))
                .filter_map(|certificate| certificate.serial_number.clone())
                .collect::<Vec<_>>();
            if serials.is_empty() {
                Ok(None)
            } else {
                Ok(Some(serials))
            }
        })
    })
}

fn validate_args(args: &Args) -> Result<(), String> {
    if !args.ipa.is_file() {
        return Err(format!("IPA introuvable : {}", args.ipa.display()));
    }
    if args.udid.len() < 20
        || args.udid.len() > 64
        || !args
            .udid
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-')
    {
        return Err("UDID invalide".to_string());
    }
    if args.apple_id.trim().is_empty() || !args.apple_id.contains('@') {
        return Err("Adresse du compte Apple invalide".to_string());
    }
    Ok(())
}

fn copy_tree(source: &Path, destination: &Path) -> io::Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        let metadata = fs::symlink_metadata(&source_path)?;
        if metadata.file_type().is_symlink() {
            symlink(fs::read_link(&source_path)?, destination_path)?;
        } else if metadata.is_dir() {
            copy_tree(&source_path, &destination_path)?;
        } else {
            fs::copy(source_path, destination_path)?;
        }
    }
    Ok(())
}

fn package_ipa(signed_app: &Path, output: &Path) -> Result<(), String> {
    let parent = output.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let absolute_parent = fs::canonicalize(parent).map_err(|error| error.to_string())?;
    let output_name = output
        .file_name()
        .ok_or_else(|| "Nom de sortie IPA invalide".to_string())?;
    let absolute_output = absolute_parent.join(output_name);

    let staging = TempDir::new().map_err(|error| error.to_string())?;
    let payload = staging.path().join("Payload");
    fs::create_dir_all(&payload).map_err(|error| error.to_string())?;
    let app_name = signed_app
        .file_name()
        .ok_or_else(|| "Application signée invalide".to_string())?;
    copy_tree(signed_app, &payload.join(app_name)).map_err(|error| error.to_string())?;

    if absolute_output.exists() {
        fs::remove_file(&absolute_output).map_err(|error| error.to_string())?;
    }
    let output_path = absolute_output.to_string_lossy().into_owned();
    let status = Command::new("zip")
        .args(["-qry", output_path.as_str(), "Payload"])
        .current_dir(staging.path())
        .status()
        .map_err(|error| format!("Impossible de lancer zip : {error}"))?;
    if !status.success() {
        return Err(format!("zip a échoué avec le statut {status}"));
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("Erreur : {error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), String> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|_| "Impossible d'initialiser le fournisseur cryptographique".to_string())?;
    isideload::init().map_err(|error| error.to_string())?;
    tracing::subscriber::set_global_default(
        FmtSubscriber::builder()
            .with_max_level(Level::INFO)
            .with_target(false)
            .finish(),
    )
    .map_err(|error| error.to_string())?;

    let args = Args::parse();
    validate_args(&args)?;
    fs::create_dir_all(&args.state_dir).map_err(|error| error.to_string())?;
    fs::set_permissions(&args.state_dir, fs::Permissions::from_mode(0o700))
        .map_err(|error| error.to_string())?;

    let password = Zeroizing::new(
        rpassword::prompt_password("Mot de passe du compte Apple (reste local) : ")
            .map_err(|error| error.to_string())?,
    );

    println!("Authentification auprès d'Apple…");
    let provider = RemoteV3AnisetteProvider::default().map_err(|error| error.to_string())?;
    let mut account = AppleAccount::builder(&args.apple_id)
        .anisette_provider(provider)
        .login(password.as_str(), two_factor_callback)
        .await
        .map_err(|error| error.to_string())?;

    let mut developer_session = DeveloperSession::from_account(&mut account)
        .await
        .map_err(|error| error.to_string())?;
    let teams = developer_session
        .list_teams()
        .await
        .map_err(|error| error.to_string())?;
    let team = select_team(teams, args.team_id.as_deref())?;

    println!("Enregistrement de l'iPhone dans l'équipe {}…", team.team_id);
    developer_session
        .ensure_device_registered(&team, &args.device_name, &args.udid, None)
        .await
        .map_err(|error| error.to_string())?;

    let storage = FsStorage::new(args.state_dir.join("isideload"));
    let mut sideloader =
        SideloaderBuilder::<MaxCertsCallbackBox, ExtensionsBehaviorCallbackBox>::new(
            developer_session,
            args.apple_id.clone(),
        )
        .team_selection(TeamSelection::First)
        .max_certs_behavior(MaxCertsBehavior::Prompt(certificate_prompt()))
        .extensions_behavior(ExtensionsBehaviorSelection::Fixed(
            ExtensionsBehavior::RegisterAll,
        ))
        .storage(Box::new(storage))
        .machine_name("iPhone iSH bootstrap".to_string())
        .delete_app_after_install(false)
        .build();

    println!("Signature locale de {}…", args.ipa.display());
    let (signed_app, _) = sideloader
        .sign_app(
            args.ipa.clone(),
            Some(team.clone()),
            false,
            None::<fn(f32) -> std::future::Ready<()>>,
            None,
            Some(&args.udid),
        )
        .await
        .map_err(|error| error.to_string())?;

    package_ipa(&signed_app, &args.output)?;
    println!(
        "{}",
        json!({
            "status": "signed",
            "output": args.output,
            "teamIdentifier": team.team_id,
            "udid": args.udid,
        })
    );
    Ok(())
}
