//framing protocol:  length--type--payload
// length --4B
// type --1B
// 1 frame --4+1+N bytes
#[repr(u8)]
pub enum MessageType {
    Discover = 0,
    DiscoverResponse = 1,
    TransferRequest = 2,
    TransferAccept = 3,
    TransferReject = 4,
    TransferComplete = 5,
    FileMetadata = 6,
    Chunk = 7,
    ChunkAck = 8,
    SyncUpdate = 9,
}
impl MessageType {
    fn try_from(value:u8) -> Option<Self>{
        match value {
            0 => Some(MessageType::Discover),
            1 => Some(MessageType::DiscoverResponse),
            2 => Some(MessageType::TransferRequest),
            3 => Some(MessageType::TransferAccept),
            4 => Some(MessageType::TransferReject),
            5 => Some(MessageType::TransferComplete),
            6 => Some(MessageType::FileMetadata),
            7 => Some(MessageType::Chunk),
            8 => Some(MessageType::ChunkAck),
            9 => Some(MessageType::SyncUpdate),
            _ => None
        }

    }
}

pub fn encode_frame(message_type:MessageType,payload:&[u8])->Vec<u8>{
    let mut frame = Vec::new();
    let length:[u8;4] = (payload.len() as u32).to_be_bytes();
    frame.extend_from_slice(&length);
    frame.push(message_type as u8);
    frame.extend_from_slice(payload);
    frame
}

pub fn decode_frame(buffer:&mut Vec<u8>)->Option<(MessageType,&[u8])>{
    if buffer.len() >=4 {
        let value = u32::from_be_bytes(buffer[..4].try_into().unwrap());
        let frame_length = 4+1+value as usize;
        if buffer.len()>=frame_length {
            let message_type = MessageType::try_from(buffer[4]);
            let payload = &buffer[5..frame_length];
            return Some((message_type.unwrap(),payload))
        }
    }
    None
}
