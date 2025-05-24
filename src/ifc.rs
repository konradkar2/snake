use bincode::{Decode, Encode};


#[derive(Decode, Encode, Debug)]
pub enum Message
{
    JoinLobby(String),
}