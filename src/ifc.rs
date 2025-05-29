use bincode::{Decode, Encode};

use crate::game::game_core::GameCore;



#[derive(Decode, Encode, Debug)]
pub enum Message
{
    JoinLobby(String),
    Ok,
    Nok(String),
    GameUpdate(GameCore),
    SendInput(char)
}