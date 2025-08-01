pub mod game_core;
use crate::{game_core::*, snake_cfg::PLAYER_COUNT_MAX};
use macroquad::{color::Color, prelude as mcq};
pub mod snake;
use crate::common::from_color;

#[derive(Debug)]
pub struct GameLocal {
    pub game_core: GameCore,
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
    pub fn new(player_name: &str) -> Self {
        let mut game_core = GameCore::new(false);
        game_core.add_player(player_name);

        Self {
            game_core: game_core,
            player_name: player_name.to_string(),
        }
    }

    pub fn update(&mut self) {
        self.game_core.update();
    }

    pub fn handle_input(&mut self, c: char) {
        self.game_core.handle_input(&self.player_name, c);
    }

    pub fn get_players_status_text(&self) -> String {
        let mut text = String::new();
        for (_, player) in &self.game_core.players {
            let state_str = {
                if player.state == PlayerState::NotReady {
                    "not ready"
                } else {
                    "ready"
                }
            };

            text += format!("player {}: {}\n", player.name, state_str).as_str();
        }
        text
    }

    pub fn draw(&self) {
        mcq::clear_background(mcq::RED);
        match &self.game_core.state {
            GameState::NotStarted | GameState::Paused => {
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
                if players.len() != PLAYER_COUNT_MAX {
                    text = String::from("Waiting for all players\n");
                }
                text = text + &self.get_players_status_text();

                mcq::draw_multiline_text(&text, 20.0, 100.0, 30.0, None, mcq::RED);
            }
            GameState::Finished(finish_details) => {
                let background_color: Color;
                let mut game_status_text = String::new();

                if finish_details.draw {
                    game_status_text += "It's a draw!\n";
                    background_color = mcq::DARKGRAY;
                } else {
                    if finish_details.winner == self.player_name {
                        game_status_text += format!("You win!\n").as_str();
                        background_color = mcq::DARKBLUE;
                    } else {
                        game_status_text +=
                            format!("Player {} won!\n", finish_details.winner).as_str();
                        background_color = mcq::RED;
                    }
                }

                let player_status_text = self.get_players_status_text();

                mcq::draw_rectangle(
                    0.0,
                    0.0,
                    mcq::screen_width(),
                    mcq::screen_height(),
                    background_color,
                );
                mcq::draw_multiline_text(&game_status_text, 20.0, 100.0, 30.0, None, mcq::BLACK);
                mcq::draw_multiline_text(&player_status_text, 20.0, 300.0, 30.0, None, mcq::BLACK);
                self.game_core.draw_objects();
            }
            _ => self.game_core.draw_objects(),
        }
        mcq::draw_fps();
    }
}
