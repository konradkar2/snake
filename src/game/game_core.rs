use std::collections::BTreeMap;

use crate::{
    common::{MyColor, MyVec2, from_color},
    snake_cfg::*,
};
use macroquad::prelude as mcq;
use rand::{Rng, rng};

use crate::snake::{Direction, Snake};

use bincode::{Decode, Encode};

use super::snake::SnakesColission;

#[derive(Encode, Decode, Debug, Clone)]

pub struct FinishDetails {
    pub draw: bool,
    pub winner: String,
}

#[derive(Encode, Decode, Debug, Clone)]
pub enum GameState {
    NotStarted,
    Paused,
    Playing,
    Finished(FinishDetails),
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
    pub players: BTreeMap<String, Player>,
    snakes: BTreeMap<String, Snake>,
    pub fruit_pos: Option<MyVec2>,
    is_server: bool,
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

pub enum PlayerColission {
    SelfColission(String),
    InBetween(SnakesColission, String, String),
    FruitColission(String),
}

impl GameCore {
    pub fn new(is_server: bool) -> Self {
        Self {
            state: GameState::NotStarted,
            snakes: BTreeMap::new(),
            players: BTreeMap::new(),
            fruit_pos: None,
            is_server,
        }
    }

    pub fn start(&mut self) {
        self.snakes.clear();

        for (index, (name, _player)) in self.players.iter().enumerate() {
            self.snakes.insert(
                name.to_string(),
                Snake::new(
                    from_color(PLAYER_COLORS[index]),
                    Self::create_player_position(index),
                ),
            );
        }
    }

    fn create_player_position(player_index: usize) -> MyVec2 {
        let x = {
            let mut block = SCREEN_WIDTH / (PLAYER_COUNT_MAX + 1) as f32;
            block = block * (player_index + 1) as f32;
            align_to_snake_size(block) as f32
        };

        let y = {
            let block = SCREEN_HEIGHT / 2.0;
            align_to_snake_size(block) as f32
        };

        MyVec2 { x, y }
    }

    pub fn add_player(&mut self, name: &str) {
        self.players.insert(
            name.to_string(),
            Player {
                name: name.to_string(),
                state: PlayerState::NotReady,
            },
        );
    }

    pub fn reset_game_state(&mut self) {
        self.state = GameState::NotStarted;
        self.fruit_pos = None;
        self.snakes.clear();
    }

    pub fn remove_player(&mut self, name: &str) {
        self.reset_game_state();
        self.players.remove(name);
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

    pub fn finish_the_game(&mut self, winner: Option<&str>) {
        for (_, player) in &mut self.players {
            player.state = PlayerState::NotReady;
        }

        let details = FinishDetails {
            draw: !winner.is_some(),
            winner: winner.unwrap_or("").to_string(),
        };
        self.state = GameState::Finished(details);
    }

    pub fn get_enemy_name(&mut self, player_name: &str) -> String {
        let enemy_name = self
            .players
            .values()
            .find(|iter_player| iter_player.name != player_name);
        return enemy_name.unwrap().name.clone();
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
                    let winner_name = self.get_enemy_name(&player_name);
                    self.finish_the_game(Some(&winner_name));
                }
                PlayerColission::InBetween(snakes_colission, _loser, winner) => {
                    match snakes_colission {
                        SnakesColission::HeadToHeadColission => {
                            self.finish_the_game(None);
                        }
                        SnakesColission::HeadToTailColission => {
                            self.finish_the_game(Some(&winner));
                        }
                    }
                }
            }
        }
    }

    pub fn update(&mut self) {
        match &self.state {
            GameState::NotStarted => {
                if self.players.iter().count() == 2
                    && self.players.iter().all(|player| {
                        return player.1.state == PlayerState::Ready;
                    })
                {
                    self.state = GameState::Playing;
                    self.start();
                    return;
                }
            }
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

                if self.is_server && self.fruit_pos.is_none() {
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
            GameState::Finished(_finish_details) => {
                if self.players.len() < PLAYER_COUNT_MAX {
                    self.state = GameState::NotStarted;
                    return;
                }

                if self.players.iter().all(|player| {
                    return player.1.state == PlayerState::Ready;
                }) {
                    self.start();
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
                    return;
                }
                _ => {}
            },
            PlayerState::Ready => {
                match c {
                    ESCAPE => {
                        *player_state = PlayerState::NotReady;
                        return;
                    }
                    _ => {}
                }

                match &self.state {
                    GameState::Playing => {
                        let snake = self.snakes.get_mut(player_name).unwrap();
                        match c {
                            'w' => snake.change_direction(Direction::Up),
                            's' => snake.change_direction(Direction::Down),
                            'a' => snake.change_direction(Direction::Left),
                            'd' => snake.change_direction(Direction::Right),
                            _ => {}
                        }
                    }
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
