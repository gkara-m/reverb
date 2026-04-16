use std::{fs, io, sync::{Arc, mpsc::Sender}};
use anyhow::anyhow;
use quinn::{Connection, Endpoint, Incoming, RecvStream};
use quinn_proto::crypto::rustls::QuicServerConfig;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};

use reverb_core::{network::*, failure::failure::{Failure, FailureType}};
use crate::handle_packet;

pub async fn handle_connection(conn: Incoming) -> Result<(), Failure> {
    // Wait for the connection handshake to complete
    let conn_bi = conn.await
        .map_err(|e| Failure::from((e.into(), FailureType::Warning)))?;
    println!("Client connected");
    let conn_uni = conn_bi.clone();

    tokio::spawn(async move {
        loop {
            if let Err(e) = handle_bi(&conn_bi).await {
                eprintln!("Server bi_connection error: {e}");
                return;
            }
        }
    });

    tokio::spawn(async move {
        loop {
            if let Err(e) = handle_uni(&conn_uni).await {
                eprintln!("Server uni_connection error: {e}");
                return;
            }
        }
    });

    Ok(())
}

async fn handle_bi(conn: &Connection) -> Result<(), Failure> {
    let (mut send, mut recv) = conn.accept_bi().await
        .map_err(|e| Failure::from((e.into(), FailureType::Warning)))?;

    let data = read_incoming(recv).await?;
    let packet = Packet::parse(&data)?;

    // Prepare and send a response back to the client
    let response = handle_packet(packet)?
        .ok_or(Failure::from((anyhow!("error creating response packet"), FailureType::Warning)))?;
    send.write_all(&response.serialize()?).await;
    send.finish();
    
    Ok(())
}

async fn handle_uni(conn: &Connection) -> Result<(), Failure> {
    let mut recv = conn.accept_uni().await
        .map_err(|e| Failure::from((e.into(), FailureType::Warning)))?;

    let data = read_incoming(recv).await?;
    let packet = Packet::parse(&data)?;

    Ok(())
}

async fn read_incoming(mut recv: RecvStream) -> Result<Vec<u8>, Failure> {
    recv.read_to_end(1024).await
        .map_err(|e| Failure::from((e.into(), FailureType::Warning)))
}

pub enum UserAvailability {
    OpenToEcho,
    ClosedToEcho
}

pub struct User {
    username: String,
    group: String,
    availability: UserAvailability, 
}

