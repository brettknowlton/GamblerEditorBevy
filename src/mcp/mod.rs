//! MCP bridge: rmcp **streamable HTTP** server ↔ Bevy via crossbeam channels.
//!
//! The MCP endpoint is **`http://127.0.0.1:8080/mcp`**
//! Override the port with `GAMBLER_MCP_PORT`.
//!
//! Bulk tools (`gambler_place_tiles`, `gambler_place_tile_picture`) and collider helpers live here so
//! clients do not need external scripts for large layouts. See `GAMBLER_MCP_INSTRUCTIONS` (re-exported below).

mod bridge;
mod server;
mod systems;

pub use bridge::{McpCmd, McpEnvelope, McpReply, PlaceTilePayload};
pub use server::{
    GamblerEditorMcp, GAMBLER_MCP_INSTRUCTIONS, MCP_DEFAULT_PORT, McpToBevyBridge, PlaceColliderArgs,
    RemoveColliderArgs, FillRectArgs, PlaceActorArgs, RemoveActorArgs,
    SetPixelArtSettingsArgs,
    PlaceTileArgs, PlaceTileEntry, PlaceTilePictureArgs, PlaceTilesArgs, RemoveTileArgs,
    SetCrosshairArgs, SetEditorStateArgs, SetSelectedTileArgs, spawn_mcp_streamable_http,
};
pub use systems::MCP_MAX_PLACE_TILES;

use bevy::prelude::*;
use crossbeam_channel::{Receiver, Sender};

/// When present, MCP is active: drain [`McpEnvelope`] each frame and reply on [`McpReply`].
#[derive(Resource)]
pub struct EditorMcpBridge {
    pub from_mcp: Receiver<McpEnvelope>,
    pub to_mcp: Sender<McpReply>,
}

pub struct McpPlugin;

impl Plugin for McpPlugin {
    fn build(&self, app: &mut App) {

        let (tx_cmd, rx_cmd) = crossbeam_channel::unbounded();
        let (tx_resp, rx_resp) = crossbeam_channel::unbounded();

        app.insert_resource(EditorMcpBridge {
            from_mcp: rx_cmd,
            to_mcp: tx_resp,
        });

        let bridge = McpToBevyBridge::new(tx_cmd, rx_resp);
        let handle = spawn_mcp_streamable_http(bridge);
        std::mem::forget(handle);

        app.add_systems(Update, systems::drain_mcp_inbox);
    }
}
