# Tutorial & FAQ: Bevy Networking

This guide covers the implementation of a simple UDP client-server game in Bevy, and specifically answers common questions about **performance, file size, and why we use Bevy over standard Rust**.

## 1. Why is the project "Heavy"? (File Size)

You might notice the project folder is large (e.g., 4GB). **This is temporary.**
Rust compiles every single dependency from source code to ensure maximum speed. These intermediate files live in the `target/` folder.

- **Development Build (Huge)**: `cargo run` creates debug symbols, incremental compilation files, and artifacts. This folder can grow to gigabytes. **You can verify this by deleting the `target/` folder—the actual source code is tiny.**
- **Release Build (Tiny)**: When shipping to a user, you run `cargo build --release`.
    - We have configured `Cargo.toml` to strip unnecessary data.
    - The final executable will be **a few megabytes**, not gigabytes.

### Optimization Tip
We added this to `Cargo.toml` to automatically optimize your release builds:
```toml
[profile.release]
opt-level = "z"   # Optimize for small binary size
lto = true        # Link Time Optimization (removes unused code)
strip = true      # Removes debug symbols
```

## 2. Why use Bevy if Standard Rust works?

You correctly pointed out that we are using `std::net::UdpSocket`. So why add Bevy?

- **Standard Rust**: Great for sending bytes. Bad for drawing windows, playing sounds, and handling input.
- **Bevy**: Adds the "Game" part.
    - **Visual Debugging**: You can see "Tx: Ping" and "Rx: Pong" on screen immediately.
    - **State Management**: Bevy's ECS (Entity Component System) handles the complex game state (players, health, positions) much better than a giant `main.rs` file.
    - **Cross-Platform**: Run the same code on Windows, Linux, Mac, and Web (WASM).

## 3. "Where are the Bevy connection features?"

You asked about Authentication, Encryption, and Connection handling (Renet, Lightyear, etc.).

**This tutorial shows the Foundation.**
Before using a complex library like `bevy_renet` (which *does* handle encryption/auth), you must understand what it is doing for you.

- **Current Project**: Raw UDP. "Here is a packet." (Fast, simple, good for learning).
- **Bevy Renet / Lightyear**: "Here is an encrypted, authenticated connection with reliability." (Complex, black box).

**We built this to be transparent.** You can *see* the socket. You can *see* the thread. Once you understand this, switching to `bevy_renet` is just swapping the `UdpSocket` for a `RenetClient`.

## 4. How the Code Works (The "Knock Knock" Logic)

### The Server
1.  **Starts**: Binds to `0.0.0.0:12345`.
2.  **Listens**: A background thread waits for packets.
3.  **Responds**: When it hears "Ping", it sends "Pong".

```rust
// server/src/main.rs - The "Connection" logic
thread::spawn(move || {
    loop {
        // Wait for "Knock Knock"
        if let Ok((_, addr)) = socket.recv_from(&mut buf) {
             // Store who knocked so we can answer later
             *received.lock().unwrap() = Some(addr);
        }
    }
});
```

### The Client
1.  **Connects**: Sends a packet to the server address.
2.  **Waits**: A background thread waits for the "Pong".

## 5. Analyzing Your Code (client.rs)

I looked at the `client.rs` you provided. It contains **two completely different networking systems side-by-side**, which is why it is confusing!

1.  **The Top Part (Lines 1-80)**: This is "Standard Rust Networking".
    - Uses `std::net::UdpSocket`.
    - Manually handles `socket.send_to()` and `socket.recv_from()`.
    - This is exactly what **this tutorial** teaches you to build first.

2.  **The Bottom Part (Lines 82+)**: This is "Bevy Renet Networking".
    - Uses `RenetClient` and plugins.
    - Handles encryption, authentication, and reliable channels for you.
    - This is the "advanced" library method.

**The Confusion**: You have both "Knock Knock" (Standard) and "Renet Connection" (Advanced) in the same file.
**The Solution**: Follow this tutorial to build the "Standard" version first. Once you understand `socket.send_to()`, you will understand what `RenetClient` is doing for you (wrapping that socket with extra features).

## 6. The "Knock Knock" Example (`knock_knock/`)

We created a working implementation of your "Knock Knock" / "Who Is There?" code in the `knock_knock/` folder!

**Run it:**
```bash
# Terminal 1: Start Server
cargo run --bin knock_server

# Terminal 2: Start Client
cargo run --bin knock_client
```

Click "KNOCK KNOCK" in the client window. Watch the server log "KNOCK KNOCK" and reply "WHO IS THERE?".

This proves your networking concept works. The Bevy part just adds the visual window and buttons.

## Next Steps

To add **Authentication and Encryption**, graduate from this "Raw UDP" setup to **`bevy_renet`**.
But establishing this base proves your networking layer works fundamentally before adding complexity.
