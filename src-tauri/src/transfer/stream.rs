use std::{io, net::SocketAddr,error::Error};

use tokio::{io::{AsyncWriteExt,AsyncReadExt}, net::TcpStream};

use crate::protocol::frame::{DecodeError, MessageType, decode_frame};

pub async fn connect_to_peer(addr:SocketAddr)->Result<TcpStream,io::Error>{
    TcpStream::connect(addr).await
}

pub async fn send_frame(tcp_stream:&mut TcpStream,bytes:&[u8])->Result<(),io::Error>{
    tcp_stream.write_all(bytes).await
}

pub async fn read_frame(tcp_stream:&mut TcpStream,buffer:&mut Vec<u8>)->Result<(MessageType,Vec<u8>),Box<dyn Error>> {
    let mut scratch = [0u8; 1024];
    loop {
        
        match decode_frame(&buffer) {
            Ok((msg_type,payload)) => {
                let consumed = 5 + payload.len();
                buffer.drain(0..consumed);
                return Ok((msg_type,payload));
            },
            Err(DecodeError::Incomplete) => (),
            Err(DecodeError::UnknownType(byte)) => return Err(format!("unknown message type: {byte}").into())

        };

        let bytes_read = tcp_stream.read(&mut scratch).await?;
        if bytes_read == 0 {
            return Err("connection closed before a full frame arrived".into());
        }
        buffer.extend_from_slice(&scratch[..bytes_read]);

    }
}
