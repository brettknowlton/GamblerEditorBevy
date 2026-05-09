use crate::editor_modes::{
    actor_mode::actor::Actor,
    collider_mode::ColliderObject,
    significant_component::SignificantComponent,
    tile_mode::{TileID, TileObject},
    EditorObject, EditorObjectKind,
};

use super::*;
use bevy::{prelude::*, tasks::IoTaskPool};
use bevy::camera::visibility::RenderLayers;
use resources::*;
use std::{fs::File, io::Write, path::Path};
use crate::rendering::TILE_PASS_LAYER;

pub fn scene_plugin(app: &mut App) {
    app.register_type::<EditorObject>()
        .register_type::<Actor>()
        .register_type::<TileObject>()
        .register_type::<ColliderObject>()
        .register_type::<crate::editor_modes::normal_mode::NormalObject>()
        .register_type::<crate::editor_modes::selection::SelectionRect>()
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
        .add_systems(
            OnEnter(EditorState::SaveAsk),
            (add_save_ask_mode_kb).chain(),
        )
        .add_systems(
            OnExit(EditorState::SaveAsk),
            (remove_io_ask_mode_kb).chain(),
        )
        .add_systems(
            OnEnter(EditorState::LoadAsk),
            (add_load_ask_mode_kb).chain(),
        )
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
        )
        .add_systems(Update, redraw_scene_visuals.run_if(on_message::<RedrawScene>));
}

fn redraw_scene_visuals(
    mut commands: Commands,
    mut bottom_bar: ResMut<MessageDisplay>,
    mut redraw_hint: ResMut<crate::ui::SceneRedrawHint>,
    tiles_with_sprite: Query<Entity, (With<TileObject>, With<Sprite>)>,
) {
    let mut refreshed = 0usize;
    for entity in tiles_with_sprite.iter() {
        commands.entity(entity).remove::<Sprite>();
        refreshed += 1;
    }

    bottom_bar.send_message(format!(
        "Scene re-draw recommended: refreshed {refreshed} tile sprites"
    ));
    redraw_hint.required = false;
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
fn load_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut bottom_bar: ResMut<MessageDisplay>,
) {
    //create scene manager component that will read/write our scene data between the enviornment and a json file
    let scene_path = format!("assets/{DEFAULT_SCENE_PATH}.ron");
    if !Path::new(&scene_path).exists() {
        bottom_bar.send_message(format!(
            "Scene file not found at \"{}\". Loading current world instead.",
            scene_path
        ));
        return;
    }

    if let Err(err) = std::fs::read_to_string(&scene_path) {
        bottom_bar.send_message(format!(
            "Failed to read scene file \"{}\": {}",
            scene_path, err
        ));
        return;
    }

    commands.spawn(DynamicSceneRoot(
        asset_server.load(format!("{DEFAULT_SCENE_PATH}.ron")),
    ));
}

fn add_missing_colliders(
    mut commands: Commands,
    editor_objects: Query<(
        Entity,
        &EditorObject,
        Option<&ColliderObject>,
        Option<&Collider>,
    )>,
) {
    let mut collider_count: u32 = 0;
    for (entity, editor_object, collider_object, physics_collider) in editor_objects.iter() {
        if editor_object.get_major_type() != EditorObjectKind::Collider {
            continue;
        }

        let mut ecmd = commands.entity(entity);

        if physics_collider.is_none() {
            info!(
                "Adding missing Rapier collider for EditorObject ID: {:?}",
                editor_object.coordinate
            );
            collider_count += 1;
            ecmd.insert((
                Collider::cuboid(
                    ((TILE_SIZE / 2) * TILE_SCALE) as f32,
                    ((TILE_SIZE / 2) * TILE_SCALE) as f32,
                ),
                Friction::coefficient(0.5),
                Transform {
                    translation: Vec3::new(
                        (editor_object.coordinate.x + (SCALED_TILE_WIDTH / 2) as i64) as f32,
                        (editor_object.coordinate.y + (SCALED_TILE_HEIGHT / 2) as i64) as f32,
                        -5.0,
                    ),
                    ..default()
                },
            ));
        }

        if collider_object.is_none() {
            let coord = editor_object.coordinate;

            debug!(
                "Adding missing ColliderObject marker for EditorObject ID: {:?}",
                editor_object.coordinate
            );
            ecmd.insert(ColliderObject::at_coordinate(coord));
        }
    }
    if collider_count > 0 {
        debug!("Added {} missing colliders to the scene.", collider_count);
    }
}

fn spawn_sprites(
    mut tiles: Query<(Entity, &mut EditorObject), (With<TileObject>, Without<Sprite>)>,
    rendered_tiles: Query<Entity, (With<TileObject>, With<Sprite>)>,
    mut commands: Commands,
    spritesheets: Res<TextureHandles>,
    mut redraw_hint: ResMut<crate::ui::SceneRedrawHint>,
) {
    let had_rendered_tiles = !rendered_tiles.is_empty();
    let mut spawned_any = false;
    //spawn the sprites for each tile, use the editorObject's kind and coordinate to determine the sprite's position
    //if the EditorObject has a kind of Tile
    for (entity, eo) in tiles.iter_mut() {
        if let EditorObjectKind::Tile(TileID::Some(id)) = eo.kind {
            let rect = Some(TileObject::get_uv_rect(id));

            let sprite = if let Some(image) =
                spritesheets.0.get(&EditorObjectKind::Tile(TileID::Any))
            {
                Sprite {
                    image: image.clone(),
                    rect,
                    custom_size: Some(Vec2::splat(TILE_SIZE as f32 * TILE_SCALE as f32)),
                    ..default()
                }
            } else {
                panic!(
                    "Texture for EditorObjectKind::Tile(TileID::Some({})) not found in TextureHandles",
                    id
                );
            };

            // calculate the position for the Transform component, this will be in the center of the item's hitbox locked to the grid
            let pos = Vec3::new(
                (eo.coordinate.x + (SCALED_TILE_WIDTH / 2) as i64) as f32,
                (eo.coordinate.y + (SCALED_TILE_HEIGHT / 2) as i64) as f32,
                -5.0,
            );
            println!("item's position offset calculated: {:?}", pos);

            commands.entity(entity).insert((
                sprite,
                TileObject::at_coordinate(eo.coordinate).with_id(eo.kind),
                Visibility::default(),
                RenderLayers::layer(TILE_PASS_LAYER),
                Transform {
                    translation: pos,
                    scale: Vec3::new((TILE_SCALE / 2) as f32, (TILE_SCALE / 2) as f32, 1.0),
                    ..default()
                },
                eo.clone(),
            ));
            spawned_any = true;
        }
    }

    if spawned_any && had_rendered_tiles {
        redraw_hint.required = true;
    }
}

fn load_empty_scene(
    mut commands: Commands,
    editor_entities: Query<Entity, With<EditorObject>>,
    scene_roots: Query<Entity, With<DynamicSceneRoot>>,
) {
    // Clear prior placed content so "empty" is actually empty (MCP / reload would otherwise stack tiles).
    let editors: Vec<Entity> = editor_entities.iter().collect();
    for e in editors {
        commands.entity(e).despawn();
    }
    let roots: Vec<Entity> = scene_roots.iter().collect();
    for e in roots {
        commands.entity(e).despawn();
    }
    commands.spawn(DynamicSceneRoot(Handle::default()));
}

fn goto_normal_state(
    mut next_state: ResMut<NextState<EditorState>>,
    mut bottom_bar: ResMut<MessageDisplay>,
    mut reset_message_writer: MessageWriter<ResetScene>,
) {
    //change the state
    next_state.set(EditorState::Normal);
    bottom_bar.send_message("FileIO Operations completed, returning to Normal Mode");
    reset_message_writer.write(ResetScene);
}

fn serialize_editor_scene(world: &mut World, type_registry: &AppTypeRegistry) -> String {
    let mut query = world.query_filtered::<Entity, With<EditorObject>>();

    let scene = DynamicSceneBuilder::from_world(world)
        .deny_all()
        .allow_component::<EditorObject>()
        .allow_component::<TileObject>()
        .allow_component::<ColliderObject>()
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
