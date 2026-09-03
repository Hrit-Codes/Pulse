use std::{error::Error, net::SocketAddr, path::Path};

use sha2::{Sha256,Digest};
use uuid::Uuid;
use crate::{protocol::frame::{MessageType, encode_frame}, transfer::{CHUNK_SIZE, Chunk, FileMetadata, TransferComplete, TransferReject, TransferRequest, stream::{connect_to_peer, read_frame, send_frame}}};

pub async fn send_file(addr:SocketAddr, file_path: &Path)->Result<(),Box<dyn Error>>{

    let transfer_id = Uuid::new_v4().to_string(); 
    let file_data = tokio::fs::metadata(file_path).await?;
    let filename = file_path.file_name()
                            .ok_or("invalid file path")?
                            .to_string_lossy().to_string();
    let transfer_req = TransferRequest {
        transfer_id:transfer_id.clone(),
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
            let file_bytes = tokio::fs::read(file_path).await?;
            let mut hasher = Sha256::new();
            hasher.update(&file_bytes);
            let hash_bytes = hasher.finalize();
            let file_hash = hex::encode(hash_bytes);
            let file_metadata = FileMetadata {
                transfer_id:transfer_id.clone(),
                file_hash:file_hash.clone(),
                chunk_size:CHUNK_SIZE,
                total_chunks:(file_bytes.len()+CHUNK_SIZE-1)/CHUNK_SIZE
            };
            let payload = bincode::serialize(&file_metadata)?;
            let frame = encode_frame(MessageType::FileMetadata, &payload);

            send_frame(&mut tcp_stream, &frame).await?; //file metadata has been sent

            for (idx,chunk) in file_bytes.chunks(CHUNK_SIZE).enumerate() {
                let chunk = Chunk{
                    transfer_id:transfer_id.clone(),
                    chunk_index:idx,
                    data: chunk.to_vec()
                };
                let payload = bincode::serialize(&chunk)?;
                let frame = encode_frame(MessageType::Chunk, &payload);
                send_frame(&mut tcp_stream, &frame).await?;
                //should i go for chunk ack?
            }
            let (response_type, response_payload) = read_frame(&mut tcp_stream, &mut buffer).await?;
            match response_type {
                MessageType::TransferComplete => {
                    let complete:TransferComplete = bincode::deserialize(&response_payload)?;
                    if complete.success && complete.receiver_hash == file_hash {
                        println!("Transfer verified successfully");
                    } else {
                        eprintln!("Transfer failed or hash mismatch");
                    }
                },
                _ => {
                    eprintln!("unexpected response: {:?}", response_type);
                }
            };
            
        }
        _ => {
            eprintln!("unexpected response: {:?}",response_type);
        }
    }

    
    Ok(())
}
