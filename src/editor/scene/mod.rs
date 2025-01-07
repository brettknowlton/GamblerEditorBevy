use bevy::prelude::*;
use super::*;
use std::collections::HashMap;
use serde::{ Serialize, Deserialize };
use std::path::PathBuf;


pub use crate::utilities::*;

//Helper Components


#[derive(Serialize, Deserialize, Component, Debug, Clone, PartialEq, Default)]
///A Scene will manage and hold onto any and all of our editor objects,
///  upon the entity being added to the Scene, it will be serialized and saved as a String in  hashmap for later use
pub struct Scene {
    layout: HashMap<TCoordinate, Entity>,
    scene_path: PathBuf,
}

impl Scene {
    pub fn new(load_path: Option<PathBuf>) -> Self {
            match load_path.clone() {
                Some(path) => {
                    println!("Attempting to create scene from scene data from file: {path:?}");
                },
                None => {
                    println!("Path was not provided, starting with blank scene");
                },
            }
        let scene = Self::default();
        scene.read_and_deserialize(&load_path.unwrap())
    }

    pub fn push(&mut self, tcoord: TCoordinate, e: Entity) {
        self.layout.insert(tcoord, e);
    }

    pub fn remove(&mut self, object: TCoordinate) {
        self.layout.remove(&object);
    }

    pub fn get(&self, k: &TCoordinate) -> Option<&Entity> {
        self.layout.get(k)
    }

    pub fn serialize(&self) -> String {
        serde_json::to_string(&self.layout).unwrap_or_default()
    }

    fn from_data(bytes: String) -> Self {
        match serde_json::from_str::<Self>(&bytes) {
            Ok(scene_data) => scene_data,
            Err(e) => {
                println!("Reading scene data failed: {e:?}");
                println!("Continuing with blank scene");
                Self::default()
            }
        }
    }

    ///reads the provided path and tries to deserialize the contents into a Scene struct.
    /// If the file is not found, or if the file is not valid JSON, it will return an empty scene.
    pub fn read_and_deserialize(&self, path: &PathBuf) -> Self {

        let read_dir = std::fs::read(&path);
        match read_dir {
            Ok(file_bytes) => {
                let to_string = String::from_utf8(file_bytes).unwrap_or_default();
                Self::from_data(to_string)
            },
            Err(e) => {
                println!("Error reading file contents: {e:?}");
                println!("Continuing with blank scene");
                Self::default()
            }
        }
    }

    pub fn write_serialized_scene(&self, path: Option<PathBuf>) {
        //write all scene data to path's file, create a new file or overwite an existing one if it exists for now
        let p = if let Some(path) = path {
            path
        } else {
            PathBuf::from(DEFAULT_SCENE_PATH)
        };

        let mut data: Vec<serde_json::Value> = Vec::new();

        for e in &self.layout {
            info!("Gathering Entity for write: {:?}", e);
            data.push(serde_json::to_value(e).unwrap());
        }

        let json_data = serde_json::to_string(&data).unwrap();
        warn!("Saving Json payload:\n{}", json_data);

        todo!();

        write_json(&json_data, &p).expect("Issue Creating or Writing file");



        let write_result = std::fs::write(p.clone(), self.serialize());
        match write_result {
            Ok(_) => println!("Scene data written to file: {:?}", p),
            Err(e) => println!("Error writing scene data to file: {e:?}"),
        };
    }
}

fn write_json(json_string: &str, file_path: &str) -> Result<(), std::io::Error> {
    info!("Attempting to create file: {}", file_path);
    std::fs::write(file_path, json_string.as_bytes())?;
    info!("File Created Successfully");
    Ok(())
}


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
