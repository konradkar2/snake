use std::collections::HashMap;

use crate::{
    common::{MyColor, MyVec2},
    snake_cfg::*,
};
use macroquad::prelude as mcq;
use rand::{Rng, rng};

use crate::snake::{Direction, Snake};

use bincode::{Decode, Encode};

use super::snake::SnakesColission;

#[derive(Encode, Decode, Debug, Clone)]

struct WinDetails {
    winner: String,
}

pub enum GameState {
    Paused,
    Playing,
    Finished(WinDetails),
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

fn align_to_snake_size(pos: f32) -> u32 {
    let pos = pos as u32;
    let remainder = pos % SNAKE_SIZE as u32;
    pos - remainder
}

enum PlayerColission {
    SelfColission(String),
    InBetween(SnakesColission, String, String),
    FruitColission(String),
}

impl GameCore {
    pub fn new() -> Self {
        Self {
            state: GameState::Paused,
            snakes: HashMap::new(),
            players: HashMap::new(),
            fruit_pos: None,
        }
    }

    pub fn add_player(&mut self, name: &str, color: MyColor) {
        let player_count = self.players.iter().count();

        let x_start = {
            let mut block = SCREEN_WIDTH / (PLAYER_COUNT_MAX + 1) as f32;
            block = block * player_count as f32;
            align_to_snake_size(block) as f32
        };
        let y_start = {
            let mut block = SCREEN_HEIGHT / (PLAYER_COUNT_MAX + 1) as f32;
            block = block * player_count as f32;
            align_to_snake_size(block) as f32
        };

        self.snakes.insert(
            name.to_string(),
            Snake::new(color, MyVec2::new(x_start, y_start)),
        );

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
        self.state = GameState::Paused
    }

    pub fn update_fruit_pos(&mut self) {
        let mut new_fruit_pos;

        loop {
            new_fruit_pos = generate_fruit_pos();
            let mut fruit_collides = false;
            for (_, (_, snake)) in self.snakes.iter().enumerate() {
                if snake.collides_object(&new_fruit_pos) {
                    fruit_collides = true;
                }
            }
            if !fruit_collides {
                break;
            }
        }

        self.fruit_pos = Some(new_fruit_pos);
    }

    pub fn get_colissions(&self) -> Option<PlayerColission> {
        for (player_name, snake) in self.snakes.iter() {
            if snake.collides_self() {
                return Some(PlayerColission::SelfColission(player_name.clone()));
            }

            if let Some(fruit_pos) = &self.fruit_pos {
                if snake.collides_object(fruit_pos) {
                    return Some(PlayerColission::FruitColission(player_name.clone()));
                }
            }
        }

        if self.snakes.iter().count() > 2 {
            todo!("colissions for more than 2 player");
        }

        let (player1_name, player1_snake) = self.snakes.iter().nth(0).unwrap();
        let (player2_name, player2_snake) = self.snakes.iter().nth(1).unwrap();

        if let Some(colission) = player1_snake.collides_other(player2_snake) {
            return Some(PlayerColission::InBetween(
                colission,
                player1_name.clone(),
                player2_name.clone(),
            ));
        }

        if let Some(colission) = player2_snake.collides_other(player1_snake) {
            return Some(PlayerColission::InBetween(
                colission,
                player2_name.clone(),
                player1_name.clone(),
            ));
        }

        None
    }

    pub fn finish_the_game(&mut self, winner: &str) {
        for (_, player) in &mut self.players {
            player.state = PlayerState::NotReady;
        }

        self.state = GameState::Finished(WinDetails {
            winner: winner.to_string(),
        });
    }

    pub fn check_collissions(&mut self) {
        if let Some(player_colission) = self.get_colissions() {
            match player_colission {
                PlayerColission::FruitColission(player_name) => {
                    if let Some(snake) = self.snakes.get_mut(player_name.as_str()) {
                        snake.grow();
                        self.fruit_pos = None;
                    };
                }
                PlayerColission::SelfColission(player_name) => {
                    self.players.get_mut(&player_name).unwrap().is_loser = true;
                    self.finish_the_game();
                }
                PlayerColission::InBetween(snakes_colission, player_a, player_b) => {
                    match snakes_colission {
                        SnakesColission::HeadToHeadColission => {
                            self.players.get_mut(&player_a).unwrap().is_loser = true;
                            self.players.get_mut(&player_b).unwrap().is_loser = true;
                        }
                        SnakesColission::HeadToTailColission => {
                            self.players.get_mut(&player_a).unwrap().is_loser = true;
                            self.players.get_mut(&player_b).unwrap().is_loser = false;
                        }
                    }
                    self.finish_the_game();
                }
            }
        }
    }

    pub fn update(&mut self, update_fruit_pos: bool) {
        match self.state {
            GameState::Playing => {
                if self.players.iter().any(|player| {
                    return player.1.state == PlayerState::NotReady;
                }) {
                    self.state = GameState::Paused;
                    return;
                }

                self.check_collissions();

                for (_, snake) in self.snakes.iter_mut() {
                    snake.move_step_tick();
                }

                if update_fruit_pos && self.fruit_pos.is_none() {
                    self.update_fruit_pos();
                }
            }
            GameState::Paused => {
                if self.players.len() == PLAYER_COUNT_MAX {
                    if self.players.iter().all(|player| {
                        return player.1.state == PlayerState::Ready;
                    }) {
                        self.state = GameState::Playing;
                        return;
                    }
                }
            }
            GameState::Finished => {
                if self.players.len() < PLAYER_COUNT_MAX {
                    self.state = GameState::Paused;
                    return;
                }

                if self.players.iter().all(|player| {
                    return player.1.state == PlayerState::Ready;
                }) {
                    self.state = GameState::Playing;
                    return;
                }
            }
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
