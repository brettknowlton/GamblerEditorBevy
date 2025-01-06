use std::path::PathBuf;

use crate::{ utilities::*, TILE_SIZE };
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
        .add_systems(OnEnter(EditorState::Tile), (
            init_tilemode,
            init_tilemode_textures.before(init_tilemode),
        ))

        //placeholder resource for whatever tile we are trying to place
        .insert_resource(CurrentEditorObject(Tile::new()))

        //more keybinds for tile editing
        .add_systems(Update, (
            tilemode_keybinds.run_if(in_state(TileEditorState::Active)),
            move_tile_ui.run_if(in_state(TileEditorState::Active)),
        ))
        .add_systems(OnExit(EditorState::Tile), (
            exit_tilemode.before(tilemode_keybinds),
            despawn_all::<TileModeUI>.before(exit_tilemode),
        ));
}


fn move_tile_ui(
    mut tileUI: Query<(&mut Transform, &TileModeUI)>, 
    crosshair: Query<&Crosshair>
) {
    for (mut transform, ui) in tileUI.iter_mut() {
        let crosshair = crosshair.single();
        transform.translation = Vec3::new(crosshair.location.0 as f32 + ui.x_offset, crosshair.location.1 as f32 + ui.y_offset, 0.0);
    }
}

fn init_tilemode_textures(mut commands: Commands, asset_server: Res<AssetServer>) {
    //load the tilesheet for this mode
    let tex_path = PathBuf::from("textures/tiles/tilesheet.png");
    let texture = asset_server.load(tex_path);

    commands.insert_resource(TilesheetHandle(texture.clone()));
}

fn init_tilemode(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<TileEditorState>>
) {
    println!("Entering Tile Editing Mode");
    next_state.set(TileEditorState::Active);

    //display the "tilemode" menu
    let texpath = PathBuf::from("textures/menus/menu1.png");
    let tex1 = asset_server.load(texpath);

    let x_off = - WINDOW_WIDTH / 2.0 + UI_SCALE as f32 * TILE_SIZE as f32;
    let y_off = - WINDOW_HEIGHT / 2.0 + UI_SCALE as f32 * TILE_SIZE as f32;

    //spawn tilemodeUI
    commands.spawn((
        Sprite {
            image: tex1,
            ..default()
        },
        Transform {
            translation: Vec3::new(WINDOW_WIDTH / 2.0 + x_off, WINDOW_HEIGHT / 2.0 + y_off, 0.0),
            scale: Vec3::new(UI_SCALE as f32, UI_SCALE as f32, 1.0),
            ..default()
        },
        TileModeUI {
            x_offset: x_off,
            y_offset: y_off,
        },
    ));
}

fn tilemode_keybinds(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    crosshair: Query<&Crosshair>,

    // state: ResMut<NextState<EditorMode>>,
    mut current_editor_object: ResMut<CurrentEditorObject>,

    spritesheet: Res<TilesheetHandle>
    // atlas_layout: Res<TileAtlasLayout>
) {
    if input.just_pressed(KeyCode::KeyP) {
        //clean up the bevy query overhead
        let mut coord = crosshair.single().location;

        //"floor" the coordinate to the nearest tile grid space in a way that (kind of) respects the negative coordinate space, just dont place anything more than 1000 tiles away from the origin until I can figure that out
        let pushover = 1000 * (TILE_SIZE * TILE_SCALE) as i64;
        coord = Coordinate(
            (((coord.0 + pushover) / ((TILE_SIZE * TILE_SCALE) as i64))) * ((TILE_SIZE * TILE_SCALE)) as i64 - pushover,
            (((coord.1 + pushover) / ((TILE_SIZE * TILE_SCALE) as i64))) * ((TILE_SIZE * TILE_SCALE)) as i64 - pushover
        );

        println!("coords: {}{}", coord.0, coord.1);

        let focused_item = &current_editor_object.0;

        //add the currently selected tile to the scene at this coordinate (Rounded to the nearest tile gridspace)
        commands.spawn((
            //may be able to get rid of this and just use the EditorObject component
            Tile {
                tile_type: focused_item.tile_type,
                coordinate: coord,
            },

            //all sprites will use the same texture as a source, just change UV according to the current tile type
            //spritesheet is always SPRITESHEET_WIDTH many tiles wide so SPRITESHEET_WIDTH*SPRITE_SIZE is the width of the texture, the height is determinable because we know the MAX_SPRITESHEET_ITEMS so stop loading if we reach that many
            Sprite {
                image: spritesheet.0.clone(),
                //the UVs are the same for every tile, just change the offset
                rect: Some(Rect {
                    min: Vec2::new(
                        (((focused_item.tile_type as usize) % SPRITESHEET_WIDTH) as f32) *
                            (TILE_SIZE as f32),
                        (((focused_item.tile_type as usize) / SPRITESHEET_WIDTH) as f32) *
                            (TILE_SIZE as f32)
                    ),
                    max: Vec2::new(
                        (((focused_item.tile_type as usize) % SPRITESHEET_WIDTH) as f32) *
                            (TILE_SIZE as f32) +
                            (TILE_SIZE as f32),
                        (((focused_item.tile_type as usize) / SPRITESHEET_WIDTH) as f32) *
                            (TILE_SIZE as f32) +
                            (TILE_SIZE as f32)
                    ),
                }),
                anchor: Anchor::BottomLeft,
                ..default()
            },
            Transform {
                translation: Vec3::new(coord.0 as f32, coord.1 as f32, 0.0),
                scale: Vec3::new(TILE_SCALE as f32, TILE_SCALE as f32, 1.0),
                ..default()
            },
            EditorObject::new('T', focused_item.tile_type, coord),
        ));
    }

    if input.just_pressed(KeyCode::KeyL) {
        // scene.single_mut().remove(focused_item.0.coordinate);
        println!("todo!()");
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
        current_editor_object.0.tile_type =
            (current_editor_object.0.tile_type + (SPRITESHEET_WIDTH as u64)) %
            (MAX_SPRITESHEET_ITEMS as u64);
    }
    if input.just_pressed(KeyCode::ArrowDown) {
        //cycles through the spritesheet down
        current_editor_object.0.tile_type =
            (current_editor_object.0.tile_type +
                (MAX_SPRITESHEET_ITEMS as u64) -
                (SPRITESHEET_WIDTH as u64)) %
            (MAX_SPRITESHEET_ITEMS as u64);
    }
}

fn exit_tilemode(mut commands: Commands, mut tile_state: ResMut<NextState<TileEditorState>>) {
    tile_state.set(TileEditorState::Inactive);
    println!("Exiting Tile Editing Mode");

    //remove the CurrentEditorObject resource
    commands.insert_resource(CurrentEditorObject(Tile::new()));
}


/// A handle to the tilesheet image.
#[derive(Resource, Default)]
struct TilesheetHandle(Handle<Image>);

/// A component that marks an entity as part of the tile editing UI.
#[derive(Component)]
struct TileModeUI{
    x_offset: f32,
    y_offset: f32,
}
/// A component to track some basic info about a tile
#[derive(Component, Debug)]
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

/// A component that marks an entity as a savable editor item.
#[derive(Component, Default)]
pub struct EditorObject {
    major_type: char, //'T' for tile, 'E' for entity, 'P' for player, etc.
    internal_type: u64, //ultimatley an index into which style of tile or entity we are using within the major type
    goid: Option<String>, //game object id
    coordinate: Coordinate,
}

impl EditorObject {
    fn new(major_type: char, internal_type: u64, coord: Coordinate) -> Self {
        (Self {
            major_type,
            internal_type,
            goid: None,
            coordinate: coord,
        }).generate_goid(coord)
    }

    fn get_object_type(&self) -> char {
        self.major_type
    }
    fn get_internal_type(&self) -> u64 {
        self.internal_type
    }
    fn get_goid(&self) -> Option<String> {
        self.goid.clone()
    }
    fn get_coordinate(&self) -> Coordinate {
        self.coordinate
    }

    fn set_major_type(&mut self, v: char) {
        self.major_type = v;
    }
    fn set_internal_type(&mut self, v: u64) {
        self.internal_type = v;
    }
    fn set_goid(&mut self, goid: String) {
        self.goid = Some(goid);
    }
    fn set_coordinate(&mut self, coord: Coordinate) {
        self.coordinate = coord;
    }

    fn generate_goid(&self, coord: Coordinate) -> Self {
        let g = format!("X{}Y{}O{}S{}", coord.0, coord.1, self.major_type, self.internal_type);
        Self {
            goid: Some(g),
            ..*self
        }
    }
}
