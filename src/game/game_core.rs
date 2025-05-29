use std::collections::HashMap;

use crate::{
    common::{MyColor, MyVec2},
    snake_cfg::*,
};
use macroquad::prelude as mcq;
use rand::{Rng, rng};

use crate::snake::{Direction, Snake};

use bincode::{Decode, Encode};

#[derive(Encode, Decode, Debug, Clone)]
pub enum GameState {
    NotPlaying,
    Playing,
}

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub enum PlayerState {
    NotReady,
    Ready,
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct Player {
    pub name: String,
    pub state: PlayerState,
    pub is_loser: bool,
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct GameCore {
    pub state: GameState,
    pub players: HashMap<String, Player>,
    snakes: HashMap<String, Snake>,
    pub fruit_pos: Option<MyVec2>,
}

const ENTER: char = '\x0D';
const ESCAPE: char = '\x1B';

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
            state: GameState::NotPlaying,
            snakes: HashMap::new(),
            players: HashMap::new(),
            fruit_pos: None,
        }
    }

    pub fn add_player(&mut self, name: &str, color: MyColor) {
        self.snakes.insert(name.to_string(), Snake::new(color));
        self.players.insert(
            name.to_string(),
            Player {
                is_loser: false,
                name: name.to_string(),
                state: PlayerState::NotReady,
            },
        );
    }

    pub fn remove_player(&mut self, name: &str) {
        self.snakes.remove(name);
        self.players.remove(name);
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
                if self.players.len() < PLAYER_COUNT {
                    self.state = GameState::NotPlaying;
                    return;
                }

                if self.players.iter().any(|player| {
                    return player.1.state == PlayerState::NotReady;
                }) {
                    self.state = GameState::NotPlaying;
                    return;
                }

                for (player_name, snake) in self.snakes.iter_mut() {
                    if snake.collides_self() {
                        self.state = GameState::NotPlaying;
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
            GameState::NotPlaying => {
                if self.players.len() == PLAYER_COUNT {
                    if self.players.iter().all(|player| {
                        return player.1.state == PlayerState::Ready;
                    }) {
                        self.state = GameState::Playing;
                        return;
                    }
                }
            },
        }
    }

    pub fn handle_input(&mut self, player_name: &str, c: char) {
        let player_state = &mut self.players.get_mut(player_name).unwrap().state;

        match player_state {
            PlayerState::NotReady => match c {
                ENTER => {
                    *player_state = PlayerState::Ready;
                }
                _ => {}
            },
            PlayerState::Ready => {
                let snake = self.snakes.get_mut(player_name).unwrap();
                match c {
                    'w' => snake.change_direction(Direction::Up),
                    's' => snake.change_direction(Direction::Down),
                    'a' => snake.change_direction(Direction::Left),
                    'd' => snake.change_direction(Direction::Right),
                    ESCAPE => *player_state = PlayerState::NotReady,
                    _ => {}
                }
            }
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
