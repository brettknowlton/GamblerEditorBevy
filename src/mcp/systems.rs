use std::collections::{BTreeMap, HashSet};

use bevy::prelude::*;

use crate::{
    Crosshair, EditorObject, EditorObjectKind, EditorState, SelectedTileID, TileID,
};
use crate::editor::editor_modes::collider_mode::ColliderObject;
use crate::editor::editor_modes::significant_component::SignificantComponent;
use crate::editor::editor_modes::tile_mode::TileObject;
use crate::editor::ui::message_display::MessageDisplay;
use crate::utilities::coordinate::{Coordinate, CoordinateSpace};
use crate::{MAX_SPRITESHEET_ITEMS, SCALED_TILE_WIDTH};

use super::bridge::{McpCmd, McpEnvelope, McpReply, PlaceTilePayload};
use super::EditorMcpBridge;

/// Max placements per `PlaceTiles` / picture call to keep one frame responsive.
pub const MCP_MAX_PLACE_TILES: usize = 8192;

fn stride_world() -> i64 {
    SCALED_TILE_WIDTH as i64
}

fn validate_tile_id(tile_id: u64) -> Result<(), String> {
    if tile_id >= MAX_SPRITESHEET_ITEMS {
        return Err(format!(
            "tile_id must be in 0..{} (got {})",
            MAX_SPRITESHEET_ITEMS, tile_id
        ));
    }
    Ok(())
}

/// True if a tile sprite occupies this snapped grid cell (colliders must only exist here).
fn snapped_grid_has_tile(
    tiles: &Query<(Entity, &EditorObject), With<TileObject>>,
    gx: i64,
    gy: i64,
) -> bool {
    tiles.iter().any(|(_, eo)| {
        matches!(eo.kind, EditorObjectKind::Tile(TileID::Some(_)))
            && eo.coordinate.x == gx
            && eo.coordinate.y == gy
    })
}

/// ` ` `.` `_` = empty; `0`–`9` = 0–9; `a`–`v` / `A`–`V` = 10–31.
fn decode_picture_char(c: u8) -> Result<Option<u64>, String> {
    Ok(match c {
        b' ' | b'.' | b'_' | b'\t' => None,
        b'0'..=b'9' => Some((c - b'0') as u64),
        b'a'..=b'v' | b'A'..=b'V' => {
            let lc = c.to_ascii_lowercase();
            Some(10 + (lc - b'a') as u64)
        }
        _ => {
            return Err(format!(
                "invalid picture character {:?} (use 0-9, a-v, space, . or _)",
                c as char
            ));
        }
    })
}

fn parse_editor_state(s: &str) -> Result<EditorState, String> {
    Ok(match s.trim().to_ascii_lowercase().as_str() {
        "inactive" => EditorState::Inactive,
        "normal" => EditorState::Normal,
        "load_ask" => EditorState::LoadAsk,
        "loading" => EditorState::Loading,
        "loading_empty" => EditorState::LoadingEmpty,
        "save_ask" => EditorState::SaveAsk,
        "saving" => EditorState::Saving,
        "quit_ask" => EditorState::QuitAsk,
        "edit_tile" => EditorState::Editing(EditorObjectKind::Tile(TileID::Any)),
        "edit_collider" => EditorState::Editing(EditorObjectKind::Collider),
        "edit_actor" => EditorState::Editing(EditorObjectKind::Actor),
        other => {
            return Err(format!(
                "unknown editor state '{other}'; use inactive|normal|load_ask|loading|loading_empty|save_ask|saving|quit_ask|edit_tile|edit_collider|edit_actor"
            ));
        }
    })
}

fn editor_state_label(state: &EditorState) -> String {
    match state {
        EditorState::Inactive => "inactive".into(),
        EditorState::Normal => "normal".into(),
        EditorState::LoadAsk => "load_ask".into(),
        EditorState::Loading => "loading".into(),
        EditorState::LoadingEmpty => "loading_empty".into(),
        EditorState::SaveAsk => "save_ask".into(),
        EditorState::Saving => "saving".into(),
        EditorState::QuitAsk => "quit_ask".into(),
        EditorState::Editing(k) => match k {
            EditorObjectKind::Tile(_) => "edit_tile".into(),
            EditorObjectKind::Collider => "edit_collider".into(),
            EditorObjectKind::Actor => "edit_actor".into(),
            _ => format!("editing:{k:?}"),
        },
    }
}

pub fn drain_mcp_inbox(
    bridge: Res<EditorMcpBridge>,
    mut commands: Commands,
    mut next_editor: ResMut<NextState<EditorState>>,
    editor_state: Res<State<EditorState>>,
    mut selected_tile: ResMut<SelectedTileID>,
    mut bottom_bar: ResMut<MessageDisplay>,
    tiles: Query<(Entity, &EditorObject), With<TileObject>>,
    colliders: Query<(Entity, &EditorObject), With<ColliderObject>>,
    mut crosshair: Query<&mut Transform, With<Crosshair>>,
) {
    while let Ok(env) = bridge.from_mcp.try_recv() {
        let McpEnvelope { id, cmd } = env;
        let result = run_mcp_cmd(
            cmd,
            &mut commands,
            &mut next_editor,
            editor_state.get(),
            &mut selected_tile,
            &mut bottom_bar,
            &tiles,
            &colliders,
            &mut crosshair,
        );
        let _ = bridge.to_mcp.send(McpReply { id, result });
    }
}

fn run_mcp_cmd(
    cmd: McpCmd,
    commands: &mut Commands,
    next_editor: &mut NextState<EditorState>,
    current: &EditorState,
    selected_tile: &mut SelectedTileID,
    bottom_bar: &mut MessageDisplay,
    tiles: &Query<(Entity, &EditorObject), With<TileObject>>,
    colliders: &Query<(Entity, &EditorObject), With<ColliderObject>>,
    crosshair: &mut Query<&mut Transform, With<Crosshair>>,
) -> Result<serde_json::Value, String> {
    match cmd {
        McpCmd::PlaceTile {
            world_x,
            world_y,
            tile_id,
        } => {
            validate_tile_id(tile_id)?;
            let coord = Coordinate::new_world_space(world_x, world_y).snap_to_grid();
            let to_place = EditorObject::new(
                EditorObjectKind::Tile(TileID::Some(tile_id)),
                coord,
            );
            TileObject::place(commands, to_place, tiles);
            bottom_bar.send_place_eo_message("tile (MCP)", coord);
            Ok(serde_json::json!({
                "placed": true,
                "grid_x": coord.x,
                "grid_y": coord.y,
                "tile_id": tile_id,
            }))
        }
        McpCmd::PlaceTiles { tiles: payloads } => {
            if payloads.len() > MCP_MAX_PLACE_TILES {
                return Err(format!(
                    "too many tiles in one call (max {}, got {})",
                    MCP_MAX_PLACE_TILES,
                    payloads.len()
                ));
            }
            let mut merged: BTreeMap<(i64, i64), u64> = BTreeMap::new();
            for PlaceTilePayload {
                world_x,
                world_y,
                tile_id,
            } in &payloads
            {
                validate_tile_id(*tile_id)?;
                let coord = Coordinate::new_world_space(*world_x, *world_y).snap_to_grid();
                merged.insert((coord.x, coord.y), *tile_id);
            }
            let n = merged.len();
            for ((gx, gy), tile_id) in merged {
                let coord = Coordinate {
                    x: gx,
                    y: gy,
                    format: CoordinateSpace::GridSpace,
                };
                let to_place =
                    EditorObject::new(EditorObjectKind::Tile(TileID::Some(tile_id)), coord);
                TileObject::place(commands, to_place, tiles);
            }
            bottom_bar.send_message(format!("MCP: bulk placed {n} tile cells"));
            Ok(serde_json::json!({
                "placed_cells": n,
                "input_count": payloads.len(),
            }))
        }
        McpCmd::PlaceTilePicture {
            origin_world_x,
            origin_world_y,
            rows,
        } => {
            if rows.is_empty() {
                return Err("place_tile_picture: rows must be non-empty".into());
            }
            let width = rows[0].len();
            if width == 0 {
                return Err("place_tile_picture: each row must be non-empty".into());
            }
            for (i, row) in rows.iter().enumerate() {
                if row.len() != width {
                    return Err(format!(
                        "place_tile_picture: row {i} length {} != first row width {}",
                        row.len(),
                        width
                    ));
                }
            }
            let cell_count = width * rows.len();
            if cell_count > MCP_MAX_PLACE_TILES {
                return Err(format!(
                    "picture too large ({} cells, max {})",
                    cell_count, MCP_MAX_PLACE_TILES
                ));
            }
            let stride = stride_world();
            let mut placed = 0u64;
            for (row_idx, row) in rows.iter().enumerate() {
                for (col, c) in row.bytes().enumerate() {
                    let tile_id = match decode_picture_char(c).map_err(|e| {
                        format!("place_tile_picture row {row_idx} col {col}: {e}")
                    })? {
                        Some(id) => id,
                        None => continue,
                    };
                    validate_tile_id(tile_id)?;
                    let dy = (rows.len() - 1 - row_idx) as i64;
                    let world_x = origin_world_x + col as i64 * stride;
                    let world_y = origin_world_y + dy * stride;
                    let coord = Coordinate::new_world_space(world_x, world_y).snap_to_grid();
                    let to_place =
                        EditorObject::new(EditorObjectKind::Tile(TileID::Some(tile_id)), coord);
                    TileObject::place(commands, to_place, tiles);
                    placed += 1;
                }
            }
            bottom_bar.send_message(format!(
                "MCP: picture placed {placed} non-empty tiles ({}x{})",
                width,
                rows.len()
            ));
            Ok(serde_json::json!({
                "placed_tiles": placed,
                "width_cells": width,
                "height_rows": rows.len(),
            }))
        }
        McpCmd::PlaceCollider { world_x, world_y } => {
            let coord = Coordinate::new_world_space(world_x, world_y).snap_to_grid();
            if !snapped_grid_has_tile(tiles, coord.x, coord.y) {
                return Err(format!(
                    "cannot place collider at grid ({},{}): no tile on this cell — colliders must align with tile art or the player hits invisible walls",
                    coord.x, coord.y
                ));
            }
            let to_place = EditorObject::new(EditorObjectKind::Collider, coord);
            ColliderObject::place(commands, to_place, colliders);
            bottom_bar.send_place_eo_message("collider (MCP)", coord);
            Ok(serde_json::json!({
                "placed": true,
                "grid_x": coord.x,
                "grid_y": coord.y,
            }))
        }
        McpCmd::EnsureCollidersForAllTiles => {
            let mut occupied: HashSet<(i64, i64)> = colliders
                .iter()
                .map(|(_, eo)| (eo.coordinate.x, eo.coordinate.y))
                .collect();
            let mut added = 0u64;
            let mut skipped_existing = 0u64;
            for (_, eo) in tiles.iter() {
                if let EditorObjectKind::Tile(TileID::Some(_)) = eo.kind {
                    let k = (eo.coordinate.x, eo.coordinate.y);
                    if occupied.contains(&k) {
                        skipped_existing += 1;
                        continue;
                    }
                    let to_place = EditorObject::new(EditorObjectKind::Collider, eo.coordinate);
                    ColliderObject::place(commands, to_place, colliders);
                    occupied.insert(k);
                    added += 1;
                }
            }
            bottom_bar.send_message(format!(
                "MCP: ensure colliders — added {added}, cells already had collider {skipped_existing}"
            ));
            Ok(serde_json::json!({
                "colliders_added": added,
                "cells_already_had_collider": skipped_existing,
            }))
        }
        McpCmd::RemoveTile { world_x, world_y } => {
            let coord = Coordinate::new_world_space(world_x, world_y).snap_to_grid();
            TileObject::remove(
                commands,
                coord,
                EditorObjectKind::Tile(TileID::Any),
                tiles,
            );
            ColliderObject::remove(
                commands,
                coord,
                EditorObjectKind::Collider,
                colliders,
            );
            bottom_bar.send_remove_eo_message("tile (MCP)", coord);
            Ok(serde_json::json!({ "removed": true, "grid_x": coord.x, "grid_y": coord.y }))
        }
        McpCmd::SetSelectedTile { tile_id } => {
            validate_tile_id(tile_id)?;
            selected_tile.0 = tile_id;
            Ok(serde_json::json!({ "selected_tile_id": tile_id }))
        }
        McpCmd::SetEditorState { state } => {
            let next = parse_editor_state(&state)?;
            next_editor.set(next);
            bottom_bar.send_message(format!("MCP: editor state -> {}", editor_state_label(&next)));
            Ok(serde_json::json!({ "state": editor_state_label(&next) }))
        }
        McpCmd::SetCrosshair { x, y } => {
            let Ok(mut t) = crosshair.single_mut() else {
                return Err("crosshair not found".into());
            };
            t.translation.x = x;
            t.translation.y = y;
            Ok(serde_json::json!({ "x": x, "y": y }))
        }
        McpCmd::GetSnapshot => {
            let ch = crosshair
                .single()
                .map(|t| serde_json::json!({ "x": t.translation.x, "y": t.translation.y }))
                .unwrap_or(serde_json::Value::Null);
            let collider_set: HashSet<(i64, i64)> = colliders
                .iter()
                .filter(|(_, eo)| matches!(eo.kind, EditorObjectKind::Collider))
                .map(|(_, eo)| (eo.coordinate.x, eo.coordinate.y))
                .collect();

            let mut tile_coords: HashSet<(i64, i64)> = HashSet::new();
            let mut placed = Vec::new();
            for (_, eo) in tiles.iter() {
                if let EditorObjectKind::Tile(TileID::Some(id)) = eo.kind {
                    let c = eo.coordinate;
                    tile_coords.insert((c.x, c.y));
                    placed.push(serde_json::json!({
                        "grid_x": c.x,
                        "grid_y": c.y,
                        "format": format!("{:?}", c.format),
                        "tile_id": id,
                    }));
                }
            }
            let mut collider_cells = Vec::new();
            for (_, eo) in colliders.iter() {
                if matches!(eo.kind, EditorObjectKind::Collider) {
                    let c = eo.coordinate;
                    collider_cells.push(serde_json::json!({
                        "grid_x": c.x,
                        "grid_y": c.y,
                    }));
                }
            }
            let tiles_without_collider: u64 = tile_coords
                .iter()
                .filter(|k| !collider_set.contains(*k))
                .count() as u64;
            let colliders_without_tile: u64 = collider_set
                .iter()
                .filter(|k| !tile_coords.contains(*k))
                .count() as u64;
            Ok(serde_json::json!({
                "editor_state": editor_state_label(current),
                "selected_tile_id": selected_tile.0,
                "crosshair": ch,
                "tiles": placed,
                "colliders": collider_cells,
                "tile_cells_missing_collider": tiles_without_collider,
                "collider_cells_without_tile": colliders_without_tile,
                "editor_constants": {
                    "grid_cell_world_px": SCALED_TILE_WIDTH,
                    "max_tile_id": MAX_SPRITESHEET_ITEMS - 1,
                    "spritesheet_columns": crate::consts::SPRITESHEET_WIDTH,
                    "mcp_max_place_tiles_per_call": MCP_MAX_PLACE_TILES,
                },
            }))
        }
        McpCmd::RequestLoadScene => {
            next_editor.set(EditorState::Loading);
            bottom_bar.send_message("MCP: Loading scene");
            Ok(serde_json::json!({ "next": "loading" }))
        }
        McpCmd::RequestLoadEmptyScene => {
            next_editor.set(EditorState::LoadingEmpty);
            bottom_bar.send_message("MCP: Loading empty scene");
            Ok(serde_json::json!({ "next": "loading_empty" }))
        }
        McpCmd::RequestSaveScene => {
            next_editor.set(EditorState::Saving);
            bottom_bar.send_message("MCP: Saving scene");
            Ok(serde_json::json!({ "next": "saving" }))
        }
    }
}
