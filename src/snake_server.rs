use bincode::config;
use common::from_color;
use game::game_core::GameCore;
use macroquad::color::{GREEN, PINK};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Error;
use std::net::{TcpListener, TcpStream};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::{io, thread, time};

pub mod ifc;
use crate::ifc::*;

pub mod snake_cfg;
use crate::snake_cfg::*;

pub mod comms;
use crate::comms::*;

pub mod game;
use crate::game::*;

pub mod common;

enum ServerState {
    WaitingForPlayers,
    Running,
}

struct Server {
    state: ServerState,
    game_guard: Arc<Mutex<GameCore>>,
    player_comms: HashMap<String, Rc<RefCell<Comms>>>,
}

impl Server {
    fn try_add_player(&mut self, nickname: &str, comms: Rc<RefCell<Comms>>) -> Result<(), String> {
        if self.player_comms.contains_key(nickname) {
            Err(format!("Player {} is already added", nickname))?
        }

        {
            let mut game_guard = self.game_guard.lock().unwrap();
            game_guard.add_player(&nickname);
        }

        self.player_comms.insert(nickname.to_string(), comms);
        Ok(())
    }

    fn remove_player(&mut self, nickname: &str) {
        println!("[WARNING] removing player {}", nickname);
        self.player_comms.remove(nickname);
        self.state = ServerState::WaitingForPlayers;

        let mut game_guard = self.game_guard.lock().unwrap();
        game_guard.remove_player(nickname);
    }

    fn handle_connection(&mut self, stream: TcpStream) -> Result<(), CommError> {
        println!("[SERVER]: client connecting {:?}...", stream.peer_addr());
        let comms_rc = Rc::new(RefCell::new(Comms::new(Some(stream))));

        let msg: Message = comms_rc.borrow_mut().receive_message()?;
        println!("[SERVER]: got msg: {:?}", msg);

        let mut resp: Message;

        if let Message::JoinLobby(nickname) = msg {
            resp = Message::Ok;

            let result = self.try_add_player(nickname.as_str(), comms_rc.clone());

            if result.is_err() {
                let msg = format!("Player '{}' is already added", nickname.as_str());
                eprintln!("[WARNING]: {}", msg);
                resp = Message::Nok(msg);
            } else {
                comms_rc.borrow_mut().set_nonblocking();
            }
        } else {
            resp = Message::Nok("Invalid message, expected join lobby".to_string());
        }

        comms_rc.borrow_mut().send_message(&resp)
    }

    fn send_update(&mut self) {
        let game_copy: GameCore = {
            let game_guard = self.game_guard.lock().unwrap();
            game_guard.clone()
        };

        let mut disconnected_players: Vec<String> = Vec::new();

        for (player_name, player_rc) in &self.player_comms {
            let _ = player_rc
                .borrow_mut()
                .send_message(&Message::GameUpdate(game_copy.clone()))
                .inspect_err(|err| match err {
                    CommError::WouldBlock => {}
                    _ => {
                        disconnected_players.push(player_name.clone());
                    }
                });
        }

        for player in disconnected_players {
            self.remove_player(player.as_str());
        }
    }

    fn receive_messages(&mut self) {
        let mut disconnected_players: Vec<String> = Vec::new();

        for (player_name, comms_rc) in &self.player_comms {
            let message = comms_rc.borrow_mut().receive_message();
            match message {
                Ok(message) => match message {
                    Message::SendInput(c) => {
                        let mut game = self.game_guard.lock().unwrap();
                        game.handle_input(player_name.as_str(), c);
                    }
                    _ => {
                        eprintln!("[ERROR] unexpected message received");
                    }
                },
                Err(e) => match e {
                    CommError::WouldBlock => {}
                    _ => {
                        disconnected_players.push(player_name.clone());
                    }
                },
            }
        }

        for player in disconnected_players {
            self.remove_player(player.as_str());
        }
    }

    fn main_loop(&mut self) -> io::Result<()> {
        let listener = TcpListener::bind(SERVER_ADDRESS).inspect_err(|err| {
            eprintln!("[ERROR]: failed to bind to address {}: {}", SERVER_ADDRESS, err)
        })?;

        listener.set_nonblocking(true).inspect_err(|err| {
            eprintln!("[ERROR]: failed to set nonblocking on socket");
        })?;

        println!("[INFO]: Listening for connection at {}...", SERVER_ADDRESS);

        loop {
            self.send_update();

            match self.state {
                ServerState::WaitingForPlayers => {
                    let result = listener.accept();
                    match result {
                        Ok((stream, _socket_addr)) => {
                            let _ = self.handle_connection(stream);
                            if self.player_comms.iter().count() == 2 {
                                println!("2 players joined the lobby, starting the game");
                                self.state = ServerState::Running;
                            }
                        }
                        Err(e) => {
                            if e.kind() != io::ErrorKind::WouldBlock {
                                println!("[ERROR] failed to handle stream");
                            }
                        }
                    }
                    if !self.player_comms.is_empty() {
                        self.receive_messages();
                    }
                }

                ServerState::Running => {
                    self.receive_messages();
                }
            }
            thread::sleep(time::Duration::from_secs_f64(TICK_RATE_TIME));
        }
    }
}

fn launch_game_update_thread(game_guard: Arc<Mutex<GameCore>>) {
    std::thread::spawn(move || {
        println!("[INFO]: starting game thread...");

        loop {
            {
                let mut game = game_guard.lock().unwrap();
                game.update();
            }

            thread::sleep(time::Duration::from_secs_f64(TICK_RATE_TIME));
        }
    });
}

fn main() -> Result<(), ()>{
    let game_guard = Arc::new(Mutex::new(GameCore::new(true)));

    let mut server = Server {
        state: ServerState::WaitingForPlayers,
        game_guard: game_guard.clone(),
        player_comms: HashMap::new(),
    };

    launch_game_update_thread(game_guard.clone());

    server.main_loop().map_err(|_err| {
        eprintln!("[ERROR]: failed to start main loop");
    })
}
