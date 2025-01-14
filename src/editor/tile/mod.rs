use bevy::prelude::*;
use bevy::sprite::Anchor;
use std::path::PathBuf;
use crate::{ utilities::*, resources::*, EditorObject, TILE_SIZE };
use crate::consts::*;
use super::*;

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum TileEditorState {
    #[default]
    Inactive,
    Active,
}

pub fn tilemode_plugin(app: &mut App) {
    app.init_state::<TileEditorState>()
        .register_type::<Tile>()
        .register_type::<Coordinate>()
        .register_type::<TCoordinate>()
        .insert_resource(PlaceholderTile(Tile::new()))
        //startup systems (may need to be moved from here to maintain order)
        .add_systems(Startup, load_spritesheet)

        //OnEnter systems
        .add_systems(OnEnter(EditorState::Tile), (init_tilemode, show_placeholder).chain())

        //Update systems, that run only while TileEditor is active
        .add_systems(
            Update,
            (tilemode_keybinds, update_placeholder)
                .chain()
                .run_if(in_state(TileEditorState::Active))
        )

        //OnExit systems
        .add_systems(OnExit(EditorState::Tile), (
            exit_tilemode.before(tilemode_keybinds),
            despawn_all::<TileModeUI>.before(exit_tilemode),
        ));

    //we could also take care of some post-exit cleanup here, like despawning all the UI elements by using the schedule OnEnter(EditorState::Inactive) and then despawning all the UI elements
}

// fn ensure_unique_tiles(mut commands: Commands, tiles: Query<(Entity, &Tile)>) {
//     let mut seen = std::collections::HashSet::new();

//     for (e, t) in tiles.iter() {
//         if seen.contains(&t.coordinate) {
//             //remove the older of the two tiles
//             commands.entity(e).despawn();
//         } else {
//             seen.insert(&t.coordinate);
//         }
//     }
// }

fn init_tilemode(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<TileEditorState>>,

    crosshairs: Query<(&Transform, &Crosshair)>
) {
    println!("Entering Tile Editing Mode");
    next_state.set(TileEditorState::Active);

    //display the "tilemode" menu
    let texpath = PathBuf::from("textures/menus/menu1.png");
    let tex1 = asset_server.load(texpath);

    //offsets to make UI appear in the top left corner of the screen while still being anchored to the crosshair location
    let c = crosshairs.single();

    let x_off = -WINDOW_WIDTH / 2.0 + c.0.translation.x;
    let y_off = -WINDOW_HEIGHT / 2.0 + c.0.translation.y;

    //spawn tilemodeUI
    commands.spawn((
        Sprite {
            image: tex1,
            anchor: Anchor::BottomLeft,
            custom_size: Some(Vec2::new(WINDOW_WIDTH / 6.0, WINDOW_HEIGHT)),
            image_mode: bevy::sprite::SpriteImageMode::Sliced(TextureSlicer {
                border: BorderRect {
                    bottom: 4.0,
                    left: 4.0,
                    right: 4.0,
                    top: 4.0,
                },
                sides_scale_mode: bevy::sprite::SliceScaleMode::Stretch,
                ..default()
            }),
            ..default()
        },
        Transform {
            translation: Vec3::new(x_off, y_off, 0.0),
            ..default()
        },
        UIItem {
            ..default()
        },
        TileModeUI,
    ));
}

fn tilemode_keybinds(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,

    crosshairs: Query<(&Transform, &Crosshair)>,
    mut current_editor_object: ResMut<PlaceholderTile>,
    tiles: Query<(Entity, &Tile)>
) {
    //"P" handles placement of a tile and adding it to the scene
    if input.just_pressed(KeyCode::KeyP) {
        //clean up the bevy query overhead
        let (t, _) = crosshairs.single();
        let focused_item = &current_editor_object.0;
        let mut coord = Coordinate::from(t.translation);
        let pushover = 1000 * ((TILE_SIZE * TILE_SCALE) as i64); //this effectively offsets all my tiles 1000 to the right and down, so that negative coordinates arent a problem and rounds to the nearest integer
        coord = Coordinate(
            ((coord.0 + pushover) / ((TILE_SIZE * TILE_SCALE) as i64)) *
                ((TILE_SIZE * TILE_SCALE) as i64) -
                pushover,
            ((coord.1 + pushover) / ((TILE_SIZE * TILE_SCALE) as i64)) *
                ((TILE_SIZE * TILE_SCALE) as i64) -
                pushover
        );

        // println!("coords: {}{}", coord.0, coord.1);

        //check if a tile already exists at this location and remove it if it does
        if let Some(item) = tiles.iter().find(|(_, t)| t.coordinate == coord) {
            //remove the old tile
            commands.entity(item.0).despawn();
        }
        commands.spawn((
            Tile {
                tile_type: focused_item.tile_type,
                coordinate: coord,
            },
            Transform {
                translation: Vec3::new(coord.0 as f32, coord.1 as f32, -5.0),
                scale: Vec3::new(TILE_SCALE as f32, TILE_SCALE as f32, 1.0),
                ..default()
            },
            EditorObject {
                coordinate: TCoordinate::new('T', coord),
                internal_type: focused_item.tile_type,
            },
        ));
    }
    // "L" handles removal of a tile from the scene, similar to placing one just doesnt need to worry about the tile creation part afterwards
    if input.just_pressed(KeyCode::KeyL) {
        let (t, _) = crosshairs.single();
        let mut coord = Coordinate::from(t.translation);
        //"floor" the coordinate to the nearest tile grid space in a way that (kind of) respects the negative coordinate space, just dont place anything more than 1000 tiles away from the origin until I can figure that out
        let pushover = 1000 * ((TILE_SIZE * TILE_SCALE) as i64);
        coord = Coordinate(
            ((coord.0 + pushover) / ((TILE_SIZE * TILE_SCALE) as i64)) *
                ((TILE_SIZE * TILE_SCALE) as i64) -
                pushover,
            ((coord.1 + pushover) / ((TILE_SIZE * TILE_SCALE) as i64)) *
                ((TILE_SIZE * TILE_SCALE) as i64) -
                pushover
        );

        // println!("coords: {}{}", coord.0, coord.1);

        //check if a tile already exists at this location and remove it if it does
        if let Some(item) = tiles.iter().find(|(_, t)| t.coordinate == coord) {
            //remove the old tile
            commands.entity(item.0).despawn();
        }
    }

    if input.just_pressed(KeyCode::ArrowRight) {
        //cycles through the spritesheet to the right
        current_editor_object.0.tile_type =
            (current_editor_object.0.tile_type + 1) % (MAX_SPRITESHEET_ITEMS as u64);
    }
    if input.just_pressed(KeyCode::ArrowLeft) {
        //cycles through the spritesheet to the left
        current_editor_object.0.tile_type =
            (current_editor_object.0.tile_type + (MAX_SPRITESHEET_ITEMS as u64) - 1) %
            (MAX_SPRITESHEET_ITEMS as u64);
    }
    if input.just_pressed(KeyCode::ArrowUp) {
        //cycles through the spritesheet up
        current_editor_object.0.tile_type = (MAX_SPRITESHEET_ITEMS - SPRITESHEET_WIDTH) as u64 + (current_editor_object.0.tile_type % SPRITESHEET_WIDTH as u64);
    }
    if input.just_pressed(KeyCode::ArrowDown) {
        //cycles through the spritesheet down
        current_editor_object.0.tile_type =
            (current_editor_object.0.tile_type + (SPRITESHEET_WIDTH as u64)) %
            (MAX_SPRITESHEET_ITEMS as u64);
    }
}

fn update_placeholder(
    mut ui: Query<(&mut Sprite, &mut PlaceholderObject)>,
    placeholder: ResMut<PlaceholderTile>
) {
    for (mut sprite, _) in ui.iter_mut() {
        //update the placeholder tile to match the current tile type of our placeholderTile resource
        //do this by updating the UVs of the sprite
        sprite.rect = Some(Rect {
            min: Vec2::new(
                (((placeholder.0.tile_type as usize) % SPRITESHEET_WIDTH) as f32) *
                    (TILE_SIZE as f32),
                (((placeholder.0.tile_type as usize) / SPRITESHEET_WIDTH) as f32) *
                    (TILE_SIZE as f32)
            ),
            max: Vec2::new(
                (((placeholder.0.tile_type as usize) % SPRITESHEET_WIDTH) as f32) *
                    (TILE_SIZE as f32) +
                    (TILE_SIZE as f32),
                (((placeholder.0.tile_type as usize) / SPRITESHEET_WIDTH) as f32) *
                    (TILE_SIZE as f32) +
                    (TILE_SIZE as f32)
            ),
        });

        //also move the placeholder tile to the current crosshair location
    }
}

fn show_placeholder(
    mut commands: Commands,
    spritesheet: Res<TilesheetHandle>,
    crosshairs: Query<(&Transform, &Crosshair)>
) {
    let c = crosshairs.single();
    let x_off = c.0.translation.x;
    let y_off = c.0.translation.y;

    let texpath = spritesheet.0.clone();
    //display the placeholder tile
    commands.spawn((
        Tile {
            tile_type: 0,
            coordinate: Coordinate(0, 0),
        },
        Sprite {
            image: texpath,
            rect: Some(Rect {
                min: Vec2::new(0.0, 0.0),
                max: Vec2::new(TILE_SIZE as f32, TILE_SIZE as f32),
            }),
            ..default()
        },
        Transform {
            translation: Vec3::new(x_off, y_off, 0.0),
            ..default()
        },
        UIItem {
            ..default()
        },
        TileModeUI,
        PlaceholderObject,
    ));
}

fn load_spritesheet(mut commands: Commands, asset_server: Res<AssetServer>) {
    //load the tilesheet for this mode
    let tex_path = PathBuf::from("textures/tiles/tilesheet.png");

    //load happens here
    let texture = asset_server.load(tex_path);

    //insert the texture handle into the resources for easy access later
    commands.insert_resource(TilesheetHandle(texture.clone()));
}

fn exit_tilemode(mut commands: Commands, mut tile_state: ResMut<NextState<TileEditorState>>) {
    tile_state.set(TileEditorState::Inactive);
    println!("Exiting Tile Editing Mode");

    //remove the CurrentEditorObject resource
    commands.insert_resource(PlaceholderTile(Tile::new()));
}

/// A component that marks an entity as part of the tile editing UI.
#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(UIItem)]
struct TileModeUI;

/// A component to track some basic info about a tile
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct Tile {
    pub tile_type: u64,
    pub coordinate: Coordinate,
}
impl Tile {
    fn new() -> Self {
        Self {
            tile_type: 0,
            coordinate: Coordinate(0, 0),
        }
    }
}
impl Default for Tile {
    fn default() -> Self {
        Self::new()
    }
}
