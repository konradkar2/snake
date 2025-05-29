use std::array;

use bincode::{Decode, Encode};

use crate::game::game_core::GameCore;

#[derive(Decode, Encode, Debug)]
struct Padding
{
    pad: [i8; 1024],
}

#[derive(Decode, Encode, Debug)]
pub enum Message
{
    JoinLobby(String),
    Ok,
    Nok(String),
    GameUpdate(GameCore),
    SendInput(char),
    NoUsePadding(Padding),
}