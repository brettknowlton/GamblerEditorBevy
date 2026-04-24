use bevy::prelude::*;

use super::*;
use crate::{
    bounding_box::BoundingBox, editor_modes::significant_component::SignificantComponent,
    SPRITESHEET_WIDTH, TILE_SIZE,
};

/// A component to track some basic info about a tile (actually its just a tag right now but that might change)
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(EditorObject)]
pub struct TileObject {
    pub kind: EditorObjectKind,
    pub coord: Coordinate,
    pub bounding_box: BoundingBox,
}

impl TileObject {
    /// Creates a new TileObject with the given coordinate and a default bounding box,
    /// the kind is set to Tile with the internal type of None by default but this can be changed after creation
    fn new(coord: Coordinate) -> Self {
        Self {
            kind: EditorObjectKind::Tile(TileID::None),
            coord,
            bounding_box: BoundingBox::from_coordinate(coord),
        }
    }

    pub fn with_id(mut self, id: EditorObjectKind) -> Self {
        self.kind = id;
        self
    }

    /// Gets the UV rect for a given tile ID, this is used to determine which part of the spritesheet to use when rendering this tile
    pub fn get_uv_rect(tile_id: u64) -> Rect {
        Rect {
            min: Vec2::new(
                (tile_id % SPRITESHEET_WIDTH) as f32 * TILE_SIZE as f32,
                (tile_id / SPRITESHEET_WIDTH) as f32 * TILE_SIZE as f32,
            ),
            max: Vec2::new(
                (tile_id % SPRITESHEET_WIDTH + 1) as f32 * TILE_SIZE as f32,
                (tile_id / SPRITESHEET_WIDTH + 1) as f32 * TILE_SIZE as f32,
            ),
        }
    }
}

impl Default for TileObject {
    fn default() -> Self {
        Self {
            kind: EditorObjectKind::Tile(TileID::None),
            coord: Coordinate::default(),
            bounding_box: BoundingBox::default(),
        }
    }
}

impl SignificantComponent for TileObject {
    fn place_rectangle(_rect: Rect, _commands: Commands) {
        //make a tile like normal in this rect, but use sliced tiles over the sprite sheet selection
        todo!();
    }

    fn at_coordinate(coord: Coordinate) -> Self {
        Self::new(coord)
    }
}
