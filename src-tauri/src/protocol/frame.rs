//framing protocol:  length--type--payload
// length --4B
// type --1B
// 1 frame --4+1+N bytes
#[repr(u8)]
#[derive(Debug,PartialEq)]
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

#[derive(Debug,PartialEq)]
pub enum DecodeError{
    Incomplete,
    UnknownType(u8)
}

pub fn encode_frame(message_type:MessageType,payload:&[u8])->Vec<u8>{
    let mut frame = Vec::new();
    let length:[u8;4] = (payload.len() as u32).to_be_bytes();
    frame.extend_from_slice(&length);
    frame.push(message_type as u8);
    frame.extend_from_slice(payload);
    frame
}

pub fn decode_frame(buffer:&[u8])->Result<(MessageType,Vec<u8>),DecodeError>{
    if buffer.len() >=4 {
        let value = u32::from_be_bytes(buffer[..4].try_into().unwrap());
        let frame_length = 4+1+value as usize;
        if buffer.len()>=frame_length {
            let message_type = MessageType::try_from(buffer[4]);
            let payload = buffer[5..frame_length].to_vec();
            if let Some(msg_type) = message_type {
                return Ok((msg_type,payload));
            }
            return Err(DecodeError::UnknownType(buffer[4]));
        }
    }
    Err(DecodeError::Incomplete)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_encode(){
        let msg_type = MessageType::TransferRequest;
        let payload = b"hi ";

        assert_eq!(encode_frame(msg_type, payload),vec![0,0,0,3, 2, b'h', b'i',b' ']);
    }
    
    #[test]
    fn test_decode_round_trip(){
        let payload = b"hi";

        let buffer = encode_frame(MessageType::Chunk, payload);
        
        assert_eq!(decode_frame(&buffer),Ok((MessageType::Chunk,payload.to_vec())));
    }
    #[test]
    fn test_decode_empty_payload(){
        let buffer = encode_frame(MessageType::Chunk, b"");
        assert_eq!(decode_frame(&buffer),Ok((MessageType::Chunk,vec![])));
    }
    #[test]
    fn test_decode_incomplete_header(){
        let buffer = [0,0,1];
        assert_eq!(decode_frame(&buffer),Err(DecodeError::Incomplete));
    }
    #[test]
    fn test_decode_incomplete_payload(){
        let buffer = [0,0,0,3,7,b'h'];
        assert_eq!(decode_frame(&buffer),Err(DecodeError::Incomplete));
    }
    #[test]
    fn test_decode_unknown_type(){
        let buffer = [0,0,0,2,10,b'h',b'i'];
        assert_eq!(decode_frame(&buffer),Err(DecodeError::UnknownType(10)));
    }
    #[test]
    fn test_decode_two_frames(){
        let mut buffer = encode_frame(MessageType::Discover, b"hi");
        let frame2 = encode_frame(MessageType::Chunk, b"hello");
        buffer.extend_from_slice(&frame2); 

        assert_eq!(decode_frame(&buffer),Ok((MessageType::Discover,b"hi".to_vec())));
        let buffer = buffer[7..].to_vec();
        assert_eq!(decode_frame(&buffer),Ok((MessageType::Chunk,b"hello".to_vec())));
    }
}
