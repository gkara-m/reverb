use anyhow::anyhow;
use quinn::{Connection, Incoming, RecvStream};

use reverb_core::{network::*, failure::failure::{Failure, FailureType}};
use crate::{SERVER_GROUP, SERVER_NAME, add_user, handle_packet};

pub async fn handle_connection(conn: Incoming) -> Result<(), Failure> {
    // Wait for the connection handshake to complete
    let conn_bi = conn.await
        .map_err(|e| Failure::from((e.into(), FailureType::Warning)))?;
    println!("Client connected");
    let conn_uni = conn_bi.clone();

    let user_info_packet = request_user_data(&conn_bi).await?;
    handle_user_info(user_info_packet);

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
    let (mut send, recv) = conn.accept_bi().await
        .map_err(|e| Failure::from((e.into(), FailureType::Warning)))?;

    let data = read_incoming(recv).await?;
    let packet = Packet::parse(&data)?;

    // Prepare and send a response back to the client
    let response = handle_packet(packet)?
        .ok_or(Failure::from((anyhow!("error creating response packet"), FailureType::Warning)))?;
    send.write_all(&response.serialize()?).await
        .map_err(|e| Failure::from((e.into(), FailureType::Warning)))?;
    send.finish()
        .map_err(|e| Failure::from((e.into(), FailureType::Warning)))?;
    
    Ok(())
}

async fn handle_uni(conn: &Connection) -> Result<(), Failure> {
    let recv = conn.accept_uni().await
        .map_err(|e| Failure::from((e.into(), FailureType::Warning)))?;

    let data = read_incoming(recv).await?;
    let packet = Packet::parse(&data)?; // TODO

    Err(Failure::from((anyhow!("feature not implemented yet"), FailureType::Fatal)))
} 

async fn read_incoming(mut recv: RecvStream) -> Result<Vec<u8>, Failure> {
    recv.read_to_end(1024).await
        .map_err(|e| Failure::from((e.into(), FailureType::Warning)))
}

async fn request_user_data(conn: &Connection) -> Result<Packet, Failure> {
    let (mut send, mut recv) = conn.open_bi().await
        .map_err(|e| Failure::from((e.into(), FailureType::Warning)))?;
    let request_packet = Packet {
        version: NETWORK_VERSION,
        username: SERVER_NAME.to_string(),
        group:SERVER_GROUP.to_string(),
        payload: Box::new(RequestUserData {})
    };
    send.write_all(&request_packet.serialize()?).await
        .map_err(|e| Failure::from((e.into(), FailureType::Warning)))?;
    send.finish()
        .map_err(|e| Failure::from((e.into(), FailureType::Warning)))?;
    let incoming_data = recv.read_to_end(1024).await
        .map_err(|e| Failure::from((e.into(), FailureType::Warning)))?;
    Packet::parse(&incoming_data)

}

fn handle_user_info(packet: Packet) -> u16 {
    let username = packet.username;
    let group = packet.group;
    let availability = UserAvailability::ClosedToEcho;
    let user = User {
        username,
        group,
        availability
    };

    add_user(user)
}

#[derive(Debug, Clone)]
pub enum UserAvailability {
    OpenToEcho,
    ClosedToEcho
}

#[derive(Debug, Clone)]
pub struct User {
    pub username: String,
    pub group: String,
    pub availability: UserAvailability, 
}

