
use serde::{Serialize, Deserialize};

use crate::game::game_core::GameCore;

#[derive(Serialize, Deserialize, Debug)]
pub enum Message
{
    JoinLobby{player_name: String},
    Ok,
    Nok{error_msg: String},
    GameUpdate(GameCore),
    SendInput(char),
}