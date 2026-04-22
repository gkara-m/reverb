use std::{fs, io, sync::Arc};
use anyhow::anyhow;
use quinn::Endpoint;
use quinn_proto::crypto::rustls::QuicServerConfig;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use reverb_core::failure::failure::{Failure, FailureType};

use crate::LISTEN_ADDR;


pub fn startup() -> Result<Endpoint, Failure> {
    let (certs, key) =  load_generate_certificate_and_key()?;

    // --- TLS/QUIC server configuration ---
    // Build a rustls server config with no client authentication and our certificate
    let server_crypto = rustls::ServerConfig::builder().with_no_client_auth().with_single_cert(certs, key)
        .map_err(|e| Failure::from((e.into(), FailureType::Fatal)))?;

    // Wrap the rustls config for use with Quinn (QUIC implementation)
    let mut server_config =
        quinn::ServerConfig::with_crypto(Arc::new(QuicServerConfig::try_from(server_crypto)
            .map_err(|e| Failure::from((e.into(), FailureType::Fatal)))?));
    // Set transport-level options: here, disable unidirectional streams
    let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();
    transport_config.max_concurrent_uni_streams(16_u8.into());

    // --- Start the QUIC endpoint (server) ---
    let endpoint = quinn::Endpoint::server(server_config, LISTEN_ADDR.parse()
        .map_err(|e: std::net::AddrParseError| Failure::from((e.into(), FailureType::Fatal)))?) // address parse error
        .map_err(|e| Failure::from((e.into(), FailureType::Fatal)))?;
    println!("Server listening and waiting for one client...");

    Ok(endpoint)
}

pub fn load_generate_certificate_and_key() -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>), Failure> {
    let (cert, key) = {
        let path = std::path::Path::new("certs");
        let cert_path = path.join("cert.der");
        let key_path = path.join("key.der");
        // Try to read existing certificate and key files
        let (cert, key) = match fs::read(&cert_path).and_then(|x| Ok((x, fs::read(&key_path)?))) {
            // If both files exist, load them
            Ok((cert, key)) => (
                CertificateDer::from(cert),
                PrivateKeyDer::try_from(key).map_err(|e| Failure::from((anyhow!(e), FailureType::Fatal)))?,
            ),
            // If not found, generate a new self-signed certificate and key
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
                // Generate a self-signed certificate for "localhost"
                let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
                let key = PrivatePkcs8KeyDer::from(cert.key_pair.serialize_der());
                let cert = cert.cert.into();
                // Ensure the directory exists
                if let Err(e) = fs::create_dir_all(path) {
                    return Err(Failure::from((anyhow!("failed to create certificate directory: {e}"), FailureType::Fatal)));
                };
                if let Err(e) = fs::write(&cert_path, &cert) {
                    return Err(Failure::from((anyhow!("failed to write certificate: {e}"), FailureType::Fatal)));
                };
                if let Err(e) = fs::write(&key_path, key.secret_pkcs8_der()) {
                    return Err(Failure::from((anyhow!("failed to write private key: {e}"), FailureType::Fatal)));
                };
                (cert, key.into())
            }
            // Any other error is fatal
            Err(e) => {
                return Err(Failure::from((anyhow!("failed to load/generate certificate/key: {e}"), FailureType::Fatal)));
            }
        };
        (vec![cert], key)

    };

    Ok((cert, key))
}
