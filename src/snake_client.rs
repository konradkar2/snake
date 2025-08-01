use std::{
    any::Any,
    sync::{
        Mutex,
        mpsc::{Receiver, Sender},
    },
    thread::{self},
    time,
};

use crate::game::*;
pub mod game;

use crate::snake_cfg::*;
pub mod snake_cfg;

pub mod common;
pub mod comms;
pub mod ifc;

use crate::comms::*;
use std::io;

use bincode::config;
use ifc::Message;
use macroquad::prelude as mcq;

use std::env;
use std::sync::Arc;

use std::sync::mpsc::channel;

fn setup_screen() {
    mcq::request_new_screen_size(SCREEN_WIDTH, SCREEN_HEIGHT);
    mcq::next_frame();
}

#[derive(Debug)]
struct GameArgs {
    nickname: String,
    server_ip: String,
}

fn parse_args() -> GameArgs {
    let mut args = env::args();
    args.next().expect("executable name");

    let nickname: String = args.next().unwrap_or(DEFAULT_NICKNAME.to_string());
    let ip: String = args.next().expect("Podaj ip huju");

    GameArgs {
        nickname: nickname,
        server_ip: ip,
    }
}

fn print_game_args(game_args: &GameArgs) {
    println!("[Info]: nickname is '{}'", game_args.nickname);
}

struct Client {
    comms: Comms,
    game_args: GameArgs,
    game_guard: Arc<Mutex<GameLocal>>,
    update_count: usize,
}

impl Client {
    fn connect(&mut self) -> Result<(), ClientError> {
        self.comms
            .connect(&self.game_args.server_ip)
            .map_err(|_| ClientError::ConnectionError)
    }

    fn join_server(&mut self) -> Result<(), ClientError> {
        let register_msg = Message::JoinLobby(self.game_args.nickname.clone());
        self.comms.send_message(&register_msg).unwrap();
        let response = self
            .comms
            .receive_message()
            .map_err(|_| ClientError::ConnectionError)?;

        match response {
            Message::Ok => Ok(()),
            Message::Nok(msg) => {
                eprintln!("[ERROR]: Failed to join lobby: {}", msg);
                Err(ClientError::Unknown("".to_string()))
            }
            _ => Err(ClientError::Unknown("".to_string())),
        }?;

        Ok(())
    }

    fn send_input(&mut self, c: char) -> Result<(), ClientError> {
        let message = Message::SendInput(c);
        self.comms.send_message(&message).map_err(|err| {
            eprintln!("[ERROR]: failed to send input: {:?}", err);
            ClientError::ConnectionError
        })
    }

    fn synch_game(&mut self) -> Result<(), ClientError> {
        let response = self.comms.receive_message();
        match response {
            Ok(response) => {
                if let Message::GameUpdate(new_state) = response {
                    let mut game_guard = self.game_guard.lock().unwrap();
                    game_guard.game_core = new_state;

                    self.update_count += 1;
                    if self.update_count % TICK_RATE_FREQ as usize == 0 {
                        println!("Received {} updates!", TICK_RATE_FREQ);
                    }
                }
            }
            Err(CommError::WaitingForMoreData) => {
                eprintln!("[WARNING] waiting for more data...")
            }
            Err(CommError::WouldBlock) => {}
            Err(_) => return Err(ClientError::ConnectionError),
        }

        Ok(())
    }
}

#[macroquad::main("MyGame")]
async fn main() -> Result<(),()>{
    let game_args: GameArgs = parse_args();
    print_game_args(&game_args);

    setup_screen();

    let mut frame_started_t: f64 = 0.0;

    let game_lock = Arc::new(Mutex::new(GameLocal::new(&game_args.nickname)));

    {
        let game = game_lock.lock().unwrap();
        println!("gameLocal: {:?}", game);
    }

    // let mut client_handle: TcpStream;

    let game_lock_comms = Arc::clone(&game_lock);

    let (tx, rx): (Sender<char>, Receiver<char>) = channel();
    thread::spawn(move || {
        let mut client = Client {
            comms: Comms::new(None),
            game_args: game_args,
            game_guard: game_lock_comms,
            update_count: 0,
        };

        client.connect().expect("failed to connect to the server");
        client.join_server().expect("failed to join the server");


        loop {
            {
                if let Ok(c) = rx.try_recv() {
                    {
                        let mut game = client.game_guard.lock().unwrap();
                        game.handle_input(c);
                    }
                    client.send_input(c).unwrap();
                }
            }

            client.synch_game().unwrap();
            thread::sleep(time::Duration::from_secs_f64(TICK_RATE_TIME));
        }
    });

    let game_lock_update = Arc::clone(&game_lock);
    thread::spawn(move || {
        loop {
            {
                let mut game = game_lock_update.lock().unwrap();
                game.update();
            }
            thread::sleep(time::Duration::from_secs_f64(TICK_RATE_TIME));
        }
    });

    loop {
        {
            if let Some(c) = mcq::get_char_pressed() {
                tx.send(c).unwrap();
            }
            let game = game_lock.lock().unwrap();
            game.draw();
        }

        while (macroquad::time::get_time() - frame_started_t) <= FRAME_TIME {
            thread::sleep(time::Duration::from_micros(500));
        }

        frame_started_t = macroquad::time::get_time();
        mcq::next_frame().await
    }
}
