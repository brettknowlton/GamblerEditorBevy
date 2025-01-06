use std::path::PathBuf;

use bevy::prelude::*;

use crate::consts::*;
use crate::utilities::*;
use crate::utilities::resources::*;
use super::*;

use super::{ EditorObject, EditorMode };

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum TileModeState {
    #[default]
    Inactive,
    Active,
}

pub fn tilemode_plugin(app: &mut App) {
    app.init_state::<TileModeState>()
        .add_systems(OnEnter(EditorMode::Tile), init_tilemode)
        //placeholder for whatever tile we are trying to place
        .insert_resource(CurrentEditorObject(TileData::new()))

        //create the pop up menu for tile options

        //more keybinds for tile editing
        .add_systems(Update, tilemode_keybinds.run_if(in_state(TileModeState::Active)))
        .add_systems(OnExit(EditorMode::Tile), (
            exit_tilemode.before(tilemode_keybinds),
            despawn_all::<TileModeUI>.before(exit_tilemode),
        ));
}

#[derive(Component)]
struct TileModeUI;

fn init_tilemode(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut state: ResMut<NextState<TileModeState>>
) {
    println!("Entering Tile Editing Mode");
    state.set(TileModeState::Active);

    //display the "tilemode" menu
    let texpath = PathBuf::from("textures/menus/menu1.png");
    let tex1 = asset_server.load(texpath);

    commands.spawn((
        Sprite {
            image: tex1,
            ..default()
        },
        Transform {
            translation: Vec3::new(WINDOW_WIDTH / 2.0, WINDOW_HEIGHT / 2.0, 0.0),
            ..default()
        },
        TileModeUI,
    ));
}

fn tilemode_keybinds(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    crosshair: Query<&Crosshair>,

    mut scene: Query<&mut super::Scene>,

    mut state: ResMut<NextState<EditorMode>>,
    mut focused_item: ResMut<CurrentEditorObject>
) {
    if input.just_pressed(KeyCode::KeyP) {
        //get current location of the cursor and round to the nearest tile to get location of the new tile
        let location = crosshair.single().location;
        //add the currently selected tile to the scene at this location
        scene.single_mut().push(focused_item.0);
    }
    if input.just_pressed(KeyCode::KeyL) {
        scene.single_mut().remove(focused_item.0.coordinate);
    }
}

fn exit_tilemode(
    mut commands: Commands,
    mut tile_mode_uiitems: Query<&TileModeUI>,
    mut state: ResMut<NextState<EditorMode>>
) {
    state.set(EditorMode::Inactive);
    println!("Exiting Tile Editing Mode");

    //remove the CurrentEditorObject resource
    commands.insert_resource(CurrentEditorObject(TileData::new()));
}

#[derive(Component, Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct TileData {
    pub tile_type: u64,
    pub coordinate: Coordinate,
}

impl TileData {
    pub fn new() -> Self {
        Self {
            tile_type: 0,
            coordinate: Coordinate(0, 0),
        }
    }

    /**
     * Takes a coordinate of the currently
     * selected tile via the hovered crosshair
     * and adds it to the scene.
     */
    fn place(&self, coordinate: Coordinate, id: u64) {
        todo!()
    }

    fn get_coordinate(&self) -> Coordinate {
        todo!()
    }

    /**
     * This will be a file that we will parse through
     * the file at a given tile size some sprites,
     * we will ID them based on order received, which will
     * not ever change throughout the game unless we want to
     * change what the tile looks like
     */
    fn get_sprite(&mut self, tile_type: u64) -> anyhow::Result<Sprite, anyhow::Error> {
        todo!()
    }
}

impl EditorObject for TileData {
    fn get_goid(&self) -> String {
        format!("X{}Y{}O{}S{}", self.coordinate.0, self.coordinate.1, 'T', self.tile_type)
    }

    fn get_coordinate(&self) -> Coordinate {
        self.coordinate
    }
}
