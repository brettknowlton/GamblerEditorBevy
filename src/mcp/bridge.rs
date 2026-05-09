//! Cross-thread messages between the MCP server task and Bevy

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpEnvelope {
    pub id: u64,
    pub cmd: McpCmd,
}

/// One tile placement for bulk MCP commands (world coords snap like single `place_tile`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceTilePayload {
    pub world_x: i64,
    pub world_y: i64,
    pub tile_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum McpCmd {
    PlaceTile {
        world_x: i64,
        world_y: i64,
        tile_id: u64,
    },
    /// Many tiles in one editor frame; dedupes by snapped grid cell (last wins).
    PlaceTiles { tiles: Vec<PlaceTilePayload> },
    /// ASCII-art style: `rows[0]` is the top row visually; origin is bottom-left of the picture in world space.
    PlaceTilePicture {
        origin_world_x: i64,
        origin_world_y: i64,
        rows: Vec<String>,
    },
    PlaceCollider { world_x: i64, world_y: i64 },
    /// Remove only the collider at a cell (tile is kept). Use to fix orphan colliders.
    RemoveCollider { world_x: i64, world_y: i64 },
    /// Adds a solid collider on every snapped grid cell that has a tile but no collider yet.
    EnsureCollidersForAllTiles,
    RemoveTile {
        world_x: i64,
        world_y: i64,
    },
    /// Fill a rectangle of grid cells with tiles. x1/y1 and x2/y2 are opposite world-space corners.
    FillRect {
        x1: i64,
        y1: i64,
        x2: i64,
        y2: i64,
        tile_id: u64,
    },
    /// Place an actor at the snapped grid cell.
    PlaceActor { world_x: i64, world_y: i64 },
    /// Remove the actor at the snapped grid cell.
    RemoveActor { world_x: i64, world_y: i64 },
    SetSelectedTile {
        tile_id: u64,
    },
    SetEditorState {
        state: String,
    },
    SetCrosshair {
        x: f32,
        y: f32,
    },
    GetTileCatalog,
    GetSnapshot,
    GetPixelArtSettings,
    /// Update one or both shader passes (tile/player). Omitted fields remain unchanged.
    SetPixelArtSettings {
        target: String,
        pixel_size: Option<f32>,
        color_levels: Option<f32>,
        dither_strength: Option<f32>,
        scanline_strength: Option<f32>,
        palette_enabled: Option<f32>,
    },
    /// Returns bounding box of all tiles (min/max grid coords, center, dimensions).
    GetSceneBounds,
    RequestLoadScene,
    RequestLoadEmptyScene,
    RequestSaveScene,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpReply {
    pub id: u64,
    pub result: Result<serde_json::Value, String>,
}
