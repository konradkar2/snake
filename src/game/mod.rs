pub mod game_core;
use crate::{game_core::*, snake_cfg::PLAYER_COUNT};
use macroquad::prelude as mcq;
pub mod snake;
use crate::common::from_color;

#[derive(Debug)]
pub struct GameLocal {
    pub game_core: GameCore,
    is_multiplayer: bool,
    player_name: String,
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
        50.0,
        color,
    );
}

impl GameLocal {
    pub fn new(is_multiplayer: bool, player_name: &str) -> Self {
        let mut game_core = GameCore::new();
        game_core.add_player(player_name, from_color(mcq::GREEN));

        Self {
            game_core: game_core,
            is_multiplayer: is_multiplayer,
            player_name: player_name.to_string(),
        }
    }

    pub fn update(&mut self) {
        self.game_core.update(true);
    }

    pub fn handle_input(&mut self, c: char) {
        self.game_core.handle_input(&self.player_name, c);
    }

    pub fn draw(&self) {
        mcq::clear_background(mcq::RED);
        match self.game_core.state {
            GameState::Paused => {
                mcq::draw_rectangle(
                    0.0,
                    0.0,
                    mcq::screen_width(),
                    mcq::screen_height(),
                    mcq::BLUE,
                );
                self.game_core.draw_objects();

                let mut text = String::new();

                let players = &self.game_core.players;
                if players.len() != PLAYER_COUNT {
                    text = String::from("Waiting for all players");
                } else {
                    let mut player_names: Vec<String> = players.keys().cloned().collect();
                    player_names.sort_unstable();
                    
                    for player_name in &player_names {
                        let player = &players[player_name];
                        let state_str = {
                            if player.state == PlayerState::NotReady {
                                "not ready"
                            } else {
                                "ready"
                            }
                        };

                        text += format!("player {}: {}\n", player.name, state_str).as_str();
                    }
                }
                mcq::draw_multiline_text(&text, 20.0, 100.0, 30.0, None, mcq::RED);
            },
            GameState::Finished => {
                mcq::draw_rectangle(
                    0.0,
                    0.0,
                    mcq::screen_width(),
                    mcq::screen_height(),
                    mcq::GOLD,
                );
            }
            _ => self.game_core.draw_objects(),
        }
        mcq::draw_fps();
    }
}
