use crate::actor::Actor;

use super::*;
use bevy::log::*;
use bevy::reflect::TypeRegistryArc;
use bevy::{ prelude::*, tasks::IoTaskPool };
use resources::*;
use std::time::Duration;
use std::{ fs::File, io::Write };

pub fn scene_plugin(app: &mut App) {
    app.register_type::<EditorObject>()
        .register_type::<Actor>()
        .register_type::<Tile>()
        .register_type::<TCoordinate>()
        .register_type::<Coordinate>()

        .add_systems(OnEnter(EditorState::LoadingEmpty), load_empty_scene)
        .add_systems(OnEnter(EditorState::Loading), return_state.after(load_empty_scene))
        .add_systems(OnEnter(EditorState::Loading), (load_scene, return_state.after(load_scene)))
        .add_systems(OnEnter(EditorState::Saving), (save_items, return_state.after(save_items)))
        .add_systems(
            Update,
            (spawn_sprites, add_missing_colliders)
                .chain()
                .run_if(not(in_state(EditorState::LoadAsk)))
        );
}

// struct MyGenericType<T>(PhantomData<T>);
fn load_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    //create scene manager component that will read/write our scene data between the enviornment and a json file
    commands.spawn(DynamicSceneRoot(asset_server.load(format!("{DEFAULT_SCENE_PATH}.ron"))));
}

fn add_missing_colliders(
    mut commands: Commands,
    editor_objects: Query<(Entity, &EditorObject), Without<Collider>>
) {
    for (entity, editor_object) in editor_objects.iter() {
        if editor_object.get_major_type() == 'c' {
            println!("Adding missing collider for EditorObject ID: {:?}", editor_object.coordinate);
    
            // Add a collider based on the EditorObject's properties
            commands.entity(entity).insert((
                Collider::cuboid(((TILE_SIZE / 2)) as f32, (TILE_SIZE / 2) as f32), Friction::coefficient(0.5)
            ));
        }
    }
}

fn spawn_sprites(
    mut tiles: Query<(Entity, &mut EditorObject), Without<Sprite>>,
    mut commands: Commands,
    spritesheets: Res<TextureHandles>
) {
    //spawn the sprites for each tile, use the editorObject's tcoords to determine the sprite's position
    //if the EditorObject has a tcoord beginning with 'T'
    for (entity, eo) in tiles.iter_mut() {
        if eo.get_major_type() == 't' {
            let sprite: Sprite = Sprite {
                image: spritesheets.0.get(&'t').unwrap().clone(),
                //the UVs are the same for every tile, just change the offset by using the tiletype as a multiplier
                rect: Some(Rect {
                    min: Vec2::new(
                        (((eo.get_internal_type() as u64) % SPRITESHEET_WIDTH) as f32) *
                            (TILE_SIZE as f32),
                        (((eo.get_internal_type() as u64) / SPRITESHEET_WIDTH) as f32) *
                            (TILE_SIZE as f32)
                    ),
                    max: Vec2::new(
                        (((eo.get_internal_type() as u64) % SPRITESHEET_WIDTH) as f32) *
                            (TILE_SIZE as f32) +
                            (TILE_SIZE as f32),
                        (((eo.get_internal_type() as u64) / SPRITESHEET_WIDTH) as f32) *
                            (TILE_SIZE as f32) +
                            (TILE_SIZE as f32)
                    ),
                }),
                anchor: Anchor::BottomLeft,
                ..default()
            };

            let coord = eo.get_coordinate().coord;

            commands
                .entity(entity)
                .insert((sprite, Visibility::default()))
                .entry::<Transform>()
                .and_modify(move |mut t| {
                    t.translation = Vec3::new(coord.0 as f32, coord.1 as f32, 0.0);
                });
        }
        if eo.get_major_type() == 'c' {
            let sprite: Sprite = Sprite {
                image: spritesheets.0.get(&'c').unwrap().clone(),
                rect: Some(Rect {
                    min: Vec2::new(0.0, 0.0),
                    max: Vec2::new(TILE_SIZE as f32, TILE_SIZE as f32),
                }),
                anchor: Anchor::Center,
                ..default()
            };

            let coord = eo.get_coordinate().coord;

            // commands
            //     .entity(entity)
            //     .insert(sprite)
            //     .entry::<Transform>()
            //     .and_modify(move |mut t| {
            //         t.translation = Vec3::new(
            //             (coord.0 as f32) + ((SCALED_TILE_WIDTH / 2) as f32),
            //             (coord.1 as f32) + ((SCALED_TILE_HEIGHT / 2) as f32),
            //             0.5
            //         );
            //     });
        }
    }
}

fn load_empty_scene(mut commands: Commands) {
    //create scene manager component that will read/write our scene data between the enviornment and a json file
    commands.spawn(DynamicSceneRoot(Handle::default()));
}

fn return_state(
    mut next_state: ResMut<NextState<EditorState>>,
    mut message_queue: ResMut<EditorBottomBarQueuedMessages>
) {
    //change the state
    next_state.set(EditorState::Normal);
    send_message!(
        Some('i'),
        message_queue,
        "FileIO Operations completed successfullly, returning to Normal Mode".to_string()
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

    //create a new world that will actually be saved
    let new_world = world;

    // despawn the entities from the new world that are not EditorObjects
    for t in filtered_objects.iter() {
        debug!("despawning non-serializable entity: {t:?} from the simulated world-to-save");
        new_world.despawn(**t);
    }

    //create a new scene from the new world that now only contains EditorObjects
    let scene = DynamicSceneBuilder::from_world(&new_world)
        .deny_all_resources()
        .deny_component::<Sprite>()
        .extract_entities(new_world.iter_entities().map(|e| e.id()))
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
