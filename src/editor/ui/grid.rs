use bevy::{
    color::Color,
    gizmos::gizmos::Gizmos,
    math::{Isometry2d, Rot2, UVec2, Vec2},
    state::state::States,
};

use crate::{TILE_SCALE, TILE_SIZE, ZONE_SIZE};

/// This enum is used as a setting for the editor to determine wether or not we are trying to snap placed objects to the grid.
/// This is a user setting that can be toggled with CTRL + SHIFT + G, it is not saved to the document and will default to enabled on startup.
#[derive(Clone, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum GridSnap {
    #[default]
    Enabled,
    Disabled,
}

#[derive(Clone, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum ShowGrid {
    #[default]
    Yes,
    No,
}

pub fn draw_grid(mut gizmos: Gizmos) {
    gizmos
        .grid_2d(
            Isometry2d::new(Vec2::new(0.0, 0.0), Rot2::degrees(0.0)),
            UVec2::new(100, 100),
            Vec2::new(
                (TILE_SIZE * TILE_SCALE) as f32,
                (TILE_SIZE * TILE_SCALE) as f32,
            ),
            Color::srgba(0.0, 1.0, 0.0, 0.5),
        )
        .outer_edges();

    gizmos
        .grid_2d(
            Isometry2d::IDENTITY,
            UVec2::new(10, 10),
            Vec2::new(
                (TILE_SIZE * TILE_SCALE * ZONE_SIZE) as f32,
                (TILE_SIZE * TILE_SCALE * ZONE_SIZE) as f32,
            ),
            Color::srgba(1.0, 0.0, 0.0, 0.5),
        )
        .outer_edges();
}
