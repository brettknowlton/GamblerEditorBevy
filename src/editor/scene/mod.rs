use crate::actor::Actor;

use super::*;
use bevy::{prelude::*, tasks::IoTaskPool};
use resources::*;
use std::{fs::File, io::Write};

pub fn scene_plugin(app: &mut App) {
    app.register_type::<EditorObject>()
        .register_type::<Actor>()
        .register_type::<Tile>()
        .register_type::<TCoordinate>()
        .register_type::<Coordinate>()

        .add_systems(
            OnEnter(EditorState::LoadingEmpty),
            (load_empty_scene, goto_normal_state).chain(),
        )

        .add_systems(
            OnEnter(EditorState::Loading),
            (load_scene, goto_normal_state).chain(),
        )

        .add_systems(
            OnEnter(EditorState::Saving),
            (save_items, goto_normal_state).chain(),
        )

        .add_systems(OnEnter(EditorState::SaveAsk), (add_save_ask_mode_kb).chain())
        .add_systems(
            OnExit(EditorState::SaveAsk),
            (remove_io_ask_mode_kb).chain(),
        )

        .add_systems(OnEnter(EditorState::LoadAsk), (add_load_ask_mode_kb).chain())
        .add_systems(
            OnExit(EditorState::LoadAsk),
            (remove_io_ask_mode_kb).chain(),
        )

        .add_systems(
            Update,
            (spawn_sprites)
                .chain()
                .run_if(not(in_state(EditorState::LoadAsk))),
        )
        .add_systems(
            Update,
            (add_missing_colliders)
                .chain()
                .run_if(not(in_state(EditorState::LoadAsk))),
        );
}

fn add_save_ask_mode_kb(mut available_keybinds: ResMut<AvailableKeybinds>) {
    available_keybinds.add_keycode(
        CustomInput::Multi(vec![KeyCode::KeyY, KeyCode::Enter]),
        "Save Scene".into(),
    );
    available_keybinds.add_keycode(
        CustomInput::Multi(vec![KeyCode::KeyN, KeyCode::Escape]),
        "Cancel".into(),
    );
}

fn add_load_ask_mode_kb(mut available_keybinds: ResMut<AvailableKeybinds>) {
    available_keybinds.add_keycode(
        CustomInput::Multi(vec![KeyCode::KeyY, KeyCode::Enter]),
        "Load Scene".into(),
    );
    available_keybinds.add_keycode(
        CustomInput::Multi(vec![KeyCode::KeyN, KeyCode::Escape]),
        "Cancel".into(),
    );
}

fn remove_io_ask_mode_kb(mut available_keybinds: ResMut<AvailableKeybinds>) {
    available_keybinds.clear()
}

// struct MyGenericType<T>(PhantomData<T>);
fn load_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    //create scene manager component that will read/write our scene data between the enviornment and a json file
    commands.spawn(DynamicSceneRoot(
        asset_server.load(format!("{DEFAULT_SCENE_PATH}.ron")),
    ));
}

fn add_missing_colliders(
    mut commands: Commands,
    editor_objects: Query<(Entity, &EditorObject), Without<Collider>>,
) {
    for (entity, editor_object) in editor_objects.iter() {
        if editor_object.get_major_type() == EditorObjectKind::Collider {
            println!(
                "Adding missing collider for EditorObject ID: {:?}",
                editor_object.coordinate
            );

            // Add a collider based on the EditorObject's properties
            commands.entity(entity).insert((
                Collider::cuboid(
                    ((TILE_SIZE / 2) * TILE_SCALE) as f32,
                    ((TILE_SIZE / 2) * TILE_SCALE) as f32,
                ),
                Friction::coefficient(0.5),
                Transform {
                    translation: Vec3::new(
                        (editor_object.coordinate.0 + (SCALED_TILE_WIDTH / 2) as i64) as f32,
                        (editor_object.coordinate.1 + (SCALED_TILE_HEIGHT / 2) as i64) as f32,
                        -5.0,
                    ),
                    ..default()
                },
            ));
        }
    }
}

fn spawn_sprites(
    mut tiles: Query<(Entity, &mut EditorObject), (With<Tile>, Without<Sprite>)>,
    mut commands: Commands,
    spritesheets: Res<TextureHandles>,
) {
    //spawn the sprites for each tile, use the editorObject's kind and coordinate to determine the sprite's position
    //if the EditorObject has a kind of Tile
    for (entity, eo) in tiles.iter_mut() {
        if eo.kind == EditorObjectKind::Tile {
            let rect = Some(Tile::get_tex_rect(eo.get_internal_type() as u64));

            let sprite = Sprite {
                image: spritesheets.0.get(&EditorObjectKind::Tile).unwrap().clone(),
                //the UVs are the same for every tile, just change the offset by using the tiletype as a multiplier
                rect,
                custom_size: Some(Vec2::splat(TILE_SIZE as f32 * TILE_SCALE as f32)),

                ..default()
            };

            // calculate the position for the Transform component, this will be in the center of the item's hitbox locked to the grid
            let pos = Vec3::new(
                (eo.coordinate.0 + (SCALED_TILE_WIDTH / 2) as i64) as f32,
                (eo.coordinate.1 + (SCALED_TILE_HEIGHT / 2) as i64) as f32,
                -5.0,
            );
            println!("item's position offset calculated: {:?}", pos);

            commands.entity(entity).insert((
                sprite,
                Tile::from_rect(rect.unwrap(), eo.coordinate),
                Visibility::default(),
                Transform {
                    translation: pos,
                    scale: Vec3::new((TILE_SCALE / 2) as f32, (TILE_SCALE / 2) as f32, 1.0),
                    ..default()
                },
                eo.clone(),
            ));
        }
    }
}

fn load_empty_scene(mut commands: Commands) {
    //create scene manager component that will read/write our scene data between the enviornment and a json file
    commands.spawn(DynamicSceneRoot(Handle::default()));
}

fn goto_normal_state(
    mut next_state: ResMut<NextState<EditorState>>,
    mut message_queue: ResMut<EditorBottomBarQueuedMessages>,
) {
    //change the state
    next_state.set(EditorState::Normal);
    send_message!(
        Some('i'),
        message_queue,
        "FileIO Operations completed, returning to Normal Mode".to_string()
    );
}

fn serialize_editor_scene(world: &mut World, type_registry: &AppTypeRegistry) -> String {
    let mut query = world.query_filtered::<Entity, With<EditorObject>>();

    let scene = DynamicSceneBuilder::from_world(world)
        .deny_all()
        .allow_component::<EditorObject>()
        .allow_component::<Tile>()
        .allow_component::<Collider>()
        .allow_component::<Actor>()
        .extract_entities(query.iter(world))
        .build();

    println!("Successfully converted world to DynamicScene");

    let type_registry = type_registry.read();
    println!("Registry read to RW lock");

    let serialized_scene = scene.serialize(&type_registry).unwrap();
    println!("Scene serialized");

    serialized_scene
}

fn write_serialized_scene(serialized_scene: String) {
    #[cfg(not(target_arch = "wasm32"))]
    IoTaskPool::get()
        .spawn(async move {
            File::create(format!("assets/{DEFAULT_SCENE_PATH}.ron"))
                .and_then(|mut file| file.write(serialized_scene.as_bytes()))
                .expect("Error while writing scene to file");
        })
        .detach();
}

fn save_items(world: &mut World) {
    let type_registry = world.resource::<AppTypeRegistry>().clone();

    let serialized_scene = serialize_editor_scene(world, &type_registry);
    write_serialized_scene(serialized_scene);
}
