use bevy::prelude::*;
use bevy_egui::{
    egui::{self, RichText},
    EguiContext, EguiPlugin,
};
use futures_util::StreamExt;
use std::collections::HashMap;
use tokio::sync::mpsc;

#[derive(Component)]
struct PriceWindow {
    id: String,
    pos: egui::Pos2,
    opened: bool,
}

#[derive(Resource)]
struct WebSocketConnections {
    connections: HashMap<String, mpsc::Sender<()>>,
}

#[derive(Resource, Default)]
struct PriceUpdates {
    prices: HashMap<String, f64>,
}

// Create a resource to hold the runtime handle
#[derive(Resource)]
struct RuntimeHandle {
    runtime: tokio::runtime::Handle,
}

fn main() {
    // Create the runtime first
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let runtime_handle = runtime.handle().clone();

    // Spawn the Bevy app on the runtime
    runtime.block_on(async {
        App::new()
            .add_plugins(DefaultPlugins)
            .add_plugins(EguiPlugin)
            .insert_resource(WebSocketConnections {
                connections: HashMap::new(),
            })
            .insert_resource(ClearColor(Color::srgb_u8(18, 25, 29)))
            .insert_resource(PriceUpdates::default())
            .insert_resource(RuntimeHandle {
                runtime: runtime_handle,
            })
            .add_systems(Startup, setup)
            .add_systems(Update, window_system)
            .add_systems(Update, handle_window_spawn)
            .run();
    });
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d::default());
}

fn window_system(
    mut egui_ctx: Query<&mut EguiContext>,
    mut windows: Query<(Entity, &mut PriceWindow)>,
    mut commands: Commands,
    price_updates: Res<PriceUpdates>,
    mut websocket_connections: ResMut<WebSocketConnections>,
) {
    let mut ctx = egui_ctx.single_mut();

    for (entity, mut window) in windows.iter_mut() {
        if !window.opened {
            continue;
        }

        let window_id = window.id.clone();
        let mut should_close = false;

        egui::Window::new(&window_id)
            .default_pos(window.pos)
            .title_bar(false)
            .resizable(true)
            .frame(egui::Frame {
                inner_margin: egui::Margin::same(8.0),
                fill: egui::Color32::from_rgb(18, 25, 29), // Background color
                stroke: egui::Stroke::new(2.5, egui::Color32::from_rgb(10, 17, 21)), // Border
                ..Default::default()
            })
            .collapsible(false)
            .show(ctx.get_mut(), |ui| {
                egui::TopBottomPanel::top(format!("{}-top-bottom-panel", &window_id))
                    .frame(egui::Frame {
                        fill: egui::Color32::from_rgb(18, 25, 29), // Background color
                        inner_margin: egui::Margin {
                            bottom: 8.0,
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .show_inside(ui, |_ui| {
                        _ui.horizontal(|__ui| {
                            __ui.spacing_mut().item_spacing.x = 10.0; // Space between elements

                            // Title on the left
                            __ui.label(
                                egui::RichText::new("Orderbook")
                                    .color(egui::Color32::from_rgb(255, 255, 255)),
                            );

                            // Push close button to the right
                            __ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.add_sized([10.0, 10.0], egui::Label::new("X")).clicked() {
                                        should_close = true;
                                    }
                                    // if ui.button("❌").clicked() {
                                    //     should_close = true;
                                    // }
                                },
                            );
                        });
                    });

                ui.collapsing("Settings", |contents| {
                    contents.label(format!("Bichoso. {}", 1));
                });

                ui.label(format!(
                    "Price: {}",
                    price_updates.prices.get(&window_id).unwrap_or(&0.0)
                ));

                if ui.button("Close").clicked() {
                    should_close = true;
                }

                window.pos = ui.ctx().used_rect().left_top();
            });

        if should_close {
            window.opened = false;
            if let Some(sender) = websocket_connections.connections.remove(&window_id) {
                let _ = sender.try_send(());
            }
            commands.entity(entity).despawn();
        }
    }
}

fn handle_window_spawn(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut websocket_connections: ResMut<WebSocketConnections>,
    runtime_handle: Res<RuntimeHandle>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        spawn_price_window(&mut commands, &mut websocket_connections, &runtime_handle);
    }
}

fn spawn_price_window(
    commands: &mut Commands,
    websocket_connections: &mut ResMut<WebSocketConnections>,
    runtime_handle: &RuntimeHandle,
) {
    let window_id = format!("BTCUSDT-{}", websocket_connections.connections.len());
    let (tx, mut rx) = mpsc::channel(8);

    websocket_connections
        .connections
        .insert(window_id.clone(), tx);

    let _window_id = window_id.clone();

    // Spawn the WebSocket connection using the runtime handle
    runtime_handle.runtime.spawn(async move {
        // let url = "wss://stream.binance.com:9443/ws/btcusdt@trade";

        // let (ws_stream, _) = tokio_tungstenite::connect_async(url).await.unwrap();
        // let (_, mut read) = ws_stream.split();

        // loop {
        //     tokio::select! {
        //         msg = read.next() => {
        //             match msg {
        //                 Some(Ok(msg)) => {
        //                     println!("Received: {}", msg);
        //                 }
        //                 _ => break,
        //             }
        //         }
        //         _ = rx.recv() => {
        //             println!("Closing connection for {}", _window_id);
        //             break;
        //         }
        //     }
        // }
    });

    commands.spawn(PriceWindow {
        id: window_id,
        pos: egui::pos2(100.0, 100.0),
        opened: true,
    });
}
