use clap::Parser;
use idevice::remote_pairing::{
    PAIRABLE_HOST_SERVICE_TYPE, PairableHost, PairableHostInfo, RpPairingFile, RpPairingSocket,
};
use mdns_sd::{ServiceDaemon, ServiceInfo};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::{net::TcpListener, time::timeout};

#[derive(Parser, Debug)]
#[command(version, about = "Jumelage local iOS 27 depuis iSH")]
struct Args {
    #[arg(long, default_value = ".ios-bootstrap-state")]
    state_dir: PathBuf,

    #[arg(long, default_value = "iSH iOS Bootstrap")]
    name: String,

    #[arg(long, default_value = "Mac17,7")]
    model: String,
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn parse_hex_16(value: &str) -> Option<[u8; 16]> {
    if value.len() != 32 {
        return None;
    }
    let mut bytes = [0_u8; 16];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16).ok()?;
    }
    Some(bytes)
}

fn saved_host_irk(path: &Path) -> Option<[u8; 16]> {
    let data: Value = serde_json::from_slice(&fs::read(path).ok()?).ok()?;
    parse_hex_16(data.get("hostAltIrk")?.as_str()?)
}

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("Erreur de jumelage : {error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), String> {
    let args = Args::parse();
    fs::create_dir_all(&args.state_dir).map_err(|error| error.to_string())?;
    fs::set_permissions(&args.state_dir, fs::Permissions::from_mode(0o700))
        .map_err(|error| error.to_string())?;

    let pairing_path = args.state_dir.join("rp-pairing.plist");
    let device_path = args.state_dir.join("device.json");
    let mut pairing_file = match RpPairingFile::read_from_file(&pairing_path).await {
        Ok(mut existing) => {
            existing.alt_irk = None;
            println!("Réutilisation de l’identité de jumelage locale.");
            existing
        }
        Err(_) => RpPairingFile::generate(&args.name),
    };

    let mut host_info = PairableHostInfo::generate(&args.name, &args.model);
    if let Some(saved) = saved_host_irk(&device_path) {
        host_info.alt_irk = saved;
    }
    let host_alt_irk = host_info.alt_irk;
    let records = host_info.mdns_txt_records(pairing_file.identifier());

    let listener = TcpListener::bind("0.0.0.0:0")
        .await
        .map_err(|error| format!("écoute locale impossible : {error}"))?;
    let port = listener
        .local_addr()
        .map_err(|error| error.to_string())?
        .port();

    let properties = records.into_iter().collect::<HashMap<_, _>>();
    let service = ServiceInfo::new(
        PAIRABLE_HOST_SERVICE_TYPE,
        pairing_file.identifier(),
        "ios-bootstrap.local.",
        "",
        port,
        properties,
    )
    .map_err(|error| format!("service Bonjour invalide : {error}"))?
    .enable_addr_auto();
    let service_fullname = service.get_fullname().to_owned();
    let mdns = ServiceDaemon::new().map_err(|error| format!("Bonjour indisponible : {error}"))?;
    mdns.register(service)
        .map_err(|error| format!("publication Bonjour impossible : {error}"))?;

    println!();
    println!("Jumelage prêt sur le port {port}.");
    println!("1. Laisse iSH ouvert quelques secondes.");
    println!("2. Ouvre Réglages → Confidentialité et sécurité → Mode développeur.");
    println!("3. Touche « Jumeler avec {} ».", args.name);
    println!("4. Reviens immédiatement dans iSH pour lire le code, puis saisis-le dans Réglages.");
    println!();

    let (stream, peer_address) = timeout(Duration::from_secs(300), listener.accept())
        .await
        .map_err(|_| "aucune connexion reçue pendant cinq minutes".to_string())?
        .map_err(|error| format!("connexion de l’iPhone refusée : {error}"))?;

    let socket = RpPairingSocket::new_device(stream);
    let mut host = PairableHost::new(socket, host_info);
    let peer = timeout(
        Duration::from_secs(300),
        host.accept(&mut pairing_file, |pin| async move {
            println!();
            println!("CODE DE JUMELAGE : {pin}");
            println!("Retourne dans Réglages et saisis ce code.");
            println!();
        }),
    )
    .await
    .map_err(|_| "le dialogue de jumelage a expiré".to_string())?
    .map_err(|error| error.to_string())?;

    pairing_file
        .write_to_file(&pairing_path)
        .await
        .map_err(|error| format!("écriture du jumelage impossible : {error}"))?;
    fs::set_permissions(&pairing_path, fs::Permissions::from_mode(0o600))
        .map_err(|error| error.to_string())?;

    let state = json!({
        "deviceName": peer.name,
        "deviceModel": peer.model,
        "udid": peer.remotepairing_udid,
        "deviceAddress": peer_address.ip().to_canonical().to_string(),
        "hostAltIrk": hex(&host_alt_irk),
        "pairingFile": pairing_path,
    });
    fs::write(
        &device_path,
        serde_json::to_vec_pretty(&state).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    fs::set_permissions(&device_path, fs::Permissions::from_mode(0o600))
        .map_err(|error| error.to_string())?;

    let _ = mdns.unregister(&service_fullname);
    let _ = mdns.shutdown();
    println!(
        "Jumelage enregistré localement pour {}.",
        state["deviceName"]
    );
    Ok(())
}
