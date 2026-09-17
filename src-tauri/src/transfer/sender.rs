use std::{error::Error, io::SeekFrom, net::{SocketAddr}, path::Path, sync::Arc};

use sha2::{Sha256,Digest};
use tokio::{io::{AsyncReadExt, AsyncSeekExt}, net::{TcpListener,TcpStream}};
use uuid::Uuid;
use crate::{protocol::frame::{MessageType, encode_frame}, storage::TransferStore, transfer::{CHUNK_SIZE, Chunk, FileMetadata, ResumeRequest, TransferComplete, TransferReject, TransferRequest, stream::{connect_to_peer, read_frame, send_frame}}};

pub async fn send_file(sender_id:String,addr:SocketAddr, file_path: &Path,store:Arc<TransferStore>)
    ->Result<(),Box<dyn Error>>{

    let transfer_id = Uuid::new_v4().to_string(); //sender is generating a new trasfer_id for every
    //fn call
    let file_data = tokio::fs::metadata(file_path).await?;
    let filename = file_path.file_name()
                            .ok_or("invalid file path")?
                            .to_string_lossy().to_string();
    let transfer_req = TransferRequest {
        transfer_id:transfer_id.clone(),
        sender_id,
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
            // let file_bytes = tokio::fs::read(file_path).await?; //not supposed to do this
            store.record_sent_transfer(&transfer_id, &file_path.to_string_lossy())?;
            let mut file_handle = tokio::fs::File::open(file_path).await?;
            let mut buf = vec![0u8;CHUNK_SIZE];
            let mut hasher = Sha256::new();

            loop{ //first read
                let n = file_handle.read(&mut buf).await?;
                if n == 0 {
                    break;
                }
                hasher.update(&buf[..n]);
            }
            let hash_bytes = hasher.finalize();
            let file_hash = hex::encode(hash_bytes);

            file_handle.seek(SeekFrom::Start(0)).await?;
            let file_size = file_data.len() as usize;
            let file_metadata = FileMetadata {
                transfer_id:transfer_id.clone(),
                file_hash:file_hash.clone(),
                chunk_size:CHUNK_SIZE,
                total_chunks:(file_size+CHUNK_SIZE-1)/CHUNK_SIZE
            };
            let payload = bincode::serialize(&file_metadata)?;
            let frame = encode_frame(MessageType::FileMetadata, &payload);

            send_frame(&mut tcp_stream, &frame).await?; //file metadata has been sent
            
            let mut chunk_index = 0;
            loop{ //second read
                let n = file_handle.read(&mut buf).await?;
                if n == 0 {
                    break;
                }
                let chunk = Chunk {
                    transfer_id:transfer_id.clone(),
                    chunk_index,
                    data: buf[..n].to_vec()
                };
                let payload = bincode::serialize(&chunk)?;
                let frame = encode_frame(MessageType::Chunk, &payload);
                send_frame(&mut tcp_stream, &frame).await?;
                chunk_index+=1;
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




pub async fn run_resume_listener(addr: SocketAddr, store: Arc<TransferStore>) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(addr).await?;
    loop {
        let (stream, _peer_addr) = listener.accept().await?;
        if let Err(err) = handle_resume_request(stream, Arc::clone(&store)).await {
            eprintln!("resume error: {err}");
        }
    }
}

async fn handle_resume_request(mut tcp_stream:TcpStream,store:Arc<TransferStore>)->Result<(), Box<dyn Error>>{
    let mut buffer = Vec::new();
    let (msg_type, payload) = read_frame(&mut tcp_stream, &mut buffer).await?;

    match msg_type {
        MessageType::ResumeRequest => {
            let request: ResumeRequest = bincode::deserialize(&payload)?;
            match store.get_sent_transfer_path(&request.transfer_id)? {
                Some(file_path) => {},
                None => return Err(format!("unknown transfer id /invalid request").into())
            }
            
        },
        _ => {
            return Err(format!("transfer protocol violation").into())
        }
    }
    Ok(())
}
