use std::{fs::File, io::Write, marker::PhantomData};
use super::*;
use bevy::{ecs::observer::TriggerTargets, prelude::*, reflect::DynamicTyped, tasks::IoTaskPool, text::cosmic_text::Edit};
use tile::Tile;

pub fn scene_plugin(app: &mut App){
    app
        .add_systems(Startup, initialize)
        // .add_systems(OnEnter(EditorState::Saving), save_items::<Tile>.)
        // .add_systems(OnEnter(EditorState::Saving), return_state.after(save_items::<Tile>.into()));
}

struct MyGenericType<T>(PhantomData<T>);

fn initialize(mut commands:Commands){
    //create a scene to store the scene's saveable data
    commands.spawn((SceneRoot::default(),));
}
fn stash_objects(tiles: Query<&Tile, With<EditorObject>>, mut scenes: Query<&SceneInstance>){
    let scene = scenes.iter().next().unwrap();

    for tile in tiles.iter(){
        scene.
    }

}

fn return_state(mut next_state: ResMut<NextState<EditorState>>) {
    //change the state
    next_state.set(EditorState::Normal);
}

fn save_items<T>(world: &mut World, objects: Query<&T, With<EditorObject>> ) where T: Component + Clone {

    let mut new_world = World::new();

    // copy_entities_with_component::<EditorObject>(world, &mut new_world, eos);
    let type_registry = world.resource::<AppTypeRegistry>().clone();
    // sim_world.insert_resource(type_registry.clone());

    //Add tiles to the new world
    for t in objects.iter() {
        new_world.spawn(()).insert(t.clone());
    }

    let scene = DynamicScene::from_world(&new_world);
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
            File::create(format!("{DEFAULT_SCENE_PATH}.ron"))
                .and_then(|mut file| file.write(serialized_scene.as_bytes()))
                .expect("Error while writing scene to file");
        })
        .detach();
    
}