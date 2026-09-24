use std::{io, net::SocketAddr,error::Error};

use tokio::{io::{AsyncWriteExt,AsyncReadExt}, net::TcpStream};

use crate::protocol::frame::{DecodeError, MessageType, decode_frame};
use bytes::BytesMut;


pub async fn connect_to_peer(addr:SocketAddr)->Result<TcpStream,io::Error>{
    TcpStream::connect(addr).await
}

pub async fn send_frame(tcp_stream:&mut TcpStream,bytes:&[u8])->Result<(),io::Error>{
    tcp_stream.write_all(bytes).await
}

// pub async fn read_frame(tcp_stream:&mut TcpStream,buffer:&mut Vec<u8>)->Result<(MessageType,Vec<u8>),Box<dyn Error>> {
//     let mut scratch = [0u8; 65536]; //64KB buffer
//     loop {
//         
//         match decode_frame(&buffer) {
//             Ok((msg_type,payload)) => {
//                 let consumed = 5 + payload.len();
//                 let _ = buffer.drain(..consumed); //might wanna use queue instead
//                 return Ok((msg_type,payload));
//             },
//             Err(DecodeError::Incomplete) => (),
//             Err(DecodeError::UnknownType(byte)) => return Err(format!("unknown message type: {byte}").into())
//
//         };
//
//         let bytes_read = tcp_stream.read(&mut scratch).await?;
//         if bytes_read == 0 {
//             return Err("connection closed before a full frame arrived".into());
//         }
//         buffer.extend_from_slice(&scratch[..bytes_read]);
//
//     }
// }
pub async fn read_frame(tcp_stream: &mut TcpStream, buffer: &mut BytesMut) ->
Result<(MessageType, Vec<u8>), Box<dyn Error>> {
    loop {
        match decode_frame(buffer) {
            Ok((msg_type, payload)) => {
                let consumed = 5 + payload.len();
                let _ = buffer.split_to(consumed);  // O(1) — just advances the start pointer
                return Ok((msg_type, payload));
            },
            Err(DecodeError::Incomplete) => (),
            Err(DecodeError::UnknownType(byte)) => return Err(format!("unknown message type: {byte}").into())
        };

        buffer.reserve(65536);
        let n = tcp_stream.read_buf(buffer).await?;
        if n == 0 {
            return Err("connection closed before a full frame arrived".into());
        }
    }
}
