use std::collections::HashMap;

use crate::{
    common::{MyColor, MyVec2},
    config::*,
};
use rand::{rng, Rng};
use macroquad::prelude as mcq;


use crate::snake::{Direction, Snake};


use bincode::{Decode, Encode};

#[derive(Encode, Decode, Debug)]
pub enum GameState {
    Menu,
    Playing,
    Paused,
    Finished,
}

#[derive(Encode, Decode, Debug)]
pub struct Player {
    pub is_loser: bool,
}

#[derive(Encode, Decode, Debug)]
pub struct GameCore {
    pub state: GameState,
    pub players: HashMap<String, Player>,
    snakes: HashMap<String, Snake>,
    pub fruit_pos: Option<MyVec2>,
}

fn generate_fruit_pos() -> MyVec2 {
    let mut pos = MyVec2::new(0.0, 0.0);

    let n_snake_fits_x: u32 = (SCREEN_WIDTH / SNAKE_SIZE) as u32;
    let n_snake_fits_y: u32 = (SCREEN_HEIGHT / SNAKE_SIZE) as u32;

    pos.x = SNAKE_SIZE * rng().random_range(0..n_snake_fits_x) as f32;
    pos.y = SNAKE_SIZE * rng().random_range(0..n_snake_fits_y) as f32;

    pos
}

impl GameCore {
    pub fn new() -> Self {
        Self {
            state: GameState::Menu,
            snakes: HashMap::new(),
            players: HashMap::new(),
            fruit_pos: None,
        }
    }

    pub fn add_player(&mut self, name: &str, color: MyColor) {
        self.snakes.insert(name.to_string(), Snake::new(color));
        self.players
            .insert(name.to_string(), Player { is_loser: false });
    }

    pub fn update_fruit_pos(&mut self) {
        let mut new_fruit_pos;

        loop {
            new_fruit_pos = generate_fruit_pos();
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

        self.fruit_pos = Some(new_fruit_pos);
    }

    pub fn update(&mut self, update_fruit_pos: bool) {
        match self.state {
            GameState::Playing => {
                for (player_name, snake) in self.snakes.iter_mut() {
                    if snake.collides_self() {
                        self.state = GameState::Finished;
                        self.players.get_mut(player_name).unwrap().is_loser = true;
                        return;
                    }

                    if let Some(fruit_pos) = &self.fruit_pos {
                        if snake.collides_fruit(fruit_pos) {
                            snake.grow();
                            self.fruit_pos = None;
                        }
                    }

                    snake.move_step_tick();
                }

                if update_fruit_pos && self.fruit_pos.is_none() {
                    self.update_fruit_pos();
                }
            }
            _ => {}
        }
    }

    pub fn handle_input(&mut self, player_name: &str, c: char) {
        match self.state {
            GameState::Playing => match c {
                    'w' => self.snakes.get_mut(player_name).unwrap().change_direction(Direction::Up),
                    's' => self.snakes.get_mut(player_name).unwrap().change_direction(Direction::Down),
                    'a' => self.snakes.get_mut(player_name).unwrap().change_direction(Direction::Left),
                    'd' => self.snakes.get_mut(player_name).unwrap().change_direction(Direction::Right),
                    'l' => self.snakes.get_mut(player_name).unwrap().grow(),
                    'k' => self.update_fruit_pos(),
                    '\x1B' /*escape */ =>  self.state = GameState::Paused,
                    _ => {}
                },
            GameState::Paused => match c {
                         '\x1B' /*escape */ =>  self.state = GameState::Playing,
                         _ => {}
                    },
            GameState::Menu => match c {
                '\x0D' /*enter */ => self.state = GameState::Playing,
                _ => {}
            },
            GameState::Finished => match c {
                // '\x0D' => {
                //     *self.snakes.get_mut(player_name).unwrap() =
                //         Snake::new(SNAKE_SIZE, SNAKE_SPEED, from_color(mcq::GREEN));

                //     self.state = GameState::Playing;
                // }
                _ => {}
            },
            _ => match c {
                _ => {
                    println!("pressed: {}", c)
                }
            },
        }
    }

    pub fn draw_objects(&self) {
        for (_, snake) in self.snakes.iter() {
            snake.draw();
        }

        if let Some(fruit_pos) = self.fruit_pos {
            mcq::draw_rectangle(
                fruit_pos.x,
                fruit_pos.y,
                SNAKE_SIZE,
                SNAKE_SIZE,
                mcq::YELLOW,
            );
        }
    }
}
