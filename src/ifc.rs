
use serde::{Serialize, Deserialize};

use crate::game::game_core::GameSnapshot;

#[derive(Serialize, Deserialize, Debug)]
pub enum Message
{
    JoinLobby{player_name: String},
    Ok,
    Nok{error_msg: String},
    GameUpdate(GameSnapshot),
    SendInput(char),
}
