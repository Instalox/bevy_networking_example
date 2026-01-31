//! Knock Knock Server
//! Usage: cargo run --bin knock_server -- --port 50051
//!
//! Listens for "KNOCK KNOCK" messages and replies "WHO IS THERE?"

use bevy::prelude::*;
use clap::Parser;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Parser, Resource, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    /// Port to listen on
    #[arg(short, long, default_value_t = 50051)]
    port: u16,
}

#[derive(Resource)]
struct NetworkState {
    /// (Message, SenderAddress)
    received_message: Arc<Mutex<Option<(String, String)>>>,
    socket: Arc<UdpSocket>,
}

#[derive(Resource, Default)]
struct ServerState {
    log: Vec<String>,
}

#[derive(Component)]
struct LogText;

fn main() {
    let args = Args::parse();

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(args)
        .init_resource::<ServerState>()
        .add_systems(Startup, (setup_network, setup_ui))
        .add_systems(Update, (handle_network_messages, update_log_ui))
        .run();
}

fn setup_network(mut commands: Commands, args: Res<Args>) {
    let bind_addr = format!("0.0.0.0:{}", args.port);
    let socket = Arc::new(UdpSocket::bind(&bind_addr).expect("Failed to bind socket"));
    println!("Knock Knock Server listening on {}", bind_addr);

    socket
        .set_nonblocking(true)
        .expect("Failed to set non-blocking");

    let received_message = Arc::new(Mutex::new(None));
    let socket_clone = socket.clone();
    let received_clone = received_message.clone();

    thread::spawn(move || {
        let mut buf = [0u8; 1024];
        loop {
            match socket_clone.recv_from(&mut buf) {
                Ok((size, addr)) => {
                    let message = String::from_utf8_lossy(&buf[..size]).to_string();
                    let mut received = received_clone.lock().unwrap();
                    *received = Some((message, addr.to_string()));
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(_) => break,
            }
        }
    });

    commands.insert_resource(NetworkState {
        received_message,
        socket,
    });
}

fn setup_ui(mut commands: Commands, args: Res<Args>) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn(
        TextBundle::from_section(
            format!("Knock Knock Server - Listening on port {}", args.port),
            TextStyle {
                font_size: 24.0,
                color: Color::rgb(0.9, 0.9, 0.9),
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
    );

    commands.spawn((
        TextBundle::from_section(
            "Waiting for KNOCK KNOCK...\n",
            TextStyle {
                font_size: 18.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(50.0),
            left: Val::Px(10.0),
            ..default()
        }),
        LogText,
    ));
}

fn handle_network_messages(network: Res<NetworkState>, mut server_state: ResMut<ServerState>) {
    let mut received = network.received_message.lock().unwrap();
    if let Some((message, client_addr)) = received.take() {
        // Log what we received
        server_state
            .log
            .push(format!("[Rx from {}]: {}", client_addr, message.trim()));

        // Reply: "WHO IS THERE?"
        let reply = b"WHO IS THERE?";
        let _ = network.socket.send_to(reply, &client_addr);
        server_state
            .log
            .push(format!("[Tx to {}]: WHO IS THERE?", client_addr));

        // Keep log length manageable
        if server_state.log.len() > 20 {
            server_state.log.remove(0);
        }
    }
}

fn update_log_ui(server_state: Res<ServerState>, mut query: Query<&mut Text, With<LogText>>) {
    if server_state.is_changed() {
        for mut text in query.iter_mut() {
            text.sections[0].value = server_state.log.join("\n");
        }
    }
}
