use std::{
    env::Args,
    io::Write,
    net::TcpStream,
    thread::{self},
    time,
};

use crate::game::*;
pub mod game;

use crate::config::*;
pub mod config;

use crate::server::*;
pub mod server;

pub mod common;
pub mod ifc;

use bincode::config as BincodeConfig;
use ifc::{Message};
use macroquad::prelude as mcq;
use rand::rng;
use std::env;

enum Role {
    Server,
    Client,
}

fn read_role(args: &mut Args) -> Role {
    let role_str = args.next().unwrap_or(String::from("server"));
    println!("[info]: role is '{role_str}'");
    match role_str.trim_end() {
        "server" => Role::Server,
        _ => Role::Client,
    }
}


#[macroquad::main("MyGame")]
async fn main() {
    let mut args = env::args();
    args.next();

    let role = read_role(&mut args);
    let player_name: String = args.next().unwrap_or("player".to_string());

    println!("[info]: player name: '{player_name}'");
    let bincode_cfg = BincodeConfig::standard();

    let mut frame_started_t: f64 = 0.0;

    mcq::request_new_screen_size(SCREEN_WIDTH, SCREEN_HEIGHT);
    mcq::next_frame().await;
    let mut rng = rng();

    let mut game = GameLocal::new(false, &player_name);
    game.game_core.update_fruit_pos(&mut rng);

    let server_handle: ServerHandle;
    let mut client_handle: TcpStream;
    match role {
        Role::Server => {
            server_handle = ServerHandle::new(bincode_cfg);
            server_handle.launch_server();
        }
        _ => {
            let register_msg = Message::JoinLobby(player_name);
            let encoded: Vec<u8> = bincode::encode_to_vec(&register_msg, bincode_cfg).unwrap();
            client_handle = TcpStream::connect("127.0.0.1:223").expect("connects to server");
            let bytes = client_handle.write(&encoded).unwrap();
            println!("written n bytes: {bytes}");
        }
    }

    println!("gameLocal: {:?}", game);

    loop {
        game.draw();
        mcq::draw_fps();

        if let Some(c) = mcq::get_char_pressed() {
            game.handle_input(c, &mut rng);
        }

        game.game_core.update(&mut rng);

        while (macroquad::time::get_time() - frame_started_t) <= FRAME_TIME {
            thread::sleep(time::Duration::from_micros(500));
        }

        frame_started_t = macroquad::time::get_time();
        mcq::next_frame().await
    }
}
