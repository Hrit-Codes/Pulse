use tokio::{net::UdpSocket, sync::Mutex};
use std::{collections::HashMap, error::Error, net::IpAddr, sync::Arc};
use crate::{discover::{DeviceInfo, PORT}, protocol::frame::{MessageType, decode_frame, encode_frame}};

pub async fn listen_for_discover(devices:Arc<Mutex<HashMap<String,(DeviceInfo,IpAddr)>>>)->Result<(), Box<dyn Error>>{
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
                {
                    println!("{:?}",data);
                    let mut devices = devices.lock().await;
                    devices.insert(data.id.clone(), (data,sender_addr.ip()));
                }
                let dummy_device = DeviceInfo {
                    id: String::from("1234receiver"),
                    name: String::from("dummy device receiver"),
                    port: 9000
                };
                let payload = match bincode::serialize(&dummy_device) {
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
