use macroquad::color::{GREEN, PINK};


pub const FPS: u64 = 60;
pub const TICK_RATE_FREQ: u64 = 120;
pub const TICK_RATE_TIME: f64 = 1.0 / TICK_RATE_FREQ as f64;

pub const FRAME_TIME: f64 = 1.0 / FPS as f64;
pub const SNAKE_UPDATE_FREQ: u64 = 15;
pub const SNAKE_UPDATE_STEP: u64 = FPS / SNAKE_UPDATE_FREQ;

pub const SNAKE_SIZE: f32 = 20.0;
pub const SNAKE_TICKS_PER_MOVE: f32 = 20.0;

pub const SCREEN_WIDTH: f32 = 800.0;
pub const SCREEN_HEIGHT: f32 = 600.0;

pub const DEFAULT_NICKNAME: &'static str = "player";
pub const SERVER_ADDRESS: &'static str = "0.0.0.0:6969";

pub const PLAYER_COUNT_MAX: usize = 2;

pub const PLAYER_COLORS: [macroquad::color::Color;  PLAYER_COUNT_MAX] = [PINK, GREEN];
