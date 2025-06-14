use std::{
    array,
    io::{self, Read, Write},
    net::TcpStream,
};

use bincode::config;

use crate::{ifc::Message, snake_cfg::SERVER_ADDRESS};

pub struct Comms {
    pub bincode_cfg: bincode::config::Configuration,
    pub connection: Option<TcpStream>,
    recv_buffer: [u8; BUFF_SIZE],
    recv_buffer_pointer: usize,
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
    WaitingForMoreData,
}

#[derive(Debug)]
pub enum ClientError {
    ConnectionError,
    Unknown(String),
}

const BUFF_SIZE: usize = 4096;
const PREFIX_SIZE: usize = 4;

fn read_into_buff(connection: &mut TcpStream, dst: &mut [u8]) -> Result<usize, CommError> {
    let bytes = connection.read(dst).map_err(|err| {
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
        eprintln!("Received 0 bytes, disconnecting");
        Err(CommError::Disconnected)
    }
}

fn get_last_message(buff: &[u8]) -> (Vec<u8>, Option<Vec<u8>>) {
    let mut ret: Vec<u8> = Vec::new();
    let mut remainder: Option<Vec<u8>> = None;

    let mut buffer_pointer = 0;
    let mut bytes_left = buff.len();
    while bytes_left > PREFIX_SIZE {
        let mut prefix_buffer: [u8; PREFIX_SIZE] = [0; PREFIX_SIZE];
        prefix_buffer.copy_from_slice(&buff[buffer_pointer..buffer_pointer + PREFIX_SIZE]);
        buffer_pointer += PREFIX_SIZE;
        bytes_left -= PREFIX_SIZE;

        let frame_len = u32::from_be_bytes(prefix_buffer) as usize;
    
        if frame_len > bytes_left {
            buffer_pointer -= PREFIX_SIZE;
            remainder = Some(buff[buffer_pointer..].to_vec());
            break;
        }
        ret = buff[buffer_pointer..buffer_pointer + frame_len as usize].to_vec();

        buffer_pointer += frame_len;
        bytes_left -= frame_len;
    }

    (ret, remainder)
}

impl Comms {
    pub fn new(connection: Option<TcpStream>) -> Self {
        Self {
            bincode_cfg: config::standard(),
            connection,
            recv_buffer: [0; BUFF_SIZE],
            recv_buffer_pointer: 0,
        }
    }
    pub fn connect(&mut self, server_ip: &str) -> Result<(), io::Error> {
        let stream = TcpStream::connect(server_ip).inspect_err(|err| {
            eprintln!(
                "[ERROR] Failed to connect to the server ({}): {}",
                server_ip, err
            );
        })?;

        stream.set_nonblocking(true).inspect_err(|err| {
            eprintln!("[ERROR] Failed to set non blocking: {}", err);
        })?;

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
        let recv_buff = self.recv_buffer.as_mut_slice();
        let recv_buff_p = self.recv_buffer_pointer;
        let read_size = read_into_buff(self.connection.as_mut().unwrap(), &mut recv_buff[recv_buff_p..])?;

        let (last_msg, remainder) = get_last_message(&mut recv_buff[..recv_buff_p + read_size]);

        if let Some(remainder) = remainder {
            println!("got remainder...");
            let target = &mut self.recv_buffer[..remainder.len()];
            target.copy_from_slice(remainder.as_slice());
            self.recv_buffer_pointer += remainder.len();

            if last_msg.len() == 0 {
                return Err(CommError::WaitingForMoreData);
            }
        } else {
            self.recv_buffer_pointer = 0;
        }

    

        let (decoded, _): (Message, usize) =
            bincode::decode_from_slice(last_msg.as_slice(), self.bincode_cfg).map_err(
                |err| {
                    eprintln!(
                        "[ERROR] Failed to deserialize data (read size: {}): {}",
                        last_msg.len(), err
                    );
                    CommError::Unknown
                },
            )?;

        Ok(decoded)
    }

    pub fn send_message(&mut self, message: &Message) -> Result<(), CommError> {
        let mut buffer: [u8; BUFF_SIZE] = [0; BUFF_SIZE];

        let serialize_len =
            bincode::encode_into_slice(message, &mut buffer[4..BUFF_SIZE], self.bincode_cfg)
                .map_err(|err| {
                    eprintln!("[ERROR] failed to serialize data: {}", err);
                    CommError::InvalidData
                })? as u32;

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
