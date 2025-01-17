use bevy::color::Srgba;

//WILD shit right? dont worry its not too bad.
//disable dead code warnings with #[warn(dead...)]

pub(crate)const 
                        WINDOW_TITLE:                                                                   &str
                        =
                            "Gambler";
///////////////////////////////////////
pub const WINDOW_TITLE2:&str= "GamblerEditor";

//Default color that the background will be replaced by if nothing is being rendered
pub const WINDOW_DEFAULT_BACKGROUND_COLOR: Srgba = Srgba::new(0.31, 0.643, 0.722, 1.);


pub const TILE_SCALE: usize = 4;
pub const TILE_SCALE_X: usize = TILE_SIZE * TILE_SCALE;
pub const TILE_SCALE_Y: usize = TILE_SIZE * TILE_SCALE;
pub const TILE_SIZE: usize = 16;

pub const SPRITESHEET_WIDTH: usize = 8;
pub const MAX_SPRITESHEET_ITEMS: usize = 32;


pub const UI_SCALE: usize = 10;
pub const UI_Z_LAYER: f32 = 10.;


pub const DEFAULT_WINDOW_WIDTH: f32 = 1200.;
pub const DEFAULT_WINDOW_HEIGHT: f32 = 800.;


pub const ASSETS_PATH: &str = "assets/";
pub const TEXTURES_PATH: &str = "assets/textures/";
pub const DEFAULT_SCENE_PATH: &str = "scenes/scene";


/*

alright what the hell does this mean? im sure hyrum just said that in his head lol

    ALL THE VARIABLES THAT NEVER CHANGE FOR THE ENRITE TIME THE GAME IS RUNNING:
    the space between things doesnt matter, but the order does, first entry is just an example of that


    pub(crate)const WINDOW_TITLE2:&str= "GamblerEditor";
    ignore this    -   The Name for coding it     -    the kind of thing      -    the value it holds
    pub(crate)const     WINDOW_TITLE:                   &str                        = "Gambler";


*/