use std::{collections::HashSet, error::Error, io::SeekFrom, net::SocketAddr, path::Path, sync::Arc, time::{Duration, Instant}};

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
) -> Result<(), Box<dyn Error + Send + Sync>> {
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
    app: Option<&AppHandle>
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let payload = bincode::serialize(&file_hash)?;
    let frame = encode_frame(MessageType::FileHash, &payload);
    send_frame(tcp_stream, &frame).await?; //sent the file hash after all the chunks

    let (response_type, response_payload) = read_frame(tcp_stream, buffer).await?;
    match response_type {
        MessageType::TransferComplete => {
            let complete: TransferComplete = bincode::deserialize(&response_payload)?;
            if complete.success && complete.receiver_hash == file_hash.0 {
                emit_message(app, "transfer_complete_sender", "Transfer verified successfully".to_string()).await;
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

pub async fn request_to_send_file(sender_id: String, addr: SocketAddr, file_path: &Path, store: Arc<TransferStore>,
    app: Option<&AppHandle>) ->
    Result<(), Box<dyn Error + Send + Sync>> {
    let mut tcp_stream = connect_to_peer(addr).await?;
    //authorization
    let pin = generate_pin();
    emit_message(app, "pin", pin.clone()).await;
    let mut buffer = BytesMut::new();
    let (response_type, payload) = read_frame(&mut tcp_stream, &mut buffer).await?;
    match response_type {
        MessageType::RequestPin => {
            let new_pin: RequestPin = bincode::deserialize(&payload)?;
            if !check_equal(pin, new_pin.0) {
                let transfer_reject = TransferReject {
                    reason: "file transfer rejected: incorrect pin".to_string()
                };
                let payload = bincode::serialize(&transfer_reject)?;
                let frame = encode_frame(MessageType::TransferReject, &payload);
                send_frame(&mut tcp_stream, &frame).await?;
                return Err(transfer_reject.reason.into())
            }
        },
        _ => return Err("transfer protocol violation".into())
    }

    //authorization successful: tell the UI to close the PIN dialog and show "sending"
    let filename = file_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    emit_message(app, "transfer_started", filename).await;

    send_file(&mut tcp_stream, buffer, sender_id, file_path, store, app).await
}

async fn send_file(
    tcp_stream: &mut TcpStream,
    mut buffer: BytesMut,
    sender_id: String,
    file_path: &Path,
    store: Arc<TransferStore>,
    app: Option<&AppHandle>
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let transfer_id = Uuid::new_v4().to_string(); //sender generates a new transfer_id per call
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

    send_frame(tcp_stream, &frame).await?;

    let (response_type, response_payload) = read_frame(tcp_stream, &mut buffer).await?;

    match response_type {
        MessageType::TransferReject => {
            let reject_msg: TransferReject = bincode::deserialize(&response_payload)?;
            return Err(format!("Transfer rejected: {}", reject_msg.reason).into());
        }
        MessageType::TransferAccept => {
            store.record_sent_transfer(&transfer_id, &file_path.to_string_lossy())?;

            let file_size = file_data.len() as usize;
            let file_metadata = FileMetadata {
                transfer_id: transfer_id.clone(),
                chunk_size: CHUNK_SIZE,
                total_chunks: (file_size + CHUNK_SIZE - 1) / CHUNK_SIZE,
            };
            let payload = bincode::serialize(&file_metadata)?;
            let frame = encode_frame(MessageType::FileMetadata, &payload);

            send_frame(tcp_stream, &frame).await?; //file metadata has been sent

            let mut file_handle = tokio::fs::File::open(file_path).await?;
            file_handle.seek(SeekFrom::Start(0)).await?;

            let mut buf = vec![0u8; CHUNK_SIZE];
            let mut hasher = Sha256::new();
            let mut chunk_index = 0;
            let total_bytes = file_size as u64;
            let mut sent_bytes: u64 = 0;
            let mut last_emit = Instant::now();

            loop {
                let n = file_handle.read(&mut buf).await?;
                if n == 0 {
                    break;
                }
                hasher.update(&buf[..n]);
                send_chunk(tcp_stream, chunk_index, buf[..n].to_vec()).await?;
                chunk_index += 1;
                sent_bytes += n as u64;

                // throttle so we don't flood the event channel
                if last_emit.elapsed() >= Duration::from_millis(100) {
                    emit_message(app, "transfer_progress", format!("{},{}", sent_bytes, total_bytes)).await;
                    last_emit = Instant::now();
                }
            }

            // final 100% tick
            emit_message(app, "transfer_progress", format!("{},{}", sent_bytes, total_bytes)).await;

            let file_hash = FileHash(hex::encode(hasher.finalize()));

            send_file_hash_and_await_completion(tcp_stream, &mut buffer, file_hash, app).await?;
        }
        _ => {
            return Err(format!("unexpected response: {:?}", response_type).into())
        }
    }

    Ok(())
}

pub async fn run_resume_listener(addr: SocketAddr, store: Arc<TransferStore>,
    app: Option<&AppHandle>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let listener = TcpListener::bind(addr).await?;
    loop {
        let (mut stream, _peer_addr) = listener.accept().await?;
        if let Err(err) = handle_resume_request(&mut stream, Arc::clone(&store), app).await {
            let reason = err.to_string();
            // tell the receiver why, instead of silently closing the socket
            if let Ok(payload) = bincode::serialize(&TransferReject { reason: reason.clone() }) {
                let frame = encode_frame(MessageType::TransferReject, &payload);
                let _ = send_frame(&mut stream, &frame).await;
            }
            emit_error(app, reason).await;
        }
    }
}

async fn handle_resume_request(tcp_stream: &mut TcpStream, store: Arc<TransferStore>, app: Option<&AppHandle>
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut buffer = BytesMut::new();
    let (msg_type, payload) = read_frame(tcp_stream, &mut buffer).await?;

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

                        send_chunk(tcp_stream, i, buf[..n].to_vec()).await?;
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

                    send_file_hash_and_await_completion(tcp_stream, &mut buffer, file_hash, app).await?;
                }
                None => return Err("unknown transfer id /invalid request".into()),
            }
        }
        _ => return Err("transfer protocol violation".into()),
    }
    Ok(())
}