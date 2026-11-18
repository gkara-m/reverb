use std::{fs, net::SocketAddr, sync::Arc};

use log::{debug, error, info};
use quinn::{Connection, TransportConfig};
use quinn_proto::crypto::rustls::QuicClientConfig;
use rustls::pki_types::CertificateDer;

use anyhow::{Result, anyhow};

use crate::{Command, MAIN_SENDER, config::{config::config, internet::server_config}};
use reverb_core::{network_command::helpers::QueryOrNotify, failure::failure::{Failure, FailureType}, network::Packet};


pub(super) fn start_communicator_thread() {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        match tokio::runtime::Runtime::new().unwrap().block_on(async{
            let conn = connect_to_server().await?;
            MAIN_SENDER.get().unwrap().clone().send(Command::ServerUpdateStatus(crate::internal::internet::connection::ConnectionStatus::Connected(tx))).unwrap_or_else(|e| error!("Failed to send server update status command: {}", e));
            for packet in rx {
                // Handle incoming messages
                match packet.payload.query_or_notify() {
                    QueryOrNotify::Query => query(conn.clone(), packet).await?,
                    QueryOrNotify::Notify => notify(conn.clone(), packet).await?,
                }
            }
            Ok(())
        }) {
            Ok(_) => info!("Communicator thread exited normally"),
            Err(e) => {
                MAIN_SENDER.get().unwrap().clone().send(Command::Failure(e)).unwrap_or_else(|e| error!("Failed to send failure command: {}", e));
                MAIN_SENDER.get().unwrap().clone().send(Command::ServerUpdateStatus(crate::internal::internet::connection::ConnectionStatus::NotConnected)).unwrap_or_else(|e| error!("Failed to send server update status command: {}", e));
            }
        };
    });
}

async fn connect_to_server() -> Result<Connection, Failure> {
    info!("Starting connection process to server: {}", server_config()?.server_address);
    // Locate the directory where the server's certificate is stored (shared location)
    let data_folder = &config()?.data_folder;
    let path = std::path::Path::new(data_folder); // TODO add command to add it to the config
    let cert_path = path.join(&server_config()?.server_cert_path);


    debug!("Loading server certificate from: {:?}", cert_path);

    // Create an empty root certificate store
    let mut roots = rustls::RootCertStore::empty();
    // If the server's certificate exists, add it to the root store for TLS validation
    if let Ok(cert) = fs::read(&cert_path) {
        if let Err(e) = roots.add(CertificateDer::from(cert)) {
            return Err(Failure::from((anyhow!(e), FailureType::Warning)));
        }
    }

    if roots.is_empty() {
        return Err(Failure::from((anyhow!("No valid server certificate found at path: {:?}", cert_path), FailureType::Warning)));
    }

    // Build the client cryptographic configuration with the root certificates and no client authentication
    let client_crypto = rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();


    // Wrap the client crypto config in a Quinn QUIC client config
    let mut client_config = match QuicClientConfig::try_from(client_crypto) {
        Ok(c) => quinn::ClientConfig::new(Arc::new(c)),
        Err(e) => {
            return Err(Failure::from((e.into(), FailureType::Warning)));
        }
    };

    // set the keep alive
    let mut transport_config = TransportConfig::default();
    transport_config.keep_alive_interval(Some(std::time::Duration::from_secs(5)));
    client_config.transport_config(transport_config.into());

    // Create a new QUIC endpoint for the client, binding to an ephemeral port on all interfaces
    let addr = "[::]:0".parse::<SocketAddr>()
        .map_err(|e| Failure::from((e.into(), FailureType::Warning)))?;
    let mut endpoint = quinn::Endpoint::client(addr)
        .map_err(|e| Failure::from((e.into(), FailureType::Warning)))?;
    // Set the default client configuration for outgoing connections
    endpoint.set_default_client_config(client_config);

    // get the server name for TLS validation from the server config
    let host = server_config()?.server_name.clone();

    // Parse the server address string into a SocketAddr
    let remote = server_config()?.server_address.parse::<SocketAddr>()
        .map_err(|e| Failure::from((e.into(), FailureType::Warning)))?;

    // Initiate a QUIC connection to the server with the given address and host
    let connecting = endpoint.connect(remote, host.as_str())
        .map_err(|e| Failure::from((e.into(), FailureType::Warning)))?;
    let conn = connecting.await
        .map_err(|e| Failure::from((e.into(), FailureType::Warning)))?;
    info!("Successfully connected to server at {}", server_config()?.server_address);


    // send notification with complete header for server to identify us
    debug!("sending information about self to server");
    let packet = Packet::new(
        config()?.username.clone().as_str(),
        "none",
        Box::new(reverb_core::network_command::user_data::UserData {
            echo_avaliable: server_config()?.echo_avaliable
        })
    )?;
    notify(conn.clone(), packet).await?;

    Ok(conn)
}
    


async fn query(conn: Connection, packet: Packet) -> Result<(), Failure> {

    debug!("Sending query to server");

    // Open a bidirectional stream to the server
    let (mut send, mut recv) = conn.open_bi().await
    .map_err(|e| Failure::from((e.into(), "connection is unusable", FailureType::Warning)))?;

    // Send the message to the server
    let message = &packet.serialize()?;
    send.write_all(message).await
    .map_err(|e| Failure::from((e.into(), "sending data over internet", FailureType::Warning)))?;

    // Indicate that no more data will be sent on this stream
    send.finish()
    .map_err(|e| Failure::from((e.into(), "closing the sending data over internet", FailureType::Warning)))?;
    debug!("Sent query");

    // Wait for the server's response and print it
    match recv.read_to_end(1024).await {
        Ok(data) => {
            let response_packet = Packet::parse(&data)?;
            debug!("Received response from server");
            debug!("Response version: {:?}", response_packet.version());
            debug!("Response username: {}", response_packet.username());
            debug!("Response group: {}", response_packet.group());
            MAIN_SENDER.get().unwrap().clone().send(Command::ServerResponse(response_packet))
            .unwrap_or_else(|e| error!("Failed to send server response command to main sender: {}", e));
        }
        Err(e) => {
            error!("Receive error: {e}");
        }
    }

    Ok(())
}

async fn notify(conn: Connection, packet: Packet) -> Result<(), Failure> {
    
    debug!("Sending notification to server");

    // Open a unidirectional stream to server 
    let mut send = conn.open_uni().await
    .map_err(|e| Failure::from((e.into(), "connection is unusable", FailureType::Warning)))?;

    // Send the message to the server 
    let message = &packet.serialize()?;
    send.write_all(message).await
    .map_err(|e| Failure::from((e.into(), "sending data over internet", FailureType::Warning)))?;

    // Indicate that no more data will be sent on this stream 
    send.finish()
    .map_err(|e| Failure::from((e.into(), "closing the sending data over internet", FailureType::Warning)))?;
    
    debug!("Sent notification");

    Ok(())
}
