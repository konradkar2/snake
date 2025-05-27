use crate::multiplayer::server::*;
use crate::multiplayer::client::*;
pub mod server;
pub mod client;


enum Role {
    Server,
    Client,
}

struct Multiplayer
{
    role: Role,
    server: Option<ServerHandle>,
    client: Client,
}