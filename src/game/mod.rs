use std::collections::HashMap;

use crate::config::*;
use rand::Rng;

use crate::snake::{Direction, Snake};
pub mod snake;

use macroquad::prelude as mcq;

#[derive(Debug)]
enum GameState {
    Menu,
    Playing,
    Paused,
    Finished,
}

pub struct Player {
    is_loser: bool,
}

pub struct Game {
    state: GameState,
    is_multiplayer: bool,
    players: HashMap<String, Player>,
    snakes: HashMap<String, Snake>,
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
    pub fn new() -> Self {
        let mut ret = Self {
            state: GameState::Menu,
            is_multiplayer: false,
            snakes: HashMap::from([(
                String::from("player"),
                Snake::new(SNAKE_SIZE, SNAKE_SPEED, mcq::GREEN),
            )]),
            players: HashMap::from([(String::from("player"), Player { is_loser: false })]),
            fruit_pos: mcq::vec2(0.0, 0.0),
            movement_update_no: 0,
            rng: rand::rng(),
        };

        ret.update_fruit_pos();

        ret
    }

    fn generate_fruit_pos(&mut self) -> mcq::Vec2 {
        let mut pos = mcq::Vec2::new(0.0, 0.0);

        let n_snake_fits_x: u32 = (SCREEN_WIDTH / SNAKE_SIZE) as u32;
        let n_snake_fits_y: u32 = (SCREEN_HEIGHT / SNAKE_SIZE) as u32;

        pos.x = SNAKE_SIZE * self.rng.random_range(0..n_snake_fits_x) as f32;
        pos.y = SNAKE_SIZE * self.rng.random_range(0..n_snake_fits_y) as f32;

        pos
    }

    fn update_fruit_pos(&mut self) {
        let mut new_fruit_pos;

        loop {
            new_fruit_pos = self.generate_fruit_pos();
            let mut fruit_collides = false;
            for (_, (_, snake)) in self.snakes.iter().enumerate() {
                if snake.collides_fruit(&new_fruit_pos) {
                    fruit_collides = true;
                }
            }
            if !fruit_collides {
                break;
            }
        }

        self.fruit_pos = new_fruit_pos;
    }

    pub fn update(&mut self) {
        self.handle_input();

        match self.state {
            GameState::Playing => {
                self.movement_update_no += 1;

                let mut fruit_colided = false;
                for (player_name, snake) in self.snakes.iter_mut() {
                    if snake.collides_self() {
                        self.state = GameState::Finished;
                        self.players.get_mut(player_name).unwrap().is_loser = true;
                        return;
                    }

                    if snake.collides_fruit(&self.fruit_pos) {
                        snake.grow();
                        fruit_colided = true;
                    }

                    if self.movement_update_no % SNAKE_UPDATE_STEP == 0 {
                        snake.move_step();
                    }
                }

                if fruit_colided {
                    self.update_fruit_pos();
                }
            }
            _ => {}
        }
    }

    fn handle_input(&mut self) {
        if let Some(c) = mcq::get_char_pressed() {
            match self.state {
                GameState::Playing => match c {
                    'w' => self.snakes.get_mut("player").unwrap().change_direction(Direction::Up),
                    's' => self.snakes.get_mut("player").unwrap().change_direction(Direction::Down),
                    'a' => self.snakes.get_mut("player").unwrap().change_direction(Direction::Left),
                    'd' => self.snakes.get_mut("player").unwrap().change_direction(Direction::Right),
                    'l' => self.snakes.get_mut("player").unwrap().grow(),
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
                GameState::Finished => match c {
                    '\x0D' => {
                        *self.snakes.get_mut("player").unwrap() =
                            Snake::new(SNAKE_SIZE, SNAKE_SPEED, mcq::GREEN);

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
        for (_, snake) in self.snakes.iter() {
            snake.draw();
        }

        mcq::draw_rectangle(
            self.fruit_pos.x,
            self.fruit_pos.y,
            SNAKE_SIZE,
            SNAKE_SIZE,
            mcq::YELLOW,
        );
    }

    pub fn draw(&mut self) {
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
            GameState::Finished => {
                let winner_name: &str;

                if self.is_multiplayer {
                    winner_name = self
                        .players
                        .iter()
                        .find(|(_, player)| player.is_loser == false)
                        .expect("a player has lost")
                        .0;
                } else {
                    winner_name = ""
                }

                mcq::draw_rectangle(
                    0.0,
                    0.0,
                    mcq::screen_width(),
                    mcq::screen_height(),
                    mcq::DARKPURPLE,
                );

                let text: String;
                if self.is_multiplayer {
                    text = format!("player: {}", winner_name);
                } else {
                    text = String::from("You lost!");
                }
                draw_big_text(text.as_str(), mcq::RED);
            }
            _ => self.draw_objects(),
        }
    }
}
