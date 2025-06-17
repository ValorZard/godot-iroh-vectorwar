use game_core::server::run_server;
use godot::{classes::{Button, IButton}, prelude::*};

use crate::async_runtime::AsyncRuntime;

#[derive(GodotClass)]
#[class(base=Button)]
struct ServerButton {
    channel_map: Option<game_core::server::ChannelMap>,
    base: Base<Button>,
}

#[godot_api]
impl IButton for ServerButton {
    fn init(base: Base<Button>) -> Self {
        Self {
            channel_map: None, // Initialize with None, will be set when the server starts
            base,
        }
    }

    fn ready(&mut self) {
        godot_print!("Server button is ready!");
    }

    fn pressed(&mut self) {
        godot_print!("Server button pressed!");
        // we are going to shove this in here for now for testing purposes
        if let Ok(channel_map) = AsyncRuntime::block_on(run_server()) {
            godot_print!("server running");
            self.channel_map = Some(channel_map);
        } else {
            godot_print!("failed to run server");
        }
    }

    fn process(&mut self, delta: f64) {
        // This is where you can handle any server-related logic
        // For example, you might want to check for incoming connections or messages
        if let Some(channel_map) = &self.channel_map {
            // Handle server logic with the channel_map
            let channel_map = channel_map.lock().unwrap();
            for (player_id, channel) in channel_map.iter() {
                match channel.receiver.try_recv() {
                    Ok(message) => {
                        godot_print!("Received message from player {}: {:?}", player_id, message);
                        // Handle the received message
                    }
                    Err(async_channel::TryRecvError::Empty) => {
                        // No messages available, continue processing
                    }
                    Err(async_channel::TryRecvError::Closed) => {
                        godot_print!("Channel for player {} closed", player_id);
                        // Handle the closed channel if necessary
                    }
                }
            }
        }
    }
}