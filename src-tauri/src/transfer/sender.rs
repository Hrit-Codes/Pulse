use std::{collections::HashSet, error::Error, io::SeekFrom, net::SocketAddr, path::Path, sync::Arc};

use bytes::BytesMut;
use sha2::{Sha256,Digest};
use tauri::AppHandle;
use tokio::{io::{AsyncReadExt,AsyncWriteExt, AsyncSeekExt}, net::{TcpListener,TcpStream}};
use uuid::Uuid;
use crate::{protocol::frame::{MessageType, encode_frame}, storage::TransferStore, transfer::{CHUNK_SIZE, Chunk,
    FileHash, FileMetadata, RequestPin, ResumeRequest, TransferComplete, TransferReject, TransferRequest,
    stream::{connect_to_peer, read_frame,
        send_frame}}, util::{check_equal, emit_error, emit_message, generate_pin}};

async fn send_chunk(
    tcp_stream: &mut TcpStream,
    chunk_index: usize,
    data: Vec<u8>,
) -> Result<(), Box<dyn Error>> {
    let chunk = Chunk { chunk_index, data };
    let payload = bincode::serialize(&chunk)?;
    let frame = encode_frame(MessageType::Chunk, &payload);
    send_frame(tcp_stream, &frame).await?;
    Ok(())
}

/// Sends the computed file hash to the peer and waits for their `TransferComplete`
/// response, logging whether the transfer was verified successfully.
async fn send_file_hash_and_await_completion(
    tcp_stream: &mut TcpStream,
    buffer: &mut BytesMut,
    file_hash: FileHash,
    app:Option<&AppHandle>
) -> Result<(), Box<dyn Error>> {
    let payload = bincode::serialize(&file_hash)?;
    let frame = encode_frame(MessageType::FileHash, &payload);
    send_frame(tcp_stream, &frame).await?; //sent the file hash after all the chunks

    let (response_type, response_payload) = read_frame(tcp_stream, buffer).await?;
    match response_type {
        MessageType::TransferComplete => {
            let complete: TransferComplete = bincode::deserialize(&response_payload)?;
            if complete.success && complete.receiver_hash == file_hash.0 {
                emit_message(app, "transfer_complete", "transfer verified successfully".to_string()).await;
            } else {
                return Err("Transfer failed or hash mismatch".into())
            }
        }
        _ => {
            return Err(format!("Unexpected response: {:?}", response_type).into())
        }
    }
    Ok(())
}

pub async fn request_to_send_file(sender_id: String,addr: SocketAddr,file_path: &Path,store: Arc<TransferStore>,
    app:Option<&AppHandle>)->
    Result<(),Box<dyn Error>>{
    let mut tcp_stream = connect_to_peer(addr).await?;
    //authorization
    let pin = generate_pin();
    emit_message(app, "pin", pin.clone()).await;
    let mut buffer = BytesMut::new();
    let (response_type,payload) = read_frame(&mut tcp_stream, &mut buffer).await?;
    match response_type {
        MessageType::RequestPin => {
            let new_pin:RequestPin = bincode::deserialize(&payload)?;
            if !check_equal(pin, new_pin.0){
                let transfer_reject = TransferReject{
                    reason:"file transfer rejected: incorrect pin".to_string()
                };
                let payload = bincode::serialize(&transfer_reject)?;
                let frame = encode_frame(MessageType::TransferReject, &payload);
                send_frame(&mut tcp_stream, &frame).await?;
                return Err(transfer_reject.reason.into())
            }
        },
        _ => return Err("transfer protocol violation".into())
    }
    
    //authorization successful
    send_file(&mut tcp_stream,buffer, sender_id, file_path, store,app).await
}

async fn send_file(
    mut tcp_stream: &mut TcpStream,
    mut buffer: BytesMut,
    sender_id: String,
    file_path: &Path,
    store: Arc<TransferStore>,
    app: Option<&AppHandle>
) -> Result<(), Box<dyn Error>> {
    let transfer_id = Uuid::new_v4().to_string(); //sender is generating a new trasfer_id for every
    //fn call
    let file_data = tokio::fs::metadata(file_path).await?;
    let filename = file_path
        .file_name()
        .ok_or("invalid file path")?
        .to_string_lossy()
        .to_string();
    let transfer_req = TransferRequest {
        transfer_id: transfer_id.clone(),
        sender_id,
        filename,
        file_size: file_data.len(),
    };

    let payload = bincode::serialize(&transfer_req)?;
    let frame = encode_frame(MessageType::TransferRequest, &payload);

    // let mut tcp_stream = connect_to_peer(addr).await?;
    send_frame(&mut tcp_stream, &frame).await?;

    let (response_type, response_payload) = read_frame(&mut tcp_stream, &mut buffer).await?;

    match response_type {
        MessageType::TransferReject => {
            let reject_msg: TransferReject = bincode::deserialize(&response_payload)?;
            return Err(format!("Transfer rejected: {}", reject_msg.reason).into());
        }
        MessageType::TransferAccept => {
            // let file_bytes = tokio::fs::read(file_path).await?; //not supposed to do this
            store.record_sent_transfer(&transfer_id, &file_path.to_string_lossy())?;

            let file_size = file_data.len() as usize;
            let file_metadata = FileMetadata {
                transfer_id: transfer_id.clone(),
                // file_hash:file_hash.clone(),
                chunk_size: CHUNK_SIZE,
                total_chunks: (file_size + CHUNK_SIZE - 1) / CHUNK_SIZE,
            };
            let payload = bincode::serialize(&file_metadata)?;
            let frame = encode_frame(MessageType::FileMetadata, &payload);

            send_frame(&mut tcp_stream, &frame).await?; //file metadata has been sent

            let mut file_handle = tokio::fs::File::open(file_path).await?;
            file_handle.seek(SeekFrom::Start(0)).await?;

            let mut buf = vec![0u8; CHUNK_SIZE];
            let mut hasher = Sha256::new();
            let mut chunk_index = 0;

            loop {
                //file read
                let n = file_handle.read(&mut buf).await?;
                if n == 0 {
                    break;
                }
                hasher.update(&buf[..n]);
                send_chunk(&mut tcp_stream, chunk_index, buf[..n].to_vec()).await?;
                // --- replacement: zero-alloc direct write ---
                // let payload_len = (8 + n) as u32;
                // tcp_stream.write_all(&payload_len.to_be_bytes()).await?;
                // tcp_stream.write_all(&[MessageType::Chunk as u8]).await?;
                // tcp_stream.write_all(&chunk_index.to_le_bytes()).await?;
                // tcp_stream.write_all(&buf[..n]).await?;
                // --- end replacement ---
                chunk_index += 1;
            }

            let hash_bytes = hasher.finalize();
            let file_hash = FileHash(hex::encode(hash_bytes));

            send_file_hash_and_await_completion(&mut tcp_stream, &mut buffer, file_hash,app).await?;
        }
        _ => {
            return Err(format!("unexpected response: {:?}", response_type).into())
        }
    }

    Ok(())
}

pub async fn run_resume_listener(addr: SocketAddr, store: Arc<TransferStore>,
    app:Option<&AppHandle>) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(addr).await?;
    loop {
        let (stream, _peer_addr) = listener.accept().await?;
        if let Err(err) = handle_resume_request(stream, Arc::clone(&store),app).await {
            emit_error(app, err.to_string()).await;
        }
    }
}

async fn handle_resume_request(mut tcp_stream: TcpStream, store: Arc<TransferStore>,app:Option<&AppHandle>
    ) -> Result<(), Box<dyn Error>> {
    let mut buffer = BytesMut::new();
    let (msg_type, payload) = read_frame(&mut tcp_stream, &mut buffer).await?;

    match msg_type {
        MessageType::ResumeRequest => {
            let request: ResumeRequest = bincode::deserialize(&payload)?;
            match store.get_sent_transfer_path(&request.transfer_id)? {
                Some(file_path) => {
                    let (metadata_result, file_result) = tokio::join!(
                        tokio::fs::metadata(&file_path),
                        tokio::fs::File::open(&file_path),
                    );

                    let file_size = metadata_result?.len() as usize;
                    let mut file_handle = file_result?;
                    let total_chunks = (file_size + CHUNK_SIZE - 1) / CHUNK_SIZE;
                    let mut buf = vec![0u8; CHUNK_SIZE];
                    let received_set: HashSet<usize> = request.received_chunks.iter().copied().collect();
                    for i in 0..total_chunks {
                        if received_set.contains(&i) {
                            continue;
                        }
                        let offset = (i * CHUNK_SIZE) as u64;
                        file_handle.seek(SeekFrom::Start(offset)).await?;
                        let n = file_handle.read(&mut buf).await?;

                        send_chunk(&mut tcp_stream, i, buf[..n].to_vec()).await?;
                    }
                    file_handle.flush().await?;
                    file_handle.seek(SeekFrom::Start(0)).await?;

                    let mut hasher = Sha256::new();
                    loop {
                        let n = file_handle.read(&mut buf).await?;
                        if n == 0 {
                            break;
                        }
                        hasher.update(&buf[..n]);
                    }

                    let file_hash = FileHash(hex::encode(hasher.finalize()));

                    send_file_hash_and_await_completion(&mut tcp_stream, &mut buffer, file_hash,app).await?;
                }
                None => return Err("unknown transfer id /invalid request".into()),
            }
        }
        _ => return Err("transfer protocol violation".into()),
    }
    Ok(())
}
