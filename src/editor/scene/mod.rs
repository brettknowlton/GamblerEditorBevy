use bevy::prelude::*;
use serde_json::Error;
use std::{ collections::HashMap, path::Path };
use serde::{ Serialize, Deserialize };
use std::path::PathBuf;

pub use crate::utilities::*;

//Helper Components
#[derive(Component, Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct TCoordinate {
    pub object_type: char,
    pub coord: Coordinate,
}
///A TCoordinate, or a "typed coordinate" is a coordinate that also includes the

#[derive(Serialize, Deserialize, Component, Debug, Clone, PartialEq, Default)]
pub struct Scene {
    items: HashMap<TCoordinate, String>,
    scene_path: PathBuf,
}

impl Scene {
    pub fn new(&self, load_path: Option<PathBuf>) -> Self {
            match load_path.clone() {
                Some(path) => {
                    println!("Attempting to create scene from scene data from file: {path:?}");
                },
                None => {
                    println!("Path was not provided, starting with blank scene");
                },
            }
        self.read_and_deserialize(&load_path.unwrap())
    }


    pub fn push(&mut self, object: TCoordinate, goid: String) {
        self.items.insert(object, goid);
    }

    pub fn remove(&mut self, object: TCoordinate) {
        self.items.remove(&object);
    }

    pub fn serialize(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
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

    pub fn write_serialized_scene(&self, path: String) {
        //write all scene data to path's file, create a new file or overwite an existing one if it exists for now
        let write_path = PathBuf::from(&path);
        let write_result = std::fs::write(write_path.clone(), self.serialize());
        match write_result {
            Ok(_) => println!("Scene data written to file: {:?}", write_path),
            Err(e) => println!("Error writing scene data to file: {e:?}"),
        };
    }
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
