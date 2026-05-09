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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RemoveColliderArgs {
    pub world_x: i64,
    pub world_y: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FillRectArgs {
    /// World-space X of one corner.
    pub x1: i64,
    /// World-space Y of one corner.
    pub y1: i64,
    /// World-space X of the opposite corner.
    pub x2: i64,
    /// World-space Y of the opposite corner.
    pub y2: i64,
    pub tile_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PlaceActorArgs {
    pub world_x: i64,
    pub world_y: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RemoveActorArgs {
    pub world_x: i64,
    pub world_y: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SetPixelArtSettingsArgs {
    /// one of: tile, player, both
    pub target: String,
    pub pixel_size: Option<f32>,
    pub color_levels: Option<f32>,
    pub dither_strength: Option<f32>,
    pub scanline_strength: Option<f32>,
    pub palette_enabled: Option<f32>,
}

// !!!`[EDIT OR ADD TO THIS TO FINE TUNE HOW THE AI WORKS WITH THE EDITOR.]`!!!

/// Shown to MCP clients once per session — level-design contract for this editor.
pub const GAMBLER_MCP_INSTRUCTIONS: &str = r#"
Gambler: Bevy 2D level editor for a metroidvania game. (not abstract art).

HARD ENFORCEMENTS (verified in MCP handlers)
- World inputs are snapped to the grid for place/remove tile, collider, actor, and fill rect operations.
- gambler_place_collider rejects cells that do not currently contain a tile.
- gambler_remove_tile removes both tile and collider at that snapped cell.
- gambler_ensure_colliders_for_all_tiles adds colliders only to tile cells missing colliders.
- gambler_place_tiles and gambler_place_tile_picture reject oversized batches (max 8192 logical cells).
- gambler_place_tiles deduplicates by snapped cell (last write wins).
- gambler_request_load_empty_scene switches to LoadingEmpty, and the scene system clears all EditorObject entities and scene roots.

SPAWN / CROSSHAIR
- gambler_set_crosshair controls spawn and reset target behavior because player spawn/respawn reads crosshair transform.
- Treat crosshair placement as gameplay-critical: place it on solid, continuous tile+collider ground.
- Prefer 2+ tiles of support under spawn instead of single-column perches.

COLLIDERS AND GAMEPLAY
- Physics contact is collider-driven; visible tiles alone are not walkable.
- For fully solid layouts: place tiles, then run gambler_ensure_colliders_for_all_tiles.
- For mixed solid/decorative layouts: place colliders explicitly only where solid traversal is intended.
- Validate with gambler_get_snapshot and keep tile_cells_missing_collider=0 and collider_cells_without_tile=0 before shipping.

TOOLING NOTES
- Prefer gambler_place_tiles for large structured edits to reduce round-trips.
- gambler_place_tile_picture charset: space . _ = skip; 0-9 = tile ids 0-9; a-v / A-V = tile ids 10-31.
- rows[0] in picture mode is the visual top row; origin is bottom-left in world space.
- World step size equals editor_constants.grid_cell_world_px.
- gambler_get_pixel_art_settings / gambler_set_pixel_art_settings allow runtime shader tuning.

RECOMMENDED WORKFLOW
1) gambler_get_snapshot (or gambler_get_scene_bounds)
2) gambler_request_load_empty_scene for clean rebuilds
3) place_tiles / place_tile_picture / fill_rect
4) place or remove colliders (or ensure_colliders_for_all_tiles for all-solid maps)
5) set crosshair on safe spawn floor
6) gambler_get_snapshot and verify no collider/tile mismatches
7) gambler_request_save_scene"#;

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
        name = "gambler_remove_collider",
        description = "Remove only the collider at a snapped cell, leaving the tile in place. Use to fix orphan colliders (collider_cells_without_tile > 0) or make a tile purely decorative."
    )]
    pub async fn remove_collider(&self, Parameters(args): Parameters<RemoveColliderArgs>) -> String {
        let b = self.bridge.clone();
        let world_x = args.world_x;
        let world_y = args.world_y;
        match tokio::task::spawn_blocking(move || b.call(McpCmd::RemoveCollider { world_x, world_y })).await {
            Ok(Ok(v)) => Self::map_ok(v),
            Ok(Err(e)) => Self::map_err(e),
            Err(e) => Self::map_err(e.to_string()),
        }
    }

    #[tool(
        name = "gambler_fill_rect",
        description = "Fill every grid cell in a rectangle with the given tile_id. x1/y1 and x2/y2 are opposite world-space corners (order doesn't matter). Equivalent to calling place_tiles for every cell in the rect. Walkable floors still need gambler_ensure_colliders_for_all_tiles afterward."
    )]
    pub async fn fill_rect(&self, Parameters(args): Parameters<FillRectArgs>) -> String {
        let b = self.bridge.clone();
        match tokio::task::spawn_blocking(move || b.call(McpCmd::FillRect {
            x1: args.x1,
            y1: args.y1,
            x2: args.x2,
            y2: args.y2,
            tile_id: args.tile_id,
        })).await {
            Ok(Ok(v)) => Self::map_ok(v),
            Ok(Err(e)) => Self::map_err(e),
            Err(e) => Self::map_err(e.to_string()),
        }
    }

    #[tool(
        name = "gambler_place_actor",
        description = "Place an actor (player/NPC spawn marker) at the snapped grid cell. Only one actor per cell; placing on an occupied cell replaces it."
    )]
    pub async fn place_actor(&self, Parameters(args): Parameters<PlaceActorArgs>) -> String {
        let b = self.bridge.clone();
        let world_x = args.world_x;
        let world_y = args.world_y;
        match tokio::task::spawn_blocking(move || b.call(McpCmd::PlaceActor { world_x, world_y })).await {
            Ok(Ok(v)) => Self::map_ok(v),
            Ok(Err(e)) => Self::map_err(e),
            Err(e) => Self::map_err(e.to_string()),
        }
    }

    #[tool(
        name = "gambler_remove_actor",
        description = "Remove the actor at the snapped grid cell."
    )]
    pub async fn remove_actor(&self, Parameters(args): Parameters<RemoveActorArgs>) -> String {
        let b = self.bridge.clone();
        let world_x = args.world_x;
        let world_y = args.world_y;
        match tokio::task::spawn_blocking(move || b.call(McpCmd::RemoveActor { world_x, world_y })).await {
            Ok(Ok(v)) => Self::map_ok(v),
            Ok(Err(e)) => Self::map_err(e),
            Err(e) => Self::map_err(e.to_string()),
        }
    }

    #[tool(
        name = "gambler_get_scene_bounds",
        description = "Returns the bounding box of all tiles in the scene: min/max grid coords, center, and cell dimensions. Much faster than parsing a full snapshot when you only need spatial orientation."
    )]
    pub async fn get_scene_bounds(&self) -> String {
        let b = self.bridge.clone();
        match tokio::task::spawn_blocking(move || b.call(McpCmd::GetSceneBounds)).await {
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
        name = "gambler_get_pixel_art_settings",
        description = "Return current PixelArtSettings values for tile and player shader passes."
    )]
    pub async fn get_pixel_art_settings(&self) -> String {
        let b = self.bridge.clone();
        match tokio::task::spawn_blocking(move || b.call(McpCmd::GetPixelArtSettings)).await {
            Ok(Ok(v)) => Self::map_ok(v),
            Ok(Err(e)) => Self::map_err(e),
            Err(e) => Self::map_err(e.to_string()),
        }
    }

    #[tool(
        name = "gambler_set_pixel_art_settings",
        description = "Update pixel shader params. target is tile|player|both; omitted fields keep current values."
    )]
    pub async fn set_pixel_art_settings(
        &self,
        Parameters(args): Parameters<SetPixelArtSettingsArgs>,
    ) -> String {
        let b = self.bridge.clone();
        match tokio::task::spawn_blocking(move || {
            b.call(McpCmd::SetPixelArtSettings {
                target: args.target,
                pixel_size: args.pixel_size,
                color_levels: args.color_levels,
                dither_strength: args.dither_strength,
                scanline_strength: args.scanline_strength,
                palette_enabled: args.palette_enabled,
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
        name = "gambler_get_tile_catalog",
        description = "Return semantic catalog metadata per tile_id: name, material_type, solidity_default, traversal_affordance, style_tags, usage_hints, anti_patterns."
    )]
    pub async fn get_tile_catalog(&self) -> String {
        let b = self.bridge.clone();
        match tokio::task::spawn_blocking(move || b.call(McpCmd::GetTileCatalog)).await {
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
