use std::{fs::File, io::Write};
use super::*;
use bevy::{prelude::*, tasks::IoTaskPool};


pub fn scene_plugin(app: &mut App){
    app
        .add_systems(OnEnter(EditorState::Saving), (save_scene, return_state).chain());
}

fn return_state(mut next_state: ResMut<NextState<EditorState>>) {
    //change the state
    next_state.set(EditorState::Normal);
}

fn save_scene(world: &mut World){
    //save the existing world to a new scene ron file
    //create a new scene
    let mut saved_world = World::new();
    println!("World created");
    //copy only EditorObjects from the current world to the new world
    let mut query = world.query::<&EditorObject>();
    println!("Query created");
    for object in query.iter(&world) {
        println!("Object found");
        saved_world.spawn(object.clone());
    }
    println!("Objects copied");

    let scene = DynamicScene::from_world(&saved_world);
    println!("Successfully converted world to DynamicScene");

    // Scenes can be serialized like this:
    let type_registry = world.resource::<AppTypeRegistry>();
    println!("Sucessfully captured type registry");
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



//Helper Components


// #[derive(Serialize, Deserialize, Resource, Clone, PartialEq, Default)]
// ///A Scene will manage and hold onto any and all of our editor objects,
// ///  upon the entity being added to the Scene, it will be serialized and saved as a String in  hashmap for later use
// pub struct SceneManager {
//     scene: Option<Entity>,
//     scene_path: PathBuf,
// }

// impl SceneManager {
//     pub fn new(scene_path: PathBuf) -> Self {
//         Self {
//             scene: None,
//             scene_path,
//         }
//     }

//     pub fn from(scene: Entity) -> Self {
//         Self {
//             scene: Some(scene),
//             scene_path: PathBuf::from(""),
//         }
//     }

//     pub fn load_scene(&mut self, scene_path: PathBuf) {
//         self.scene_path = scene_path;
//         let scene_data = read_json(&self.scene_path.to_str().unwrap()).unwrap();
//         let scene: Scene = serde_json::from_str(&scene_data).unwrap();
//         // self.scene = Some(scene);
//     }

//     pub fn save_scene(&self) {
//         let scene_data = serde_json::to_string(&self.scene).unwrap();
//         write_json(&scene_data, &self.scene_path.to_str().unwrap()).unwrap();
//     }
// }

// //reads a json file and returns a string
// fn read_json(file_path: &str) -> Result<String, std::io::Error> {
//     info!("Attempting to read file: {}", file_path);
//     let file_contents = std::fs::read_to_string(file_path)?;
//     info!("File Read Successfully");
//     Ok(file_contents)
// }
// //writes a json formatted string to a file
// fn write_json(json_string: &str, file_path: &str) -> Result<(), std::io::Error> {
//     info!("Attempting to create file: {}", file_path);
//     std::fs::write(file_path, json_string.as_bytes())?;
//     info!("File Created Successfully");
//     Ok(())
// }


// #[derive(Component)]
// pub struct Scene {
//     data: HashMap<String, Box<dyn EditorObject>>,
// }

// impl Scene {
//     pub fn new() -> Self {
//         Self {
//             data: HashMap::new(),
//         }
//     }

//     pub fn push(&mut self, object: Box<dyn EditorObject>) {
//         self.data.insert(object.get_goid(), object);
//     }

//     pub fn remove(&mut self, location: Coordinate) {
//         // self.data.retain(|x| { x.clone().split_off(3) != format!("{}{}", location.0, location.1) });
//     }

//     pub fn get_data(&mut self) -> Option<Box<dyn EditorObject>> {
//         let read_dir = std::fs::read(format!("{:?}/test.json", env::current_dir()));
//         match read_dir {
//             Ok(file_bytes) => {
//                 let to_string = String::from_utf8(file_bytes).unwrap_or_default();
//                 let contents: std::result::Result<Box<dyn EditorObject>, Error> = serde_json::from_str(&to_string);
//                 if let Ok(file_contents) = contents {
//                     // println!("We got some file contents: {file_contents:?}");
//                     // let conversion =
//                 }
//             },
//             Err(e) => println!("Error reading file contents: {e:?}"),
//         }

//         None
//     }
// }
