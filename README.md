# Bevy Networking Example

A simple UDP client-server networking demonstration using the [Bevy game engine](https://bevyengine.org/).

## Overview

This project contains three separate applications:

- **Server** (`server/`): Listens for "Ping" messages and responds with "Pong". Displays a scrolling log.
- **Client** (`client/`): Connects to the server, sends "Ping" messages, receives responses. Displays a scrolling log.
- **Knock Knock** (`knock_knock/`): A simplified "Knock Knock" / "Who Is There?" example that mirrors raw UDP networking (created based on client reference code).

All applications render their activity in a graphical window with UI feedback.

## Project Structure

```
bevy_networking_example/
├── README.md
├── TUTORIAL.md
├── Cargo.toml                    # Workspace configuration
├── server/
│   ├── Cargo.toml
│   └── src/main.rs              # Ping/Pong Server
├── client/
│   ├── Cargo.toml
│   └── src/main.rs              # Ping/Pong Client
└── knock_knock/
    ├── Cargo.toml
    ├── src/server.rs            # "Who Is There?" Server
    └── src/client.rs            # "Knock Knock" Client
```

## Requirements

- Rust 2024 edition (Rust 1.82+ for 2024 edition support)
- A GPU capable of Vulkan (for Bevy's renderer)

## Building & Running

You can run both apps from the root directory using `cargo run`.

### 1. Start the Server

By default, the server listens on port `12345`.

```bash
cargo run -p server
```

**Custom Port**:
You can specify a custom port using the `--port` argument:

```bash
cargo run -p server -- --port 5000
```

### 2. Start the Client

By default, the client connects to `127.0.0.1:12345`.

```bash
cargo run -p client
```

**Custom Connection**:
You can specify the server address and your local binding port:

```bash
cargo run -p client -- --server 127.0.0.1:5000 --port 0
```
*(Port 0 means "bind to any random available port")*

---

### 3. Knock Knock Example

This is a simplified example based on client-provided reference code. It demonstrates "KNOCK KNOCK" / "WHO IS THERE?" messaging.

**Start the Knock Knock Server** (listens on port 50051 by default):
```bash
cargo run --bin knock_server
```

**Start the Knock Knock Client** (connects to 127.0.0.1:50051 by default):
```bash
cargo run --bin knock_client
```

Click the "KNOCK KNOCK" button in the client. The server receives it and replies "WHO IS THERE?".

## How It Works

### Server Flow

1.  **Setup Phase** (`setup_network`):
    - Parses CLI args for port number.
    - Creates a UDP socket bound to `0.0.0.0:PORT`.
    - Spawns a background thread that continuously receives messages.
    - Stores received messages in a thread-safe queue (`Arc<Mutex<...>>`).

2.  **Update Phase** (`handle_network_messages`):
    - Runs every frame.
    - Checks for new messages from the background thread.
    - Updates the scrolling log UI.

3.  **Interaction**:
    - Clicking "PING" sends a "Pong" back to the last known client address.

### Client Flow

1.  **Setup Phase** (`setup_network`):
    - Parses CLI args for server address and local port.
    - Creates a UDP socket.
    - Spawns a background thread to receive responses.

2.  **Update Phase**:
    - `handle_network_messages`: Receives "Pong" messages and updates the log.

3.  **Interaction**:
    - Clicking "PING" sends a "Ping" packet to the server and logs the transmission.

## Key Concepts

### Resources
Resources are global state containers in Bevy. They're perfect for storing configuration, counters, or shared state.

### Thread Safety with Arc<Mutex<T>>
Since we have a background thread receiving network data while the main Bevy thread processes it, we use:
- **Arc** (Atomic Reference Count): Allows multiple owners to share data safely.
- **Mutex**: Ensures only one thread can access the data at a time.

### CLI Arguments
We use `clap` to parse command line arguments, making it easy to configure network addresses without recompiling.

## Troubleshooting

### No messages appearing
1.  Ensure Server is running.
2.  If running on valid separate machines, ensure you use the LAN IP (e.g., `192.168.1.5`) instead of `127.0.0.1`.
3.  Check firewalls for UDP port `12345` (or your custom port).

## Dependencies

- `bevy` 0.13 - Game engine
- `crossbeam` 0.8 - Thread-safe primitives
- `clap` - Command line argument parsing

## License

MIT
