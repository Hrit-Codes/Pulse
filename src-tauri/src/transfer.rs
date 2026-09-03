use serde::{Deserialize, Serialize};

pub mod sender;
pub mod receiver;
pub mod stream;

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
    pub chunk_size: u32,
    pub total_chunks: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Chunk {
    pub transfer_id: String,
    pub chunk_index: u32,
    pub data: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChunkAck {
    pub transfer_id: String,
    pub chunk_index: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransferComplete {
    pub transfer_id: String,
    pub success: bool,
    pub receiver_hash: String,
}
