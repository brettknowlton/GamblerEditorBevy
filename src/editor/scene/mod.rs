use std::{fs::File, io::Write};

use super::*;
use bevy::{prelude::*, tasks::IoTaskPool};

pub fn scene_plugin(app: &mut App){
    app
        .add_systems(OnEnter(EditorState::LoadEmpty), load_empty_scene)
        .add_systems(OnEnter(EditorState::Loading), return_state.after(load_empty_scene))
        .add_systems(OnEnter(EditorState::Loading), load_scene)
        .add_systems(OnEnter(EditorState::Loading), return_state.after(load_scene))
        .add_systems(OnEnter(EditorState::Saving), save_items)
        .add_systems(OnEnter(EditorState::Saving), return_state.after(save_items));
}

// struct MyGenericType<T>(PhantomData<T>);
fn load_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    //create scene manager component that will read/write our scene data between the enviornment and a json file
    commands.spawn(DynamicSceneRoot(asset_server.load(format!("{DEFAULT_SCENE_PATH}.ron"))));
}

fn load_empty_scene(mut commands: Commands){
    //create scene manager component that will read/write our scene data between the enviornment and a json file
    commands.spawn(DynamicSceneRoot(Handle::default()));
}

fn return_state(mut next_state: ResMut<NextState<EditorState>>) {
    //change the state
    next_state.set(EditorState::Normal);
    println!("Saving operations complete, returning to normal state");
}

fn save_items(world: &mut World){

    // copy_entities_with_component::<EditorObject>(world, &mut new_world, eos);
    let type_registry = world.resource::<AppTypeRegistry>().clone();

    // sim_world.insert_resource(type_registry.clone());
    let objects = world.query::<Entity>().iter(world).map(|e| e).collect::<Vec<Entity>>();

    //filter out the entities that are not EditorObjects
    let filtered_objects = objects.iter().filter(|e| world.get::<EditorObject>(**e).is_none()).collect::<Vec<&Entity>>();

    //create a new world that will actually be saved
    let new_world = world;

    // despawn the entities from the new world that are not EditorObjects
    for t in filtered_objects.iter() {
        println!("despawning entity: {} from the simulated world-to-save", t);
       new_world.despawn(**t);
    }

    //create a new scene from the new world that now only contains EditorObjects
    let scene = DynamicSceneBuilder::from_world(&new_world)
        .deny_all_resources()
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