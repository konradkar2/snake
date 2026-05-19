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

fn parse_args() -> ClientSettings {
    let mut args = env::args();
    args.next().expect("executable name");

    let nickname: String = args.next().unwrap_or(DEFAULT_NICKNAME.to_string());
    let ip: String = args.next().expect("Please provide ip");

    ClientSettings {
        nickname: nickname,
        server_ip: ip,
    }
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

#[macroquad::main("Snake")]
async fn main() -> Result<(), ()> {
    let client_settings: ClientSettings = parse_args();
    client_settings.print();

    setup_screen();

    let game_lock = Arc::new(Mutex::new(GameLocal::new(&client_settings.nickname)));

    {
        let game = game_lock.lock().unwrap();
        println!("gameLocal: {:?}", game);
    }

    let game_lock_comms = Arc::clone(&game_lock);

    let (tx, rx): (Sender<char>, Receiver<char>) = channel();
    thread::spawn(move || {
        let mut client = ClientComms::new(Comms::new(None), client_settings);

        client.connect().expect("failed to connect to the server");
        client.join_server().expect("failed to join the server");

        loop {
            {
                if let Ok(c) = rx.try_recv() {
                    {
                        let mut game = game_lock_comms.lock().unwrap();
                        game.handle_input(c);
                    }
                    client.send_input(c).unwrap();
                }
            }

            match client.receive_game_update() {
                Ok(game_update) => {
                    if let Some(new_game_state) = game_update {
                        let mut game = game_lock_comms.lock().unwrap();
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

    run_drawing(game_lock, tx).await;

    Ok(())
}
