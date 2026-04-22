use std::sync::{Arc, atomic::Ordering};
use anyhow::anyhow;
use quinn::{Connection, Incoming, RecvStream};

use reverb_core::{network::*, failure::failure::{Failure, FailureType}};
use crate::{NEXT_ID, USERS, network::packet_handling::{handle_packet, handle_user_info}};

pub async fn handle_connection(conn: Incoming) -> Result<(), Failure> {
    let conn_bi = conn.await
        .map_err(|e| Failure::from((e.into(), FailureType::Warning)))?;
    println!("Client connected");
    let conn_uni = conn_bi.clone();

    receive_user_info(&conn_uni).await?;

    // separate stream handlers to avoid bidirectional handler stalling unidiractional handler
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

// atomically swap the hashmap stored in USERS for the updated one
// scales poorly as user count increases due to clone()
pub fn add_user(user: User) -> u16 {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed); // wraps around when full overwriting existing users 
    let mut map = (**USERS.load()).clone();
    map.insert(id, user);
    USERS.store(Arc::new(map));
    println!("added user: id {id}"); // DEBUG

    id
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
    let packet = Packet::parse(&data)?;
    handle_packet(packet)?;

    Ok(())
} 

async fn receive_user_info(conn: &Connection) -> Result<(), Failure> {
    let recv = conn.accept_uni().await
        .map_err(|e| Failure::from((e.into(), FailureType::Warning)))?;
    println!("received user info"); // DEBUG
    let data = read_incoming(recv).await?;
    let packet = Packet::parse(&data)?;
    handle_user_info(packet);
    Ok(())
}

async fn read_incoming(mut recv: RecvStream) -> Result<Vec<u8>, Failure> {
    recv.read_to_end(1024).await
        .map_err(|e| Failure::from((e.into(), FailureType::Warning)))
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

