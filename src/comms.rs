use std::{
    io::{self, Read, Write},
    net::TcpStream,
};

use crate::{ifc::Message, snake_cfg::SERVER_ADDRESS};

pub struct Comms {
    pub bincode_cfg: bincode::config::Configuration,
    pub connection: Option<TcpStream>,
}

// Define our error types. These may be customized for our error handling cases.
// Now we will be able to write our own errors, defer to an underlying error
// implementation, or do something in between.
#[derive(Debug, Clone)]
pub enum CommError {
    Unknown,
    Disconnected,
    InvalidData,
    WouldBlock,
}

#[derive(Debug)]
pub enum ClientError {
    ConnectionError,
    Unknown(String),
}

impl Comms {
    pub fn connect(&mut self, server_ip: &str) -> Result<(), io::Error> {
        let stream = TcpStream::connect(server_ip).inspect_err(|err| {
            eprintln!(
                "[ERROR] Failed to connect to the server ({}): {}",
                server_ip, err
            );
        })?;

        // stream.set_nonblocking(true).inspect_err(|err| {
        //     eprintln!("[ERROR] Failed to set non blocking: {}", err);
        // })?;

        self.connection = Some(stream);
        Ok(())
    }

    pub fn set_nonblocking(&mut self) {
        self.connection
            .as_ref()
            .unwrap()
            .set_nonblocking(true)
            .expect("failed to set non blocking on a socket");
    }

    pub fn receive_message(&mut self) -> Result<Message, CommError> {
        let mut buffer = [0; 4096];

        let bytes = self
            .connection
            .as_ref()
            .unwrap()
            .read(&mut buffer)
            .map_err(|err| {
                if err.kind() == io::ErrorKind::WouldBlock {
                    CommError::WouldBlock
                } else {
                    eprintln!("[ERROR] read failed {}", err);
                    CommError::Unknown
                }
            })?;

        if bytes == 0 {
            eprintln!("[ERROR] lost connection");
            Err(CommError::Disconnected)?
        }

        let (decoded, _): (Message, usize) =
            bincode::decode_from_slice(&buffer[..], self.bincode_cfg).map_err(|err| {
                eprintln!("[ERROR] Failed to deserialize data (read size: {}): {}", bytes, err);
                CommError::Unknown
            })?;

        Ok(decoded)
    }

    pub fn send_message(&mut self, message: &Message) -> Result<(), CommError> {
        let encoded: Vec<u8> =
            bincode::encode_to_vec(message, self.bincode_cfg).map_err(|err| {
                eprintln!("[ERROR] failed to serialize data: {}", err);
                CommError::InvalidData
            })?;

        let bytes_sent = self
            .connection
            .as_ref()
            .unwrap()
            .write(&encoded)
            .map_err(|err| {
                if err.kind() == io::ErrorKind::WouldBlock {
                    CommError::WouldBlock
                } else {
                    eprintln!("[ERROR] write failed {}", err);
                    CommError::Unknown
                }
            })?;

        if bytes_sent == 0 {
            eprintln!("[ERROR] lost connection");
            Err(CommError::Disconnected)?
        }

        Ok(())
    }
}
