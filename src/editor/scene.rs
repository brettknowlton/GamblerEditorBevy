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
