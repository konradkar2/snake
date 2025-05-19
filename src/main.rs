use std::{
    thread::{self},
    time,
};

use rand::Rng;

use crate::snake::{Direction, Snake};
pub mod snake;

use macroquad::{color::YELLOW, prelude as mcq};

const FPS: u64 = 30;
const FRAME_TIME: f64 = 1.0 / FPS as f64;
const SNAKE_UPDATE_FREQ: u64 = 10;
const SNAKE_UPDATE_STEP: u64 = FPS / SNAKE_UPDATE_FREQ;

const SNAKE_SIZE: f32 = 20.0;
const SNAKE_SPEED: f32 = 20.0;

#[derive(Debug)]
enum GameState {
    Menu,
    Playing,
    Paused,
    Lost,
}

struct Game {
    state: GameState,
    is_multiplayer: bool,
    snake: Snake,
    fruit_pos: mcq::Vec2,
    movement_update_no: u64,
    rng: ::rand::rngs::ThreadRng,
}

fn draw_text_center(text: &str, x: f32, y: f32, font_size: f32, color: mcq::Color) {
    let text_dims = mcq::measure_text(text, None, font_size as u16, 1.0);
    mcq::draw_text(
        text,
        x - text_dims.width / 2.0,
        y + text_dims.height / 2.0 - text_dims.offset_y / 2.0,
        100.0,
        color,
    );
}

fn draw_big_text(text: &str, color: mcq::Color) {
    draw_text_center(
        text,
        mcq::screen_width() / 2.0,
        mcq::screen_height() / 2.0,
        100.0,
        color,
    );
}

impl Game {
    fn new() -> Self {
        let mut ret = Self {
            state: GameState::Menu,
            is_multiplayer: false,
            snake: Snake::new(SNAKE_SIZE, SNAKE_SPEED),
            fruit_pos: mcq::vec2(0.0, 0.0),
            movement_update_no: 0,
            rng: rand::rng(),
        };

        ret.update_fruit_pos();

        ret
    }

    fn generate_fruit_pos(&mut self) -> mcq::Vec2 {
        let mut pos = mcq::Vec2::new(0.0, 0.0);

        let n_snake_fits_x: u32 = (mcq::screen_width() / SNAKE_SIZE) as u32;
        let n_snake_fits_y: u32 = (mcq::screen_height() / SNAKE_SIZE) as u32;

        pos.x = SNAKE_SIZE * self.rng.random_range(0..n_snake_fits_x) as f32;
        pos.y = SNAKE_SIZE * self.rng.random_range(0..n_snake_fits_y) as f32;

        pos
    }

    fn update_fruit_pos(&mut self) {
        let mut new_pos = self.generate_fruit_pos();
        while self.snake.collides_fruit(&new_pos) {
            new_pos = self.generate_fruit_pos();
        }
        self.fruit_pos = new_pos;
    }

    fn update(&mut self) {
        self.handle_input();

        match self.state {
            GameState::Playing => {
                if self.snake.collides_self() {
                    self.state = GameState::Lost;
                    return;
                }
                if self.snake.collides_fruit(&self.fruit_pos) {
                    self.snake.grow();
                    self.update_fruit_pos();
                }

                self.movement_update_no += 1;
                if self.movement_update_no % SNAKE_UPDATE_STEP == 0 {
                    self.snake.move_step();
                }
            }
            _ => {}
        }
    }

    fn handle_input(&mut self) {
        if let Some(c) = mcq::get_char_pressed() {
            match self.state {
                GameState::Playing => match c {
                    'w' => self.snake.change_direction(Direction::Up),
                    's' => self.snake.change_direction(Direction::Down),
                    'a' => self.snake.change_direction(Direction::Left),
                    'd' => self.snake.change_direction(Direction::Right),
                    'l' => self.snake.grow(),
                    'k' => self.update_fruit_pos(),
                    '\x1B' /*escape */ =>  self.state = GameState::Paused,
                    _ => {}
                },
                GameState::Paused => match c {
                         '\x1B' /*escape */ =>  self.state = GameState::Playing,
                         _ => {}
                    },
                GameState::Menu => match c {
                    '\x0D' => self.state = GameState::Playing,
                    _ => {}
                },
                GameState::Lost => match c {
                    '\x0D' => {
                        self.snake = Snake::new(SNAKE_SIZE, SNAKE_SPEED);
                        self.state = GameState::Playing;
                    }
                    _ => {}
                },
                _ => match c {
                    _ => {
                        println!("pressed: {}", c)
                    }
                },
            }
        }
    }

    fn draw_objects(&self) {
        self.snake.draw();
        mcq::draw_rectangle(
            self.fruit_pos.x,
            self.fruit_pos.y,
            SNAKE_SIZE,
            SNAKE_SIZE,
            mcq::YELLOW,
        );
    }

    fn draw(&mut self) {
        mcq::clear_background(mcq::RED);
        match self.state {
            GameState::Paused => {
                mcq::draw_rectangle(
                    0.0,
                    0.0,
                    mcq::screen_width(),
                    mcq::screen_height(),
                    mcq::BLUE,
                );
                self.draw_objects();
                draw_big_text("Paused", mcq::RED);
            }
            GameState::Menu => {
                draw_big_text("Start", mcq::GREEN);
            }
            GameState::Lost => {
                mcq::draw_rectangle(
                    0.0,
                    0.0,
                    mcq::screen_width(),
                    mcq::screen_height(),
                    mcq::DARKPURPLE,
                );
                draw_big_text("You lost!", mcq::RED);
            }
            _ => self.draw_objects(),
        }
    }
}

#[macroquad::main("MyGame")]
async fn main() {
    let mut frame_started_t: f64 = 0.0;

    mcq::request_new_screen_size(800.0, 600.0);
    mcq::next_frame().await;

    let mut game = Game::new();

    loop {
        mcq::draw_fps();
        game.draw();

        game.update();

        while (macroquad::time::get_time() - frame_started_t) <= FRAME_TIME {
            thread::sleep(time::Duration::from_micros(500));
        }
        frame_started_t = macroquad::time::get_time();

        mcq::next_frame().await
    }
}
