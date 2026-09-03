use std::{error::Error, net::SocketAddr, path::Path};

use uuid::Uuid;
use crate::{protocol::frame::{MessageType, encode_frame}, transfer::{TransferReject, TransferRequest, stream::{connect_to_peer, read_frame, send_frame}}};

pub async fn send_file(addr:SocketAddr, file_path: &Path)->Result<(),Box<dyn Error>>{

    let transfer_id = Uuid::new_v4().to_string(); 
    let file_data = tokio::fs::metadata(file_path).await?;
    let filename = file_path.file_name()
                            .ok_or("invalid file path")?
                            .to_string_lossy().to_string();
    let transfer_req = TransferRequest {
        transfer_id,
        filename,
        file_size: file_data.len()
    };

    let payload = bincode::serialize(&transfer_req)?;
    let frame = encode_frame(MessageType::TransferRequest, &payload);
    
    let mut tcp_stream = connect_to_peer(addr).await?;
    send_frame(&mut tcp_stream, &frame).await?;
    
    let mut buffer = Vec::new();
    let (response_type,response_payload) = read_frame(&mut tcp_stream, &mut buffer).await?;

    match response_type {
        MessageType::TransferReject => {
            let reject_msg:TransferReject =  bincode::deserialize(&response_payload)?;
            println!("Transfer rejected: {}", reject_msg.reason);
            return Ok(());
        },
        MessageType::TransferAccept => {
            println!("transfer");
        }
        _ => {
            eprintln!("unexpected response: {:?}",response_type);
        }
    }

    
    Ok(())
}
