use crate::{discover::{BROADCAST_ADDR, DeviceInfo, PORT},
    protocol::frame::{self, MessageType}};
use std::{error::Error, net::{IpAddr, Ipv4Addr, SocketAddr}, sync::Arc, time::Duration};
use tokio::net::UdpSocket;

fn subnet_broadcast_target() -> Option<SocketAddr> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    let IpAddr::V4(local) = socket.local_addr().ok()?.ip() else {
        return None;
    };
    // supplementary target for networks that filter 255.255.255.255; assumes a /24 LAN
    let mut octets = local.octets();
    octets[3] = 255;
    Some(SocketAddr::new(IpAddr::V4(Ipv4Addr::from(octets)), PORT))
}

// sends the Discover frame from the shared discovery socket (well-known port),
// so peers can reply directly to that port instead of an ephemeral one
pub async fn broadcast_discover(socket:Arc<UdpSocket>,my_device:&DeviceInfo)
    ->Result<(),Box<dyn Error>>{

    let payload = bincode::serialize(my_device)?;
    let frame = frame::encode_frame(MessageType::Discover, &payload);

    let mut targets = vec![format!("{}:{}",BROADCAST_ADDR,PORT)];
    if let Some(extra) = subnet_broadcast_target() {
        let extra = extra.to_string();
        if !targets.contains(&extra) {
            targets.push(extra);
        }
    }

    let mut sent = false;
    for attempt in 0..3 {
        if attempt > 0 {
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
        for target in &targets {
            if socket.send_to(&frame, target).await.is_ok() {
                sent = true;
            }
        }
    }

    if !sent {
        return Err("failed to send discovery broadcast".into());
    }
    Ok(())
}
