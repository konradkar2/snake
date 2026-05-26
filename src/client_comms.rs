use crate::Comms;
use crate::comms::CommError;
use crate::game::game_core::GameCore;
use crate::ifc::*;
use crate::snake_cfg::*;


#[derive(Debug)]
pub enum ClientError {
    ConnectionError,
    Unknown(String),
}

#[derive(Debug)]
pub struct ClientSettings {
    pub nickname: String,
    pub server_ip: String,
}

impl ClientSettings {
    pub fn print(self: &Self) {
        println!(
            "[Info]: server_address: {}, nickname '{}'",
            self.server_ip, self.nickname
        );
    }
}

pub struct ClientComms {
    comms: Comms,
    settings: ClientSettings,
    update_count: usize,
}

impl ClientComms {
    pub fn new(client_settings: ClientSettings) -> Self {
        Self {
            comms: Comms::new(None),
            settings: client_settings,
            update_count: 0,
        }
    }

    pub fn connect(&mut self) -> Result<(), ClientError> {
        self.comms
            .connect(&self.settings.server_ip)
            .map_err(|_| ClientError::ConnectionError)
    }

    pub fn join_server(&mut self) -> Result<(), ClientError> {
        let register_msg = Message::JoinLobby {
            player_name: self.settings.nickname.clone(),
        };
        self.comms.send_message(&register_msg).map_err(|_| ClientError::ConnectionError)?;

        let response = self
            .comms
            .receive_message()
            .map_err(|_| ClientError::ConnectionError)?;

        match response {
            Message::Ok => Ok(()),
            Message::Nok { error_msg: msg } => {
                eprintln!("[ERROR]: Failed to join lobby: {}", msg);
                Err(ClientError::Unknown("".to_string()))
            }
            _ => Err(ClientError::Unknown("Unexpected message received".to_string())),
        }?;

        Ok(())
    }

    pub fn send_input(&mut self, c: char) -> Result<(), ClientError> {
        let message = Message::SendInput(c);
        self.comms.send_message(&message).map_err(|err| {
            eprintln!("[ERROR]: failed to send input: {:?}", err);
            ClientError::ConnectionError
        })
    }

    pub fn receive_game_update(&mut self) -> Result<Option<GameCore>, ClientError> {
        let response = self.comms.receive_message();
        match response {
            Ok(response) => {
                if let Message::GameUpdate(new_state) = response {
                    self.update_count += 1;
                    if self.update_count % TICK_RATE_FREQ as usize == 0 {
                        println!("Received {} updates!", TICK_RATE_FREQ);
                    }

                    return Ok(Some(new_state));
                } else {
                    return Err(ClientError::Unknown("Got invalid message".to_string()));
                }
            }
            Err(CommError::WouldBlock) => Ok(None),
            Err(_) => return Err(ClientError::ConnectionError),
        }
    }
}
