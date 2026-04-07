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

pub const PLAYER_WALK_FORCE: u32 = 200;
pub const MAX_PLAYER_WALK_SPEED: u32 = 300;

pub const PLAYER_JUMP_FORCE: f32 = 550.;
pub const PLAYER_JUMP_GRACE_PERIOD: f32 = 0.3;//how long a vertical jump force can be applied to a player


pub const TILE_SCALE: usize = 4;
pub const TILE_SCALE_X: usize = TILE_SIZE * TILE_SCALE;
pub const TILE_SCALE_Y: usize = TILE_SIZE * TILE_SCALE;
pub const TILE_SIZE: usize = 16;

pub const SPRITESHEET_WIDTH: usize = 8;
pub const MAX_SPRITESHEET_ITEMS: usize = 32;
pub const DEFAULT_SCENE_PATH: &str = "scenes/scene";

pub const UI_SCALE: usize = 10;

pub const WINDOW_WIDTH: f32 = 1200.;
pub const WINDOW_HEIGHT: f32 = 800.;
pub const ASSETS_PATH: &str = "assets/";
pub const TEXTURES_PATH: &str = "assets/textures/";


/*

alright what the hell does this mean? im sure hyrum just said that in his head lol

    ALL THE VARIABLES THAT NEVER CHANGE FOR THE ENRITE TIME THE GAME IS RUNNING:
    the space between things doesnt matter, but the order does, first entry is just an example of that


    pub(crate)const WINDOW_TITLE2:&str= "GamblerEditor";
    ignore this    -   The Name for coding it     -    the kind of thing      -    the value it holds
    pub(crate)const     WINDOW_TITLE:                   &str                        = "Gambler";


*/