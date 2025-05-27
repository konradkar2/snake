use std::{
    env::Args,
    io::Write,
    net::TcpStream,
    sync::{Mutex, RwLock},
    thread::{self},
    time,
};

use crate::game::*;
pub mod game;

use crate::config::*;
pub mod config;

use crate::multiplayer::server::*;
pub mod multiplayer;

pub mod common;
pub mod ifc;

use bincode::config as BincodeConfig;
use ifc::Message;
use macroquad::prelude as mcq;

use std::env;
use std::sync::Arc;

fn setup_screen() {
    mcq::request_new_screen_size(SCREEN_WIDTH, SCREEN_HEIGHT);
    mcq::next_frame();
}

#[derive(Debug)]
enum Role {
    Server,
    Client,
}

#[derive(Debug)]
struct GameArgs {
    role: Role,
    nickname: String,
}

fn parse_args() -> GameArgs {
    let mut args = env::args();
    args.next().expect("executable name");

    let role_str = args.next().unwrap_or(String::from("server"));

    let role = match role_str.trim_end() {
        "server" => Role::Server,
        _ => Role::Client,
    };

    let nickname: String = args.next().unwrap_or(DEFAULT_NICKNAME.to_string());

    GameArgs {
        role: role,
        nickname: nickname,
    }
}

fn print_game_args(game_args: &GameArgs) {
    println!("[Info]: role is '{:?}'", game_args.role);
    println!("[Info]: nickname is '{}'", game_args.nickname);
}

#[macroquad::main("MyGame")]
async fn main() {
    let game_args: GameArgs = parse_args();
    print_game_args(&game_args);

    setup_screen();

    let bincode_cfg = BincodeConfig::standard();

    let mut frame_started_t: f64 = 0.0;

    let game_lock = Arc::new(Mutex::new(GameLocal::new(false, &game_args.nickname)));

    {
        let game = game_lock.lock().unwrap();
        println!("gameLocal: {:?}", game);
    }

    let server_handle: ServerHandle;
    let mut client_handle: TcpStream;
    match game_args.role {
        Role::Server => {
            server_handle = ServerHandle::new(bincode_cfg);
            server_handle.launch_server();
        }
        _ => {
            let register_msg = Message::JoinLobby(game_args.nickname);
            let encoded: Vec<u8> = bincode::encode_to_vec(&register_msg, bincode_cfg).unwrap();
            client_handle = TcpStream::connect("127.0.0.1:223").expect("connects to server");
            let bytes = client_handle.write(&encoded).unwrap();
            println!("written n bytes: {bytes}");
        }
    }

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
            let mut game = game_lock.lock().unwrap();
            if let Some(c) = mcq::get_char_pressed() {
                game.handle_input(c);
            }
            game.draw();
        }

        while (macroquad::time::get_time() - frame_started_t) <= FRAME_TIME {
            thread::sleep(time::Duration::from_micros(500));
        }

        frame_started_t = macroquad::time::get_time();
        mcq::next_frame().await
    }
}
