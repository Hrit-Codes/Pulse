use crate::{discover::{BROADCAST_ADDR, DeviceInfo, LOCAL_BIND_ADDR, PORT}, protocol::frame};
use std::{error::Error};
use tokio::net::UdpSocket;

pub async fn broadcast_discover()->Result<(),Box<dyn Error>>{
    let dummy_device = DeviceInfo {
        id: String::from("1234"),
        name: String::from("dummy device"),
        port: 9000
    };

    let payload = bincode::serialize(&dummy_device)?;

    let frame = frame::encode_frame(frame::MessageType::Discover, &payload);

    let broadcast_socket = UdpSocket::bind(LOCAL_BIND_ADDR).await?;
    broadcast_socket.set_broadcast(true)?;

    broadcast_socket.send_to(&frame, format!("{}:{}",BROADCAST_ADDR,PORT)).await?;
    Ok(())
}
