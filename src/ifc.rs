use std::array;

use serde::{Serialize, Deserialize};

use crate::game::game_core::GameCore;

#[derive(Serialize, Deserialize, Debug)]
pub enum Message
{
    JoinLobby(String),
    Ok,
    Nok(String),
    GameUpdate(GameCore),
    SendInput(char),
}