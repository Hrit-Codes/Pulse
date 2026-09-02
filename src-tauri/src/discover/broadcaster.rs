use crate::{discover::{BROADCAST_ADDR, DeviceInfo, LOCAL_BIND_ADDR, PORT}, protocol::frame::{self, MessageType, decode_frame}};
use std::{collections::HashMap, error::Error, net::IpAddr, sync::Arc};
use tokio::{net::UdpSocket, sync::Mutex};

pub async fn broadcast_discover(devices:Arc<Mutex<HashMap<String,(DeviceInfo,IpAddr)>>>)->Result<(),Box<dyn Error>>{
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
    //no need to propagate error from here
    let mut buf = [0u8;1024];
    let result = tokio::time::timeout(tokio::time::Duration::from_millis(3000), broadcast_socket.recv_from(&mut buf)).await;

    match result {
        Ok(Ok((bytes_received, responder_addr))) => {
            let (msg_type,payload) = match decode_frame(&buf[..bytes_received]) {
                Ok(v) => v,
                Err(err) => {
                    eprintln!("{:?}",err);
                    return Ok(());
                }
            };
            if let MessageType::DiscoverResponse = msg_type {
                match bincode::deserialize::<DeviceInfo>(&payload) {
                    Ok(data) => {
                        println!("{:?}",data);
                        let mut devices = devices.lock().await;
                        devices.insert(data.id.clone(), (data, responder_addr.ip()));
                    },
                    Err(err) => eprintln!("deserialize error: {}",err)
                };
            }
        }
        Ok(Err(err)) => {
            // recv_from itself failed
            eprintln!("recv error: {}",err);
        }
        Err(_) => {
            // no reply arrived within 3 seconds
            println!("no response within timeout");
        }
    }
    Ok(())
}
