use tauri::AppHandle;
use tokio::{net::UdpSocket, sync::Mutex};
use std::{collections::HashMap, error::Error, net::IpAddr, sync::Arc};
use crate::{discover::{DeviceInfo, PORT, add_device}, protocol::frame::{MessageType, decode_frame, encode_frame}};

pub async fn listen_for_discover(devices:Arc<Mutex<HashMap<String,(DeviceInfo,IpAddr)>>>,
    my_device:DeviceInfo,app:Option<&AppHandle>)->Result<(), Box<dyn Error>>{
    let listen_socket = UdpSocket::bind(format!("0.0.0.0:{}",PORT)).await?;
    let mut buf = [0u8; 1024];
    loop {
        let (bytes_received,sender_addr) = match listen_socket.recv_from(&mut buf).await {
            Ok(v) => v,
            Err(err) => {
                eprintln!("recv error: {}",err);
                continue;
            }
        };
        let (msg_type,payload) = match decode_frame(&buf[..bytes_received]) {
            Ok(v) => v,
            Err(err) => {
                eprintln!("decode error: {:?}",err);
                continue;
            }
        };
        match msg_type {
            MessageType::Discover => {
                let data:DeviceInfo = match bincode::deserialize(&payload) {
                    Ok(v) => v,
                    Err(err) => {
                        eprintln!("deserialize error: {}",err);
                        continue;
                    }
                };
                add_device(&devices, data, sender_addr.ip(), app).await;
                let payload = match bincode::serialize(&my_device) {
                    Ok(v) => v,
                    Err(err) => {
                        eprintln!("error :{}",err);
                        continue;
                    }
                };
                let frame = encode_frame(MessageType::DiscoverResponse, &payload);
                if let Err(err) = listen_socket.send_to(&frame, sender_addr).await {
                    eprintln!("{}",err);
                }

            }
            _ => continue
        }
    }
}
