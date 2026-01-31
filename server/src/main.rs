use bevy::prelude::*;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use clap::Parser;

#[derive(Parser, Resource, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    /// Port to listen on
    #[arg(short, long, default_value_t = 12345)]
    port: u16,
}

#[derive(Resource)]
struct NetworkState {
    received_message: Arc<Mutex<Option<(String, String)>>>,
    socket: Arc<UdpSocket>,
}

impl Default for NetworkState {
    fn default() -> Self {
        Self {
            received_message: Arc::new(Mutex::new(None)),
            // This default is unused since we initialize in setup_network, but good to keep valid
            socket: Arc::new(UdpSocket::bind("127.0.0.1:0").unwrap()),
        }
    }
}

#[derive(Resource, Default)]
struct ServerState {
    client_addr: Option<String>,
    log: Vec<String>,
}

fn main() {
    let args = Args::parse();

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(args)
        .init_resource::<ServerState>()
        .add_systems(Startup, (setup_network, setup_ui))
        .add_systems(
            Update,
            (handle_network_messages, ping_button_system, update_log_ui),
        )
        .run();
}

fn setup_network(mut commands: Commands, args: Res<Args>) {
    let bind_addr = format!("0.0.0.0:{}", args.port);
    let socket = Arc::new(UdpSocket::bind(&bind_addr).expect("Failed to bind socket"));
    println!("Server listening on {}", bind_addr);

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

#[derive(Component)]
struct LogText;

#[derive(Component)]
struct PingButton;

fn setup_ui(mut commands: Commands, args: Res<Args>) {
    commands.spawn(Camera2dBundle::default());

    // Status Header
    commands.spawn(
        TextBundle::from_section(
            format!("Server listening on 0.0.0.0:{}", args.port),
            TextStyle {
                font_size: 20.0,
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

    // Log Area
    commands.spawn((
        TextBundle::from_section(
            "Waiting for client...\n",
            TextStyle {
                font_size: 16.0,
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
                    width: Val::Px(120.0),
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
            PingButton,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "PING",
                TextStyle {
                    font_size: 24.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));
        });
}

fn handle_network_messages(network: Res<NetworkState>, mut server_state: ResMut<ServerState>) {
    let mut received = network.received_message.lock().unwrap();
    if let Some((message, client_addr)) = received.take() {
        server_state.client_addr = Some(client_addr);
        let log_entry = format!("[Rx]: {}", message);
        server_state.log.push(log_entry);
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

fn ping_button_system(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<PingButton>)>,
    network: Res<NetworkState>,
    mut server_state: ResMut<ServerState>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            let addr = server_state.client_addr.clone();
            if let Some(addr) = addr {
                let _ = network.socket.send_to("Pong".as_bytes(), &addr);
                server_state.log.push(format!("[Tx]: Pong to {}", addr));
            } else {
                server_state
                    .log
                    .push("[Error]: No client connected".to_string());
            }
            if server_state.log.len() > 20 {
                server_state.log.remove(0);
            }
        }
    }
}
