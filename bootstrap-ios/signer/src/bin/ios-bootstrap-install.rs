use clap::Parser;
use idevice::{
    provider::RsdProvider,
    remote_pairing::{
        RemotePairingClient, RpPairingFile, RpPairingSocket, connect_tls_psk_tunnel_native,
    },
    rsd::RsdHandshake,
    tcp::adapter::Adapter,
    utils::installation::install_package_with_callback_rsd,
};
use serde_json::Value;
use std::{
    fs,
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    time::Duration,
};
use tokio::{net::TcpStream, time::timeout};

const REMOTE_PAIRING_PORT: u16 = 49152;

#[derive(Parser, Debug)]
#[command(
    version,
    about = "Installation directe d’une IPA via le tunnel local iOS"
)]
struct Args {
    #[arg(long)]
    ipa: PathBuf,

    #[arg(long, default_value = ".ios-bootstrap-state")]
    state_dir: PathBuf,

    #[arg(long, default_value = "10.7.0.1")]
    device_ip: IpAddr,
}

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("Erreur d’installation : {error}");
        std::process::exit(1);
    }
}

async fn connect_candidate(address: IpAddr, port: u16) -> Result<TcpStream, String> {
    timeout(
        Duration::from_secs(8),
        TcpStream::connect(SocketAddr::new(address, port)),
    )
    .await
    .map_err(|_| format!("délai dépassé vers {address}:{port}"))?
    .map_err(|error| format!("{address}:{port} : {error}"))
}

async fn run() -> Result<(), String> {
    let args = Args::parse();
    if !args.ipa.is_file() {
        return Err(format!("IPA introuvable : {}", args.ipa.display()));
    }

    let pairing_path = args.state_dir.join("rp-pairing.plist");
    let mut pairing = RpPairingFile::read_from_file(&pairing_path)
        .await
        .map_err(|error| format!("jumelage absent ou invalide : {error}"))?;
    let device_state: Value = serde_json::from_slice(
        &fs::read(args.state_dir.join("device.json"))
            .map_err(|error| format!("état de l’iPhone absent : {error}"))?,
    )
    .map_err(|error| error.to_string())?;

    println!("Connexion RPPairing via LocalDevVPN…");
    let stream = connect_candidate(args.device_ip, REMOTE_PAIRING_PORT).await?;
    let socket = RpPairingSocket::new(stream);
    let mut client = RemotePairingClient::new(socket, "iSH iOS Bootstrap");
    client
        .connect(&mut pairing, || async { "000000".to_string() })
        .await
        .map_err(|error| format!("vérification du jumelage : {error}"))?;
    let encryption_key = client.encryption_key().to_vec();
    let tunnel_port = client
        .create_tcp_listener()
        .await
        .map_err(|error| format!("création du tunnel : {error}"))?;
    println!("Tunnel annoncé sur le port {tunnel_port}.");

    let mut candidates = vec![args.device_ip, IpAddr::from([127, 0, 0, 1])];
    if let Some(address) = device_state
        .get("deviceAddress")
        .and_then(Value::as_str)
        .and_then(|value| value.parse::<IpAddr>().ok())
        && !candidates.contains(&address)
    {
        candidates.push(address);
    }

    let mut connected = None;
    let mut failures = Vec::new();
    for address in candidates {
        match connect_candidate(address, tunnel_port).await {
            Ok(stream) => {
                println!("Port du tunnel atteint via {address}.");
                connected = Some(stream);
                break;
            }
            Err(error) => failures.push(error),
        }
    }
    let stream = connected.ok_or_else(|| {
        format!(
            "le port de tunnel créé par iOS reste inaccessible : {}",
            failures.join(" ; ")
        )
    })?;

    let tunnel = connect_tls_psk_tunnel_native(stream, &encryption_key)
        .await
        .map_err(|error| format!("tunnel TLS-PSK/CDTunnel : {error}"))?;
    let info = tunnel.info.clone();
    let client_ip = info
        .client_address
        .parse::<IpAddr>()
        .map_err(|error| error.to_string())?;
    let server_ip = info
        .server_address
        .parse::<IpAddr>()
        .map_err(|error| error.to_string())?;
    let mut adapter = Adapter::new(Box::new(tunnel.into_inner()), client_ip, server_ip);
    adapter.set_mss((info.mtu as usize).saturating_sub(60));
    let mut handle = adapter.to_async_handle();
    let rsd_stream = handle
        .connect_to_service_port(info.server_rsd_port)
        .await
        .map_err(|error| format!("connexion RSD : {error}"))?;
    let mut handshake = RsdHandshake::new(rsd_stream)
        .await
        .map_err(|error| format!("dialogue RSD : {error}"))?;

    println!("Envoi et installation de l’IPA…");
    install_package_with_callback_rsd(
        &mut handle,
        &mut handshake,
        &args.ipa,
        None,
        |(percent, ())| async move {
            println!("Installation : {percent}%");
        },
        (),
    )
    .await
    .map_err(|error| format!("installation_proxy : {error}"))?;
    println!("IPA installée avec succès.");
    Ok(())
}
