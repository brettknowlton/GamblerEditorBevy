use bevy::color::Srgba;

//WILD shit right? dont worry its not too bad.
//disable dead code warnings with #[warn(dead...)]

pub(crate) const _WINDOW_TITLE: &str = "Gambler";
///////////////////////////////////////
pub const WINDOW_TITLE2: &str = "GamblerEditor";

/// Default color that the background will be replaced by if nothing is being rendered
pub const WINDOW_DEFAULT_BACKGROUND_COLOR: Srgba = Srgba::new(0.31, 0.643, 0.722, 1.0);

/// Default scale factor for general elements
pub const DEFAULT_GENERAL_SCALE_FACTOR: u32 = 2;

/// width/height of a zone in tiles
pub const ZONE_SIZE: u32 = 16; 


/// Scale factor of tiles
pub const TILE_SCALE: u32 = DEFAULT_GENERAL_SCALE_FACTOR;
/// Number of pixels wide the tile source image is
pub const TILE_SIZE: u32 = 32;

/// Total pixel size the tile width takes up IN GAME
pub const SCALED_TILE_WIDTH: u32 = TILE_SIZE * TILE_SCALE;
/// Total pixel size the tile height takes up IN GAME
pub const SCALED_TILE_HEIGHT: u32 = TILE_SIZE * TILE_SCALE;


pub const DEFAULT_WINDOW_WIDTH: u32 = 1200;
pub const DEFAULT_WINDOW_HEIGHT: u32 = 800;

pub const ASSETS_PATH: &str = "assets/";
pub const TEXTURES_PATH: &str = "assets/textures/";
pub const DEFAULT_SCENE_PATH: &str = "scenes/scene";

pub const SPRITESHEET_WIDTH: u64 = 8; //how many tiles wide our spritesheet is allowed to be
pub const MAX_SPRITESHEET_ITEMS: u64 = 32;



//PHYSICS CONSTS
pub const GRAVITY: f32 = 12.; // applied per FixedUpdate tick (64hz) → effective ~768 units/sec²
pub const FRICTION: f32 = 0.1; //Friction of the ground, this is a value between 0 and 1, 0 is no friction, 1 is full friction

//Epsilon
pub const EPSILON: f32 = 0.01;//A small value
