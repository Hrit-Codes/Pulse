use std::{collections::HashMap, error::Error, io::SeekFrom, net::{IpAddr, SocketAddr}, sync::Arc};

use bytes::BytesMut;
use sha2::{Sha256,Digest};
use tokio::{io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt}, net::{TcpListener, TcpStream}, sync::Mutex};

use crate::{discover::DeviceInfo, protocol::frame::{MessageType, encode_frame}, 
    storage::{TransferStatus, TransferStore}, transfer::{CHUNK_SIZE, Chunk, FileHash,
        FileMetadata, ResumeRequest, TransferAccept, TransferComplete, TransferRequest,
        stream::{connect_to_peer, read_frame, send_frame}}};

pub async fn receive_file(addr:SocketAddr)-> Result<(), Box<dyn Error>>{
    let listener = TcpListener::bind(addr).await?;
    let store = Arc::new(TransferStore::new()?);
    loop {
        let (stream,_peer_addr) = listener.accept().await?;
        if let Err(err) = handle_connection(stream,Arc::clone(&store)).await {
            eprintln!("transfer error: {}",err);
        }
    }
}

const PENDING_CHUNKS:usize = 32;

async fn receive_chunks_and_finalize(
    tcp_stream: &mut TcpStream,
    buffer: &mut BytesMut,
    store: &Arc<TransferStore>,
    transfer_id: &str,
    filename: &str,
    chunk_size: usize,
    total_chunks: usize,
    mut received_count: usize,
    file: &mut tokio::fs::File,
    output_path: &std::path::Path,
) -> Result<(), Box<dyn Error>> {
    let mut pending_chunks = Vec::with_capacity(PENDING_CHUNKS);

    loop {
        let result = read_frame(tcp_stream, buffer).await;
        let (msg_type, payload) = match result {
            Ok((msg, pay)) => (msg, pay),
            Err(e) => {
                if !pending_chunks.is_empty() {
                    store.mark_chunks_received(transfer_id, &pending_chunks)?;
                    pending_chunks.clear();
                }
                return Err(e);
            }
        };

        match msg_type {
            MessageType::Chunk => {
                let chunk: Chunk = bincode::deserialize(&payload)?;
                let offset = (chunk.chunk_index * chunk_size) as u64;
                file.seek(SeekFrom::Start(offset)).await?;
                file.write_all(&chunk.data).await?;

                pending_chunks.push(chunk.chunk_index);
                received_count += 1;

                if pending_chunks.len() == PENDING_CHUNKS {
                    store.mark_chunks_received(transfer_id, &pending_chunks)?;
                    pending_chunks.clear();
                }

                if received_count == total_chunks {
                    break;
                }
            }
            _ => return Err("transfer protocol violation".into()),
        }
    }

    if !pending_chunks.is_empty() {
        store.mark_chunks_received(transfer_id, &pending_chunks)?;
        pending_chunks.clear();
    }

    let (msg_type, payload) = read_frame(tcp_stream, buffer).await?;
    let file_hash: FileHash = match msg_type {
        MessageType::FileHash => bincode::deserialize(&payload)?,
        _ => return Err("transfer protocol violation".into()),
    };

    let mut buf = vec![0u8; CHUNK_SIZE]; // heap buffer, fine for large chunk sizes
    file.flush().await?;
    file.seek(SeekFrom::Start(0)).await?;

    let mut hasher = Sha256::new();
    loop {
        let n = file.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    let computed_hash = hex::encode(hasher.finalize());
    let success = computed_hash == file_hash.0;

    if success {
        let download_dir = dirs::download_dir().ok_or("could not resolve downloads directory")?;
        let final_path = download_dir.join(filename);
        tokio::fs::rename(output_path, final_path).await?;
        store.update_status(transfer_id, TransferStatus::Complete)?;
    } else {
        store.update_status(transfer_id, TransferStatus::Failed)?;
        tokio::fs::remove_file(output_path).await?;
    }

    let complete = TransferComplete {
        transfer_id: transfer_id.to_string(),
        success,
        receiver_hash: computed_hash,
    };
    let payload = bincode::serialize(&complete)?;
    let frame = encode_frame(MessageType::TransferComplete, &payload);
    send_frame(tcp_stream, &frame).await?;

    Ok(())
}

async fn handle_connection(mut tcp_stream: TcpStream, store: Arc<TransferStore>) -> Result<(), Box<dyn Error>> {
    //might require Arc<Mutex<>> later for concurrent transfers
    let mut buffer = BytesMut::new();
    let (msg_type, payload) = read_frame(&mut tcp_stream, &mut buffer).await?;
    match msg_type {
        MessageType::TransferRequest => {
            let request: TransferRequest = bincode::deserialize(&payload)?;
            println!("Incoming transfer: {} ({} bytes)", request.filename, request.file_size);
            //going to accept it for now. rejection will be implemented later
            let transfer_accept: TransferAccept = TransferAccept {
                transfer_id: request.transfer_id,
            };
            let payload = bincode::serialize(&transfer_accept)?;
            let frame = encode_frame(MessageType::TransferAccept, &payload);
            send_frame(&mut tcp_stream, &frame).await?;

            let (msg_type, payload) = read_frame(&mut tcp_stream, &mut buffer).await?;
            match msg_type {
                MessageType::FileMetadata => {
                    let metadata: FileMetadata = bincode::deserialize(&payload)?;
                    println!("file metadata {:?}", metadata);

                    store.create_transfer(&metadata.transfer_id,&request.filename,request.file_size,
                        metadata.total_chunks,metadata.chunk_size,&request.sender_id)?;

                    let chunk_dir = store.get_chunk_dir()?;

                    //output path is created at base_location/chunks/
                    let output_path = chunk_dir.join(format!("{}.bin", metadata.transfer_id));
                    let mut file = tokio::fs::OpenOptions::new()
                        .create(true)
                        .read(true)
                        .write(true)
                        .open(&output_path)
                        .await?;
                    file.set_len(request.file_size).await?;

                    receive_chunks_and_finalize(
                        &mut tcp_stream,
                        &mut buffer,
                        &store,
                        &metadata.transfer_id,
                        &request.filename,
                        metadata.chunk_size,
                        metadata.total_chunks,
                        0,
                        &mut file,
                        &output_path,
                    )
                    .await?;
                }
                _ => return Err("transfer protocol violation".into()),
            }
        }
        _ => return Err("transfer protocol violation".into()),
    }
    Ok(())
}

pub async fn resume_transfer(
    transfer_id: String,
    store: Arc<TransferStore>,
    devices: Arc<Mutex<HashMap<String, (DeviceInfo, IpAddr)>>>,
) -> Result<(), Box<dyn Error>> {
    let pending = store.get_in_progress_transfers()?;
    let (_, sender_id, filename, _file_size, total_chunks) = pending
        .into_iter()
        .find(|(id, _, _, _, _)| id == &transfer_id)
        .ok_or("transfer not found")?;

    let sender_addr;
    {
        let device_guard = devices.lock().await;
        let (device_info, ip) = device_guard.get(&sender_id).ok_or("Device is not discoverable")?;
        sender_addr = SocketAddr::new(*ip, device_info.port);
    }
    let received_chunks = store.get_received_chunks(&transfer_id)?;
    let received_count = received_chunks.len();

    let mut tcp_stream = connect_to_peer(sender_addr).await?;
    let resume_req = ResumeRequest {
        transfer_id: transfer_id.clone(),
        received_chunks,
    };
    let payload = bincode::serialize(&resume_req)?;
    let frame = encode_frame(MessageType::ResumeRequest, &payload);
    send_frame(&mut tcp_stream, &frame).await?;

    //start accepting chunks
    let mut buffer = BytesMut::new();

    let chunk_dir = store.get_chunk_dir()?;
    let output_path = chunk_dir.join(format!("{}.bin", transfer_id));

    let mut file = tokio::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&output_path)
        .await?;

    receive_chunks_and_finalize(
        &mut tcp_stream,
        &mut buffer,
        &store,
        &transfer_id,
        &filename,
        CHUNK_SIZE,
        total_chunks as usize,
        received_count,
        &mut file,
        &output_path,
    )
    .await?;

    Ok(())
}
