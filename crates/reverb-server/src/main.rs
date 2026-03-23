use std::{fs, io, sync::Arc};
use anyhow::{Context, Result, bail};
use quinn_proto::crypto::rustls::QuicServerConfig;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};


// The address and port the server will listen on
const LISTEN_ADDR: &str = "127.0.0.1:4433";

// The server version, included in responses for client verification
const VERSION: &str = "0.1.0";

/// Entry point for the server. Installs the default crypto provider, starts the async runtime,
/// and runs the main server logic. Exits with error code 1 if the server fails.
fn main() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    println!("Server starting on {}", LISTEN_ADDR);
    // run the server (async) and handle any errors
    if let Err(e) = tokio::runtime::Runtime::new().unwrap().block_on(run()) {
        eprintln!("Server error: {e}");
        std::process::exit(1);
    }
}


async fn run() -> Result<()> {
    // --- Certificate and key loading/generation ---
    let (certs, key) = {
        let path = std::path::Path::new("certs");
        let cert_path = path.join("cert.der");
        let key_path = path.join("key.der");
        // Try to read existing certificate and key files
        let (cert, key) = match fs::read(&cert_path).and_then(|x| Ok((x, fs::read(&key_path)?))) {
            // If both files exist, load them
            Ok((cert, key)) => (
                CertificateDer::from(cert),
                PrivateKeyDer::try_from(key).map_err(anyhow::Error::msg)?,
            ),
            // If not found, generate a new self-signed certificate and key
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
                // Generate a self-signed certificate for "localhost"
                let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
                let key = PrivatePkcs8KeyDer::from(cert.key_pair.serialize_der());
                let cert = cert.cert.into();
                // Ensure the directory exists
                fs::create_dir_all(path).context("failed to create certificate directory")?;
                fs::write(&cert_path, &cert).context("failed to write certificate")?;
                fs::write(&key_path, key.secret_pkcs8_der())
                    .context("failed to write private key")?;
                (cert, key.into())
            }
            // Any other error is fatal
            Err(e) => {
                bail!("failed to read certificate: {}", e);
            }
        };
        (vec![cert], key)
    };

    // --- TLS/QUIC server configuration ---
    // Build a rustls server config with no client authentication and our certificate
    let server_crypto = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;


    // Wrap the rustls config for use with Quinn (QUIC implementation)
    let mut server_config =
        quinn::ServerConfig::with_crypto(Arc::new(QuicServerConfig::try_from(server_crypto)?));
    // Set transport-level options: here, disable unidirectional streams
    let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();
    transport_config.max_concurrent_uni_streams(0_u8.into());

    // --- Start the QUIC endpoint (server) ---
    let endpoint = quinn::Endpoint::server(server_config, LISTEN_ADDR.parse()?)?;
    println!("Server listening and waiting for one client...");

    // --- Accept a single client connection ---
    if let Some(conn) = endpoint.accept().await {
        // Wait for the connection handshake to complete
        let conn = conn.await?;
        println!("Client connected");

        // Accept a bidirectional stream from the client
        let (mut send, mut recv) = conn.accept_bi().await?;
        // Read up to 1024 bytes from the client
        let data = recv.read_to_end(1024).await?;
        println!("Received: {}", String::from_utf8_lossy(&data));

        // Prepare and send a response back to the client
        let response = format!("Server received {} bytes", data.len());
        send.write_all(response.as_bytes()).await?;
        send.finish()?;

        // Wait for all packets to be sent before shutting down
        endpoint.wait_idle().await;
        println!("Response sent, server exiting");
    }

    Ok(())
}
