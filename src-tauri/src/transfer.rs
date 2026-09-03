use serde::{Deserialize, Serialize};

pub mod sender;
pub mod receiver;
pub mod stream;

pub const CHUNK_SIZE:usize = 1024*1024;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransferRequest {
    pub transfer_id: String,
    pub filename: String,
    pub file_size: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransferAccept {
    pub transfer_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransferReject {
    pub transfer_id: String,
    pub reason: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileMetadata {
    pub transfer_id: String,
    pub file_hash: String,
    pub chunk_size: usize,
    pub total_chunks: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Chunk {
    pub transfer_id: String,
    pub chunk_index: usize,
    pub data: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChunkAck {
    pub transfer_id: String,
    pub chunk_index: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransferComplete {
    pub transfer_id: String,
    pub success: bool,
    pub receiver_hash: String,
}
