use std::{
    thread::{self},
    time,
};

use crate::game::*;
pub mod game;

use crate::config::*;
pub mod config;

use macroquad::prelude as mcq;

#[macroquad::main("MyGame")]
async fn main() {
    let mut frame_started_t: f64 = 0.0;

    mcq::request_new_screen_size(SCREEN_WIDTH, SCREEN_HEIGHT);
    mcq::next_frame().await;

    let mut game = Game::new();

    loop {
        game.draw();
        mcq::draw_fps();

        game.update();

        while (macroquad::time::get_time() - frame_started_t) <= FRAME_TIME {
            thread::sleep(time::Duration::from_micros(500));
        }
        frame_started_t = macroquad::time::get_time();

        mcq::next_frame().await
    }
}
