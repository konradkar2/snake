# Snake

A simple multiplayer Snake game.

![Alt text](/screenshots/gameplay.png "gameplay")


The game uses a client-server setup. The server owns the main game state, handles player connections, updates the game logic, and sends game snapshots to connected clients. Each client renders the game locally and sends player input back to the server.

## Running The Game

Start the server first:

```bash
cargo run --bin snake_server -- <address:port>
```

By default, the server listens on:

```text
0.0.0.0:6969
```

Then start a client

```bash
cargo run --bin snake_client -- <nickname> <server_address:port>
```

Example:

```bash
cargo run --bin snake_client -- Alice 127.0.0.1:6969
```

Start a second client

```bash
cargo run --bin snake_client -- Bob 127.0.0.1:6969
```

The game starts when two players are connected and ready.

## Controls

```text
W     move up
S     move down
A     move left
D     move right
Enter set player ready
Esc   set player not ready (pauses the game)
```

## Tests

Run the test suite with:

```bash
cargo test
```
