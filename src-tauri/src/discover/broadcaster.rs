use crate::{discover::{BROADCAST_ADDR, DeviceInfo, LOCAL_BIND_ADDR, PORT},
    protocol::frame::{self, MessageType, decode_frame}};
use std::{collections::HashMap, error::Error, net::IpAddr, sync::Arc};
use tauri::AppHandle;
use tokio::{net::UdpSocket, sync::Mutex};
use super::add_device;

pub async fn broadcast_discover(devices:Arc<Mutex<HashMap<String,(DeviceInfo,IpAddr)>>>,my_device:DeviceInfo,
    app:Option<&AppHandle>)
    ->Result<(),Box<dyn Error>>{

    let payload = bincode::serialize(&my_device)?;

    let frame = frame::encode_frame(frame::MessageType::Discover, &payload);

    let broadcast_socket = UdpSocket::bind(LOCAL_BIND_ADDR).await?;
    broadcast_socket.set_broadcast(true)?;

    broadcast_socket.send_to(&frame, format!("{}:{}",BROADCAST_ADDR,PORT)).await?;
    let mut buf = [0u8;1024];
    let result = tokio::time::timeout(tokio::time::Duration::from_millis(3000), broadcast_socket.recv_from(&mut buf)).await;

    match result {
        Ok(Ok((bytes_received, responder_addr))) => {
            let (msg_type,payload) = match decode_frame(&buf[..bytes_received]) {
                Ok(v) => v,
                Err(err) => {
                    return Err(format!("discover error: {:?}",err).into())
                }
            };
            if let MessageType::DiscoverResponse = msg_type {
                match bincode::deserialize::<DeviceInfo>(&payload) {
                    Ok(data) => {
                        add_device(&devices, data, responder_addr.ip(), app).await;
                    },
                    Err(err) => return Err(format!("deserialize error: {}",err).into())
                };
            }
        }
        Ok(Err(err)) => {
            // recv_from itself failed
            return Err(err.into());
        }
        Err(_) => {
            // no reply arrived within 3 seconds leave it be
        }
    }
    Ok(())
}
