//! Knock Knock Client
//! Usage: cargo run --bin knock_client -- --server 127.0.0.1:50051
//!
//! Sends "KNOCK KNOCK" to server and waits for "WHO IS THERE?" reply.

use bevy::prelude::*;
use clap::Parser;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Parser, Resource, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    /// Server address to connect to
    #[arg(short, long, default_value = "127.0.0.1:50051")]
    server: String,
}

#[derive(Resource)]
struct NetworkState {
    received_message: Arc<Mutex<Option<String>>>,
    socket: Arc<UdpSocket>,
}

#[derive(Resource, Default)]
struct ClientState {
    log: Vec<String>,
}

#[derive(Component)]
struct LogText;

#[derive(Component)]
struct KnockButton;

fn main() {
    let args = Args::parse();

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(args)
        .init_resource::<ClientState>()
        .add_systems(Startup, (setup_network, setup_ui))
        .add_systems(
            Update,
            (handle_network_messages, knock_button_system, update_log_ui),
        )
        .run();
}

fn setup_network(mut commands: Commands) {
    let socket = Arc::new(UdpSocket::bind("0.0.0.0:0").expect("Failed to bind socket"));
    println!(
        "Knock Knock Client bound to {}",
        socket.local_addr().unwrap()
    );

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
                Ok((size, _addr)) => {
                    let message = String::from_utf8_lossy(&buf[..size]).to_string();
                    let mut received = received_clone.lock().unwrap();
                    *received = Some(message);
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
            format!("Knock Knock Client -> {}", args.server),
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
            "Click KNOCK to send message...\n",
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

    commands
        .spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(150.0),
                    height: Val::Px(50.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(20.0),
                    right: Val::Px(20.0),
                    ..default()
                },
                background_color: Color::rgb(0.3, 0.5, 0.9).into(),
                ..default()
            },
            KnockButton,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "KNOCK KNOCK",
                TextStyle {
                    font_size: 20.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));
        });
}

fn handle_network_messages(network: Res<NetworkState>, mut client_state: ResMut<ClientState>) {
    let mut received = network.received_message.lock().unwrap();
    if let Some(message) = received.take() {
        client_state.log.push(format!("[Rx]: {}", message.trim()));
        if client_state.log.len() > 20 {
            client_state.log.remove(0);
        }
    }
}

fn update_log_ui(client_state: Res<ClientState>, mut query: Query<&mut Text, With<LogText>>) {
    if client_state.is_changed() {
        for mut text in query.iter_mut() {
            text.sections[0].value = client_state.log.join("\n");
        }
    }
}

fn knock_button_system(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<KnockButton>)>,
    network: Res<NetworkState>,
    args: Res<Args>,
    mut client_state: ResMut<ClientState>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            let message = b"KNOCK KNOCK";
            let _ = network.socket.send_to(message, &args.server);
            client_state
                .log
                .push(format!("[Tx]: KNOCK KNOCK -> {}", args.server));
            if client_state.log.len() > 20 {
                client_state.log.remove(0);
            }
        }
    }
}
