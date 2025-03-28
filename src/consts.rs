use bevy::color::Srgba;

//WILD shit right? dont worry its not too bad.
//disable dead code warnings with #[warn(dead...)]

pub(crate) const _WINDOW_TITLE: &str = "Gambler";
///////////////////////////////////////
pub const WINDOW_TITLE2: &str = "GamblerEditor";

//Default color that the background will be replaced by if nothing is being rendered
pub const WINDOW_DEFAULT_BACKGROUND_COLOR: Srgba = Srgba::new(0.31, 0.643, 0.722, 1.0);

pub const DEFAULT_GENERAL_SCALE_FACTOR: u32 = 2;

pub const ZONE_SIZE: u32 = 16; //width/height of a zone in tiles

pub const PLAYER_SIZE_X: u32 = 72;//pixels wide the player source image
pub const PLAYER_SIZE_Y: u32 = 90;//pixels tall the player source image
pub const PLAYER_SCALE: u32 = DEFAULT_GENERAL_SCALE_FACTOR;//by default player has normal scaling

pub const SCALED_PLAYER_WIDTH: u32 = PLAYER_SIZE_X * PLAYER_SCALE;//total pixel size the player width takes up IN GAME
pub const SCALED_PLAYER_HEIGHT: u32 = PLAYER_SIZE_Y * PLAYER_SCALE;//total pixel size the player height takes up IN GAME

pub const PLAYER_HB_X_OFFSET: u32 = SCALED_PLAYER_WIDTH / 3;
pub const PLAYER_HB_Y_OFFSET: u32 = SCALED_PLAYER_HEIGHT / 3;

pub const PLAYER_WALK_FORCE: u32 = 100;
pub const MAX_PLAYER_WALK_SPEED: u32 = 400;

pub const PLAYER_JUMP_FORCE: f32 = 5.;
pub const PLAYER_JUMP_GRACE_PERIOD: f32 = 0.2;//how long a vertical jump force can be applied to a player




pub const TILE_SCALE: u32 = DEFAULT_GENERAL_SCALE_FACTOR; //scale factor of tiles
pub const TILE_SIZE: u32 = 32; //number of pixels wide the tile source image is

pub const SCALED_TILE_WIDTH: u32 = TILE_SIZE * TILE_SCALE;
pub const SCALED_TILE_HEIGHT: u32 = SCALED_TILE_WIDTH;

pub const UI_SCALE: f32 = 10.0; //a scale value we can use to scale UI elements
pub const UI_Z_LAYER: f32 = 10.0; //z layer for UI elements, we can use this to make to relatively place UI elements correctly

pub const DEFAULT_TEXT_HEIGHT: f32 = 20.0; //default height of text in pixels

pub const DEFAULT_WINDOW_WIDTH: f32 = 1200.0;
pub const DEFAULT_WINDOW_HEIGHT: f32 = 800.0;

pub const ASSETS_PATH: &str = "assets/";
pub const TEXTURES_PATH: &str = "assets/textures/";
pub const DEFAULT_SCENE_PATH: &str = "scenes/scene";

pub const SPRITESHEET_WIDTH: u64 = 8; //how many tiles wide our spritesheet is allowed to be
pub const MAX_SPRITESHEET_ITEMS: u64 = 32;

pub const UI_BORDER_PX: f32 = 2.0; //border around the UI elements
pub const UI_BORDER_REAL: f32 = UI_BORDER_PX * UI_SCALE; //border around the UI elements as a fraction of the window height



//PHYSICS CONSTS
pub const GRAVITY: f32 = 20.;
pub const FRICTION: f32 = 0.1; //Friction of the ground, this is a value between 0 and 1, 0 is no friction, 1 is full friction

//Epsilon
pub const EPSILON: f32 = 0.0001;//A small value
/*

alright what the hell does this mean? im sure hyrum just said that in his head lol

    ALL THE VARIABLES THAT NEVER CHANGE FOR THE ENRITE TIME THE GAME IS RUNNING:
    the space between things doesnt matter, but the order does, first entry is just an example of that


    pub(crate)const WINDOW_TITLE2:&str= "GamblerEditor";
    ignore this    -   The Name for coding it     -    the kind of thing      -    the value it holds
    pub(crate)const     WINDOW_TITLE:                   &str                        = "Gambler";


*/
