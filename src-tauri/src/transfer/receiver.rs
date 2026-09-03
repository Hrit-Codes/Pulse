use std::{error::Error,net::SocketAddr};

use sha2::{Sha256,Digest};
use tokio::net::{TcpListener, TcpStream};

use crate::{protocol::frame::{MessageType, encode_frame}, transfer::{Chunk, FileMetadata, TransferAccept, TransferComplete, TransferRequest, stream::{read_frame, send_frame}}};

pub async fn receive_file(addr:SocketAddr)-> Result<(), Box<dyn Error>>{
    let listener = TcpListener::bind(addr).await?;
    loop {
        let (stream,_peer_addr) = listener.accept().await?;
        if let Err(err) = handle_connection(stream).await {
            eprintln!("transfer error: {}",err);
        }
    }
}

async fn handle_connection(mut tcp_stream:TcpStream)->Result<(), Box<dyn Error>>{
    let mut buffer = Vec::new();
    let (msg_type, payload) = read_frame(&mut tcp_stream, &mut buffer).await?;
    match msg_type {
        MessageType::TransferRequest => {
            let request: TransferRequest = bincode::deserialize(&payload)?;
            println!("Incoming transfer: {} ({} bytes)", request.filename, request.file_size);
            //going to accept it for now. rejection will be implemented later
            let transfer_accept:TransferAccept = TransferAccept{
                transfer_id:request.transfer_id
            };
            let payload = bincode::serialize(&transfer_accept)?;
            let frame = encode_frame(MessageType::TransferAccept, &payload);
            send_frame(&mut tcp_stream, &frame).await?;

            let (msg_type, payload) = read_frame(&mut tcp_stream, &mut buffer).await?;
            match msg_type {
                MessageType::FileMetadata => {
                    let metadata:FileMetadata = bincode::deserialize(&payload)?;
                    println!("file metadata {:?}",metadata);
                    let mut chunks:Vec<Option<Vec<u8>>> = vec![None;metadata.total_chunks];
                    let mut received_count = 0;
                    loop {
                        let (msg_type, payload) = read_frame(&mut tcp_stream, &mut buffer).await?;
                        match msg_type {
                            MessageType::Chunk => {
                                let chunk:Chunk = bincode::deserialize(&payload)?;
                                chunks[chunk.chunk_index] = Some(chunk.data);
                                received_count+=1;
                                if received_count == metadata.total_chunks {
                                    break;
                                }
                            },
                            _ => return Err(format!("transfer protocol violation").into())
                        };
                        
                    }
                    let mut file_bytes = Vec::new();
                    for chunk in chunks{
                        file_bytes.extend(chunk.unwrap());
                    }
                    let mut hasher = Sha256::new();
                    hasher.update(&file_bytes);
                    let computed_hash = hex::encode(hasher.finalize());
                    let success = computed_hash == metadata.file_hash;
                    tokio::fs::write("demo_file_received.txt", &file_bytes).await?;
                    let complete = TransferComplete {
                        transfer_id:metadata.transfer_id,
                        success,
                        receiver_hash: computed_hash,
                    };
                    let payload = bincode::serialize(&complete)?;
                    let frame = encode_frame(MessageType::TransferComplete, &payload);
                    send_frame(&mut tcp_stream, &frame).await?;
                },
                _ => return Err(format!("transfer protocol violation").into())
            }
        },
        _ => return Err(format!("transfer protocol violation").into())
    }
    Ok(())
}
