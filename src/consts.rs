use bevy::color::Srgba;

//WILD shit right? dont worry its not too bad.
//disable dead code warnings with #[warn(dead...)]

pub(crate)const 
                        _WINDOW_TITLE:                                                                   &str
                        =
                            "Gambler";
///////////////////////////////////////
pub const WINDOW_TITLE2:&str= "GamblerEditor";

//Default color that the background will be replaced by if nothing is being rendered
pub const WINDOW_DEFAULT_BACKGROUND_COLOR: Srgba = Srgba::new(0.31, 0.643, 0.722, 1.);


pub const TILE_SCALE: usize = 4;//scale factor of tiles
pub const TILE_SIZE: usize = 32;//number of pixels wide the tile source image is

pub const SCALED_TILE_WIDTH: usize = TILE_SIZE * TILE_SCALE;
pub const SCALED_TILE_HEIGHT: usize = TILE_SIZE * TILE_SCALE;


pub const UI_SCALE: f32 = 10.;//a scale value we can use to scale UI elements
pub const UI_Z_LAYER: f32 = 10.;//z layer for UI elements, we can use this to make to relatively place UI elements correctly

pub const DEFAULT_TEXT_HEIGHT: f32 = 20.;//default height of text in pixels


pub const DEFAULT_WINDOW_WIDTH: f32 = 1200.;
pub const DEFAULT_WINDOW_HEIGHT: f32 = 800.;


pub const ASSETS_PATH: &str = "assets/";
pub const TEXTURES_PATH: &str = "assets/textures/";
pub const DEFAULT_SCENE_PATH: &str = "scenes/scene";

pub const SPRITESHEET_WIDTH: u64 = 8;//how many tiles wide our spritesheet is allowed to be
pub const MAX_SPRITESHEET_ITEMS: u64 = 32;


pub const UI_BORDER_PX: f32 = 2.;//border around the UI elements
pub const UI_BORDER_REAL: f32 = UI_BORDER_PX * UI_SCALE;//border around the UI elements as a fraction of the window height
/*

alright what the hell does this mean? im sure hyrum just said that in his head lol

    ALL THE VARIABLES THAT NEVER CHANGE FOR THE ENRITE TIME THE GAME IS RUNNING:
    the space between things doesnt matter, but the order does, first entry is just an example of that


    pub(crate)const WINDOW_TITLE2:&str= "GamblerEditor";
    ignore this    -   The Name for coding it     -    the kind of thing      -    the value it holds
    pub(crate)const     WINDOW_TITLE:                   &str                        = "Gambler";


*/