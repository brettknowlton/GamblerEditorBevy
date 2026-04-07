use crate::actor::Actor;

use super::*;
use bevy::log::*;
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
        .add_systems(
            Update,
            (spawn_sprites, add_missing_colliders)
                .chain()
                .run_if(not(in_state(EditorState::LoadAsk))),
        );
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
                Collider::cuboid((TILE_SIZE / 2) as f32, (TILE_SIZE / 2) as f32),
                Friction::coefficient(0.5),
            ));
        }
    }
}

fn spawn_sprites(
    mut tiles: Query<(Entity, &mut EditorObject), Without<Sprite>>,
    mut commands: Commands,
    spritesheets: Res<TextureHandles>,
) {
    //spawn the sprites for each tile, use the editorObject's tcoords to determine the sprite's position
    //if the EditorObject has a tcoord beginning with 'T'
    for (entity, eo) in tiles.iter_mut() {
        if eo.get_major_type() == EditorObjectKind::Tile {
            let sprite_bundle = Sprite {
                image: spritesheets.0.get(&EditorObjectKind::Tile).unwrap().clone(),
                //the UVs are the same for every tile, just change the offset by using the tiletype as a multiplier
                rect: Some(Rect {
                    min: Vec2::new(
                        (((eo.get_internal_type() as u64) % SPRITESHEET_WIDTH) as f32)
                            * (TILE_SIZE as f32),
                        (((eo.get_internal_type() as u64) / SPRITESHEET_WIDTH) as f32)
                            * (TILE_SIZE as f32),
                    ),
                    max: Vec2::new(
                        (((eo.get_internal_type() as u64) % SPRITESHEET_WIDTH) as f32)
                            * (TILE_SIZE as f32)
                            + (TILE_SIZE as f32),
                        (((eo.get_internal_type() as u64) / SPRITESHEET_WIDTH) as f32)
                            * (TILE_SIZE as f32)
                            + (TILE_SIZE as f32),
                    ),
                }),

                ..default()
            };

            let coord = eo.get_coordinate();

            commands
                .entity(entity)
                .insert((sprite_bundle, Anchor::CENTER, Visibility::default()))
                .entry::<Transform>()
                .and_modify(move |mut t| {
                    t.translation = Vec3::new(
                        (coord.0 + (SCALED_TILE_WIDTH / 2) as i64) as f32,
                        (coord.1 + (SCALED_TILE_HEIGHT / 2) as i64) as f32,
                        t.translation.z,
                    );
                });
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

fn save_items(world: &mut World) {
    // copy_entities_with_component::<EditorObject>(world, &mut new_world, eos);
    let type_registry = world.resource::<AppTypeRegistry>().clone();

    // sim_world.insert_resource(type_registry.clone());
    let objects = world
        .query::<Entity>()
        .iter(world)
        .map(|e| e)
        .collect::<Vec<Entity>>();

    //filter out the entities that are not EditorObjects
    let filtered_objects = objects
        .iter()
        .filter(|e| world.get::<EditorObject>(**e).is_none())
        .collect::<Vec<&Entity>>();

    //create a new world (copy of orignial world) that will actually be saved
    let mut new_world =
        world
            .query::<Entity>()
            .iter(world)
            .map(|e| e)
            .fold(World::new(), |mut acc, e| {
                if let Some(editor_object) = world.get::<EditorObject>(e) {
                    acc.spawn(editor_object.clone());
                }
                acc
            });

    // despawn the entities from the new world that are not EditorObjects
    for t in filtered_objects.iter() {
        debug!("despawning non-serializable entity: {t:?} from the simulated world-to-save");
        new_world.despawn(**t);
    }

    //create a new scene from the new world that now only contains EditorObjects
    let scene = DynamicSceneBuilder::from_world(world)
        .deny_all_resources()
        .deny_component::<Sprite>()
        .extract_entities(&mut new_world.query::<Entity>().iter(&new_world).map(|e| e))
        .build();

    println!("Successfully converted world to DynamicScene");

    let type_registry = type_registry.read();
    println!("Registry read to RW lock");

    let serialized_scene = scene.serialize(&type_registry).unwrap();
    println!("Scene serialized");

    // Showing the scene in the console
    // info!("{}", serialized_scene);

    // Writing the scene to a new file. Using a task to avoid calling the filesystem APIs in a system
    // as they are blocking
    // This can't work in Wasm as there is no filesystem access
    #[cfg(not(target_arch = "wasm32"))]
    IoTaskPool::get()
        .spawn(async move {
            // Write the scene RON data to file
            File::create(format!("assets/{DEFAULT_SCENE_PATH}.ron"))
                .and_then(|mut file| file.write(serialized_scene.as_bytes()))
                .expect("Error while writing scene to file");
        })
        .detach();
}
