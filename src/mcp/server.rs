//! rmcp streamable HTTP MCP server; tools forward to Bevy via [`super::bridge`] channels.

use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use crossbeam_channel::{Receiver as CbReceiver, Sender as CbSender};
use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
    transport::streamable_http_server::{
        StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
    },
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::bridge::{McpCmd, McpEnvelope, McpReply, PlaceTilePayload};

/// Shared channel ends used by MCP tool handlers (blocking).
#[derive(Debug, Clone)]
pub struct McpToBevyBridge {
    pub to_bevy: CbSender<McpEnvelope>,
    pub from_bevy: CbReceiver<McpReply>,
    next_id: Arc<AtomicU64>,
}

impl McpToBevyBridge {
    pub fn new(to_bevy: CbSender<McpEnvelope>, from_bevy: CbReceiver<McpReply>) -> Self {
        Self {
            to_bevy,
            from_bevy,
            next_id: Arc::new(AtomicU64::new(1)),
        }
    }

    fn call(&self, cmd: McpCmd) -> Result<serde_json::Value, String> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        self.to_bevy
            .send(McpEnvelope { id, cmd })
            .map_err(|_| "Bevy channel closed (is the editor running?)".to_string())?;
        loop {
            match self.from_bevy.recv() {
                Ok(McpReply {
                    id: rid,
                    result,
                }) if rid == id => return result,
                Ok(_) => continue,
                Err(_) => return Err("Bevy response channel closed".to_string()),
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PlaceTileArgs {
    /// World-space X before grid snap (same frame as editor crosshair).
    pub world_x: i64,
    pub world_y: i64,
    /// Spritesheet tile index.
    pub tile_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RemoveTileArgs {
    pub world_x: i64,
    pub world_y: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SetSelectedTileArgs {
    pub tile_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SetEditorStateArgs {
    /// One of: inactive, normal, load_ask, loading, loading_empty, save_ask, saving, quit_ask, edit_tile, edit_collider, edit_actor
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SetCrosshairArgs {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PlaceTileEntry {
    pub world_x: i64,
    pub world_y: i64,
    pub tile_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PlaceTilesArgs {
    /// Up to 8192 entries; same snapped cell appears once (last wins). Prefer this over per-tile HTTP round-trips.
    #[schemars(length(max = 8192))]
    pub tiles: Vec<PlaceTileEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PlaceTilePictureArgs {
    /// Bottom-left corner of the picture in world space (same frame as crosshair).
    pub origin_world_x: i64,
    pub origin_world_y: i64,
    /// `rows[0]` is the top row visually. All rows must be the same width. Charset: space `.` `_` = skip; `0`–`9` = tile 0–9; `a`–`v` or `A`–`V` = tiles 10–31.
    #[schemars(length(max = 256))]
    pub rows: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PlaceColliderArgs {
    pub world_x: i64,
    pub world_y: i64,
}

// !!!`[EDIT OR ADD TO THIS TO FINE TUNE HOW THE AI WORKS WITH THE EDITOR.]`!!!

/// Shown to MCP clients once per session — level-design contract for this editor.
pub const GAMBLER_MCP_INSTRUCTIONS: &str = r#"Gambler: Bevy 2D level editor for a playable game (not abstract art).

PLAYER SPAWN / CROSSHAIR
- **`gambler_set_crosshair` is the player spawn point** (and camera reset target): new spawns / respawns use the crosshair `Transform`.
- **Always finish a level by moving the crosshair onto solid, continuous tile+collider ground** — not over pits, decorative gaps, or empty sky. If the column under the feet has missing tiles/colliders, the player will fall through or hover over a “hole”.
- After edits, confirm the snapped cells under your chosen spawn include real floor (wide is safer than a 1-tile pillar).

COLLIDERS ↔ TILES (hard rule — no invisible walls)
- **A collider may exist only on a grid cell that already has a tile.** Empty air must never get a collider or the player walks into invisible geometry.
- `gambler_place_collider` **rejects** cells with no tile (enforced in the editor).
- Prefer **`gambler_ensure_colliders_for_all_tiles`** after laying **only** tiles that should all be solid (it adds colliders one-to-one under existing tiles). If part of your tile art is non-solid decoration, **do not** blanket-ensure — instead omit tiles there or add colliders manually only on solid cells (still requires a tile first).
- **`gambler_remove_tile` removes the collider on the same cell** so deleting art does not leave invisible walls.
- `gambler_get_snapshot`: keep **`tile_cells_missing_collider`** and **`collider_cells_without_tile`** at **0** for a shippable layout.

GAMEPLAY / PHYSICS
- Rapier uses **colliders** for contact; tiles are the visible layer. Walkable surfaces need **both** on the same cell.

TOOLS (prefer bulk)
- `gambler_place_tiles` — many placements, one round-trip; dedupes by snapped cell (last wins).
- `gambler_place_tile_picture` — ASCII art: `rows[0]` top; `origin_*` bottom-left of the picture; `0`-`9`, `a`-`v` / `A`-`V`, space/`.`/`_` skip.

LEVEL DESIGN
- Readable silhouettes, coherent materials, intentional platforms, clear routes, floating routes that still read as built for play — avoid random tile noise.

Workflow: snapshot → `gambler_request_load_empty_scene` (clears all placed `EditorObject` tiles/colliders and prior scene roots) → tiles / picture → **set crosshair on safe spawn** → `gambler_ensure_colliders_for_all_tiles` only when every placed tile should be solid → snapshot (both gap counts **0**) → save.

World coords snap to `editor_constants.grid_cell_world_px`."#;

#[derive(Debug, Clone)]
pub struct GamblerEditorMcp {
    bridge: McpToBevyBridge,
    tool_router: ToolRouter<Self>,
}

impl GamblerEditorMcp {
    pub fn new(bridge: McpToBevyBridge) -> Self {
        Self {
            bridge,
            tool_router: Self::tool_router(),
        }
    }

    fn map_err(e: String) -> String {
        serde_json::json!({ "ok": false, "error": e }).to_string()
    }

    fn map_ok(v: serde_json::Value) -> String {
        serde_json::json!({ "ok": true, "data": v }).to_string()
    }
}

#[tool_router(router = tool_router)]
impl GamblerEditorMcp {
    #[tool(
        name = "gambler_place_tile",
        description = "Place one tile at snapped grid coordinates. For large layouts prefer gambler_place_tiles or gambler_place_tile_picture (fewer round-trips). Walkable surfaces still need colliders — see server instructions."
    )]
    pub async fn place_tile(&self, Parameters(args): Parameters<PlaceTileArgs>) -> String {
        let b = self.bridge.clone();
        let args = args.clone();
        match tokio::task::spawn_blocking(move || {
            b.call(McpCmd::PlaceTile {
                world_x: args.world_x,
                world_y: args.world_y,
                tile_id: args.tile_id,
            })
        })
        .await
        {
            Ok(Ok(v)) => Self::map_ok(v),
            Ok(Err(e)) => Self::map_err(e),
            Err(e) => Self::map_err(e.to_string()),
        }
    }

    #[tool(
        name = "gambler_place_tiles",
        description = "Bulk place many tiles in one editor frame. Snaps each cell; duplicate grid cells keep the last tile_id. Max 8192 entries. Use for structured game levels instead of hundreds of gambler_place_tile calls."
    )]
    pub async fn place_tiles(&self, Parameters(args): Parameters<PlaceTilesArgs>) -> String {
        let b = self.bridge.clone();
        let tiles: Vec<PlaceTilePayload> = args
            .tiles
            .into_iter()
            .map(|e| PlaceTilePayload {
                world_x: e.world_x,
                world_y: e.world_y,
                tile_id: e.tile_id,
            })
            .collect();
        match tokio::task::spawn_blocking(move || b.call(McpCmd::PlaceTiles { tiles })).await {
            Ok(Ok(v)) => Self::map_ok(v),
            Ok(Err(e)) => Self::map_err(e),
            Err(e) => Self::map_err(e.to_string()),
        }
    }

    #[tool(
        name = "gambler_place_tile_picture",
        description = "Draw tile art from ASCII rows: rows[0] is top; origin_world_* is bottom-left of the picture; chars map to tile ids 0-31 (see server instructions). All rows same width."
    )]
    pub async fn place_tile_picture(
        &self,
        Parameters(args): Parameters<PlaceTilePictureArgs>,
    ) -> String {
        let b = self.bridge.clone();
        let origin_world_x = args.origin_world_x;
        let origin_world_y = args.origin_world_y;
        let rows = args.rows;
        match tokio::task::spawn_blocking(move || {
            b.call(McpCmd::PlaceTilePicture {
                origin_world_x,
                origin_world_y,
                rows,
            })
        })
        .await
        {
            Ok(Ok(v)) => Self::map_ok(v),
            Ok(Err(e)) => Self::map_err(e),
            Err(e) => Self::map_err(e.to_string()),
        }
    }

    #[tool(
        name = "gambler_place_collider",
        description = "Place one collider on an existing tile cell only (rejects empty cells — no invisible walls). Prefer gambler_ensure_colliders_for_all_tiles when every placed tile should be solid."
    )]
    pub async fn place_collider(&self, Parameters(args): Parameters<PlaceColliderArgs>) -> String {
        let b = self.bridge.clone();
        let world_x = args.world_x;
        let world_y = args.world_y;
        match tokio::task::spawn_blocking(move || b.call(McpCmd::PlaceCollider { world_x, world_y })).await
        {
            Ok(Ok(v)) => Self::map_ok(v),
            Ok(Err(e)) => Self::map_err(e),
            Err(e) => Self::map_err(e.to_string()),
        }
    }

    #[tool(
        name = "gambler_ensure_colliders_for_all_tiles",
        description = "Adds a collider on each cell that has a tile but no collider; never touches empty cells. Use only when every placed tile should be solid; skip if some tiles are decorative-only."
    )]
    pub async fn ensure_colliders_for_all_tiles(&self) -> String {
        let b = self.bridge.clone();
        match tokio::task::spawn_blocking(move || b.call(McpCmd::EnsureCollidersForAllTiles)).await
        {
            Ok(Ok(v)) => Self::map_ok(v),
            Ok(Err(e)) => Self::map_err(e),
            Err(e) => Self::map_err(e.to_string()),
        }
    }

    #[tool(
        name = "gambler_remove_tile",
        description = "Remove the tile at the snapped cell and also remove any collider on that cell (keeps colliders aligned with tile art)."
    )]
    pub async fn remove_tile(&self, Parameters(args): Parameters<RemoveTileArgs>) -> String {
        let b = self.bridge.clone();
        let args = args.clone();
        match tokio::task::spawn_blocking(move || {
            b.call(McpCmd::RemoveTile {
                world_x: args.world_x,
                world_y: args.world_y,
            })
        })
        .await
        {
            Ok(Ok(v)) => Self::map_ok(v),
            Ok(Err(e)) => Self::map_err(e),
            Err(e) => Self::map_err(e.to_string()),
        }
    }

    #[tool(
        name = "gambler_set_selected_tile",
        description = "Set the active spritesheet tile index used for subsequent place_tile calls."
    )]
    pub async fn set_selected_tile(
        &self,
        Parameters(args): Parameters<SetSelectedTileArgs>,
    ) -> String {
        let b = self.bridge.clone();
        let id = args.tile_id;
        match tokio::task::spawn_blocking(move || b.call(McpCmd::SetSelectedTile { tile_id: id })).await
        {
            Ok(Ok(v)) => Self::map_ok(v),
            Ok(Err(e)) => Self::map_err(e),
            Err(e) => Self::map_err(e.to_string()),
        }
    }

    #[tool(
        name = "gambler_set_editor_state",
        description = "Transition editor FSM (see SetEditorStateArgs.state schema enum in description)."
    )]
    pub async fn set_editor_state(
        &self,
        Parameters(args): Parameters<SetEditorStateArgs>,
    ) -> String {
        let b = self.bridge.clone();
        let s = args.state.clone();
        match tokio::task::spawn_blocking(move || b.call(McpCmd::SetEditorState { state: s })).await
        {
            Ok(Ok(v)) => Self::map_ok(v),
            Ok(Err(e)) => Self::map_err(e),
            Err(e) => Self::map_err(e.to_string()),
        }
    }

    #[tool(
        name = "gambler_set_crosshair",
        description = "Move crosshair to world (x, y). This is the player spawn / respawn position and camera reset target — place it on continuous solid tile ground, not over holes."
    )]
    pub async fn set_crosshair(&self, Parameters(args): Parameters<SetCrosshairArgs>) -> String {
        let b = self.bridge.clone();
        let args = args.clone();
        match tokio::task::spawn_blocking(move || {
            b.call(McpCmd::SetCrosshair { x: args.x, y: args.y })
        })
        .await
        {
            Ok(Ok(v)) => Self::map_ok(v),
            Ok(Err(e)) => Self::map_err(e),
            Err(e) => Self::map_err(e.to_string()),
        }
    }

    #[tool(
        name = "gambler_get_snapshot",
        description = "Editor state, crosshair, tiles, colliders, tile_cells_missing_collider, collider_cells_without_tile (must be 0), editor_constants. Use to verify spawn safety and no orphan colliders."
    )]
    pub async fn get_snapshot(&self) -> String {
        let b = self.bridge.clone();
        match tokio::task::spawn_blocking(move || b.call(McpCmd::GetSnapshot)).await {
            Ok(Ok(v)) => Self::map_ok(v),
            Ok(Err(e)) => Self::map_err(e),
            Err(e) => Self::map_err(e.to_string()),
        }
    }

    #[tool(
        name = "gambler_request_load_scene",
        description = "Enter Loading state (loads assets/scenes/scene.ron if present)."
    )]
    pub async fn request_load_scene(&self) -> String {
        let b = self.bridge.clone();
        match tokio::task::spawn_blocking(move || b.call(McpCmd::RequestLoadScene)).await {
            Ok(Ok(v)) => Self::map_ok(v),
            Ok(Err(e)) => Self::map_err(e),
            Err(e) => Self::map_err(e.to_string()),
        }
    }

    #[tool(
        name = "gambler_request_load_empty_scene",
        description = "Clear all placed tiles/colliders/editor objects and scene roots, then start an empty dynamic scene (prevents stacked MCP builds)."
    )]
    pub async fn request_load_empty_scene(&self) -> String {
        let b = self.bridge.clone();
        match tokio::task::spawn_blocking(move || b.call(McpCmd::RequestLoadEmptyScene)).await {
            Ok(Ok(v)) => Self::map_ok(v),
            Ok(Err(e)) => Self::map_err(e),
            Err(e) => Self::map_err(e.to_string()),
        }
    }

    #[tool(
        name = "gambler_request_save_scene",
        description = "Enter Saving state (serializes editor entities to assets/scenes/scene.ron)."
    )]
    pub async fn request_save_scene(&self) -> String {
        let b = self.bridge.clone();
        match tokio::task::spawn_blocking(move || b.call(McpCmd::RequestSaveScene)).await {
            Ok(Ok(v)) => Self::map_ok(v),
            Ok(Err(e)) => Self::map_err(e),
            Err(e) => Self::map_err(e.to_string()),
        }
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for GamblerEditorMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_instructions(GAMBLER_MCP_INSTRUCTIONS)
    }
}

/// Default MCP listen port when `GAMBLER_MCP_PORT` is unset.
pub const MCP_DEFAULT_PORT: u16 = 8080;

/// Binds `0.0.0.0:{port}` (default [`MCP_DEFAULT_PORT`]) and serves MCP at path `/mcp` until process exit.
///
/// `GAMBLER_MCP_PORT` overrides the port
pub fn spawn_mcp_streamable_http(bridge: McpToBevyBridge) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let rt = match tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(e) => {
                eprintln!("[GAMBLER_MCP] failed to create tokio runtime: {e}");
                return;
            }
        };
        rt.block_on(async move {
            let port: u16 = std::env::var("GAMBLER_MCP_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(MCP_DEFAULT_PORT);
            let bind_addr = format!("0.0.0.0:{port}");
            let listener = match tokio::net::TcpListener::bind(&bind_addr).await {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("[GAMBLER_MCP] bind {bind_addr} failed: {e}");
                    return;
                }
            };

            let bridge_for_factory = bridge.clone();
            let service = StreamableHttpService::new(
                move || Ok(GamblerEditorMcp::new(bridge_for_factory.clone())),
                Arc::new(LocalSessionManager::default()),
                StreamableHttpServerConfig::default()
                    .with_sse_keep_alive(None)
                    .disable_allowed_hosts(),
            );
            let router = axum::Router::new().nest_service("/mcp", service);
            eprintln!(
                "[GAMBLER_MCP] streamable HTTP MCP at http://127.0.0.1:{port}/mcp (listening on {bind_addr})"
            );
            if let Err(e) = axum::serve(listener, router).await {
                eprintln!("[GAMBLER_MCP] server ended: {e}");
            }
        });
    })
}
