use std::{
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

pub mod client_comms;
pub mod common;
pub mod comms;
pub mod ifc;

use crate::comms::*;

use crate::client_comms::*;
use macroquad::prelude as mcq;

use std::env;
use std::sync::Arc;

use std::sync::mpsc::channel;

fn setup_screen() {
    mcq::request_new_screen_size(SCREEN_WIDTH, SCREEN_HEIGHT);
    mcq::next_frame();
}

fn print_help() {
    println!("<nickname> <server IP address:port>\n");
}

fn parse_args() -> Option<ClientSettings> {
    let mut args = env::args();
    args.next().expect("executable name");

    let nickname: String = args.next()?;
    let ip: String = args.next()?;

    Some(ClientSettings {
        nickname: nickname,
        server_ip: ip,
    })
}

async fn run_drawing(game_lock: Arc<Mutex<GameLocal>>, input_tx: Sender<char>) {
    let mut frame_started_t: f64 = 0.0;
    loop {
        {
            if let Some(c) = mcq::get_char_pressed() {
                input_tx.send(c).unwrap();
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

fn run_connection_thread(
    game_lock: Arc<Mutex<GameLocal>>,
    mut client_comms: ClientComms,
    input_rx: Receiver<char>,
) {
    thread::spawn(move || {
        client_comms
            .connect()
            .expect("failed to connect to the server");
        client_comms
            .join_server()
            .expect("failed to join the server");

        loop {
            if let Ok(c) = input_rx.try_recv() {
                {
                    let mut game = game_lock.lock().unwrap();
                    game.handle_input(c);
                }
                client_comms.send_input(c).unwrap();
            }

            match client_comms.receive_game_update() {
                Ok(game_update) => {
                    if let Some(new_game_state) = game_update {
                        let mut game = game_lock.lock().unwrap();
                        game.game_core = new_game_state;
                    }
                }
                Err(err) => {
                    eprintln!("error: {:?}", err);
                    panic!()
                }
            }
            thread::sleep(time::Duration::from_secs_f64(TICK_RATE_TIME));
        }
    });
}

fn run_game_logic_thread(game_lock: Arc<Mutex<GameLocal>>) {
    thread::spawn(move || {
        loop {
            {
                let mut game = game_lock.lock().unwrap();
                game.update();
            }
            thread::sleep(time::Duration::from_secs_f64(TICK_RATE_TIME));
        }
    });
}

#[macroquad::main("Snake")]
async fn main() -> Result<(), ()> {
    let client_settings = match parse_args() {
        Some(cs) => cs,
        None => {
            print_help();
            return Err(());
        }
    };

    client_settings.print();
    setup_screen();

    let game_lock = Arc::new(Mutex::new(GameLocal::new(&client_settings.nickname)));

    let (input_tx, input_rx): (Sender<char>, Receiver<char>) = channel();
    run_connection_thread(
        game_lock.clone(),
        ClientComms::new(client_settings),
        input_rx,
    );
    run_game_logic_thread(game_lock.clone());
    run_drawing(game_lock, input_tx).await;

    Ok(())
}
