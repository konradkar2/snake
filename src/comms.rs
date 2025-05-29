use std::{
    array,
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

const BUFF_SIZE: usize = 4096;

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
        let mut prefix_buffer: [u8; 4] = [0; 4];
        self.read_into_buff(prefix_buffer.as_mut_slice())?;

        let frame_len = u32::from_be_bytes(prefix_buffer);

        let mut message_buffer = vec![0u8; frame_len as usize];
        let bytes = self.read_into_buff(message_buffer.as_mut_slice())?;

        if bytes != frame_len as usize {
            eprintln!(
                "Received different sized than expected in prefix: expected: {}, actual: {}",
                frame_len, bytes
            );
            return Err(CommError::Unknown);
        }

        let (decoded, _): (Message, usize) =
            bincode::decode_from_slice(message_buffer.as_slice(), self.bincode_cfg).map_err(
                |err| {
                    eprintln!(
                        "[ERROR] Failed to deserialize data (read size: {}): {}",
                        bytes, err
                    );
                    CommError::Unknown
                },
            )?;

        Ok(decoded)
    }

    fn read_into_buff(&mut self, dst: &mut [u8]) -> Result<usize, CommError> {
        let bytes = self.connection.as_ref().unwrap().read(dst).map_err(|err| {
            if err.kind() == io::ErrorKind::WouldBlock {
                CommError::WouldBlock
            } else {
                eprintln!("[ERROR] read failed {}", err);
                CommError::Unknown
            }
        })?;

        if bytes > 0 {
            Ok(bytes)
        } else {
            Err(CommError::Disconnected)
        }
    }

    pub fn send_message(&mut self, message: &Message) -> Result<(), CommError> {
        let mut buffer: [u8; BUFF_SIZE] = [0; BUFF_SIZE];

        let serialize_len =
            bincode::encode_into_slice(message, &mut buffer[4..BUFF_SIZE], self.bincode_cfg).map_err(
                |err| {
                    eprintln!("[ERROR] failed to serialize data: {}", err);
                    CommError::InvalidData
                },
            )? as u32;

        buffer[0..4].copy_from_slice(&serialize_len.to_be_bytes());

        let total_len: usize = serialize_len as usize + 4;

        self.connection
            .as_ref()
            .unwrap()
            .write_all(&buffer[..total_len])
            .map_err(|err| {
                if err.kind() == io::ErrorKind::WouldBlock {
                    CommError::WouldBlock
                } else {
                    eprintln!("[ERROR] write failed {}", err);
                    CommError::Unknown
                }
            })?;

        println!("written bytes: {}", total_len);

        Ok(())
    }
}
