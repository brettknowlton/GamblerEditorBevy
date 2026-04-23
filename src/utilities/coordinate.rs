use serde::{Deserialize, Serialize};

use bevy::prelude::*;

use crate::editor_object::EditorObjectKind;
use crate::{SCALED_TILE_WIDTH, TILE_SCALE, TILE_SIZE, ZONE_SIZE};

pub fn snap_value_to_grid(value: i64, grid_size: i64) -> i64 {
    //floor x and y values to the last multiple of grid_size
    //if coordinate is negative, this will round down to the nearest multiple of grid_size further negative
    let x: i64;
    if value < 0 {
        x = value - (grid_size + (value % grid_size));
    } else {
        x = value - (value % grid_size);
    }

    x
}

#[derive(
    Component, Reflect, Deserialize, Serialize, Debug, Hash, Eq, PartialEq, Clone, Copy, Default,
)]
pub enum CoordinateFormat {
    ///The default format is just the x and y values, this is used for most things, and is generally assumed to be in world space unless otherwise specified
    #[default]
    Undefined,
    Screen,         //raw screen coordinates, with (0, 0) being the top right of the screen
    ScreenCentered, //raw screen coordinates, with (0, 0) being the center of the screen
    Game, //raw screen coordinates, with (0, 0) being the center of the screen, but with the Y axis flipped
    GridSpace, //coordinates in grid space
    ZoneSpace, //coordinates in zone space
}

///A Coordinate is a simple struct that holds two i64 values, x and y identifying a point in our editor
/// most items are anchored to Bottom Left, so the x and y values (generally) define the bottom left corner of the object
#[derive(
    Component, Reflect, Deserialize, Serialize, Debug, Hash, Eq, PartialEq, Clone, Copy, Default,
)]
#[reflect(Component)]
pub struct Coordinate {
    pub x: i64,
    pub y: i64,
    format: CoordinateFormat,
}

impl Coordinate {
    pub fn new(x: i64, y: i64, format: CoordinateFormat) -> Self {
        Self { x, y, format }
    }

    pub fn screen(x: i64, y: i64) -> Self {
        Self {
            x,
            y,
            format: CoordinateFormat::Screen,
        }
    }

    pub fn screen_centered(x: i64, y: i64) -> Self {
        Self {
            x,
            y,
            format: CoordinateFormat::ScreenCentered,
        }
    }

    pub fn game(x: i64, y: i64) -> Self {
        Self {
            x,
            y,
            format: CoordinateFormat::Game,
        }
    }

    pub fn grid_space(x: i64, y: i64) -> Self {
        Self {
            x,
            y,
            format: CoordinateFormat::GridSpace,
        }
    }

    pub fn zone_space(x: i64, y: i64) -> Self {
        Self {
            x,
            y,
            format: CoordinateFormat::ZoneSpace,
        }
    }

    pub fn from(v: Vec3) -> Self {
        Self {
            x: v.x as i64,
            y: v.y as i64,
            format: CoordinateFormat::Undefined,
        }
    }

    pub fn add_tile_scale(&self) -> Self {
        Self {
            x: self.x + (TILE_SIZE as i64),
            y: self.y + (TILE_SIZE as i64),
            format: self.format,
        }
    }

    pub fn as_vec2(&self) -> Vec2 {
        Vec2::new(self.x as f32, self.y as f32)
    }

    pub fn snap_to_grid(&self) -> Coordinate {
        Coordinate {
            x: snap_value_to_grid(self.x, (TILE_SIZE * TILE_SCALE) as i64),
            y: snap_value_to_grid(self.y, (TILE_SIZE * TILE_SCALE) as i64),
            format: CoordinateFormat::GridSpace,
        }
    }

    fn screen_px_center(window_size: &Vec2) -> Vec2 {
        Vec2::new(window_size.x / 2.0, window_size.y / 2.0)
    }

    fn screen_to_centered(&self, window_size: &Vec2) -> Coordinate {
        assert!(
            self.format == CoordinateFormat::Screen,
            "This function can only convert screen coordinates to centered screen coordinates"
        );
        let half_screen = Coordinate::screen_px_center(window_size);
        Coordinate {
            x: self.x - half_screen.x as i64,
            y: self.y - half_screen.y as i64,
            format: CoordinateFormat::ScreenCentered,
        }
    }

    fn centered_to_game(&self, camera_transform: &Transform) -> Coordinate {
        assert!(
            self.format == CoordinateFormat::ScreenCentered,
            "This function can only convert centered screen coordinates to game coordinates"
        );
        Coordinate {
            x: self.x + camera_transform.translation.x as i64,
            y: -(self.y + camera_transform.translation.y as i64),
            format: CoordinateFormat::Game,
        }
    }

    fn game_to_grid_space(&self) -> Coordinate {
        assert!(
            self.format == CoordinateFormat::Game,
            "This function can only convert game coordinates to grid coordinates"
        );
        Coordinate {
            x: self.x.div_euclid(SCALED_TILE_WIDTH as i64),
            y: self.y.div_euclid(SCALED_TILE_WIDTH as i64),
            format: CoordinateFormat::GridSpace,
        }
    }

    fn grid_to_zone_space(&self) -> Coordinate {
        assert!(
            self.format == CoordinateFormat::GridSpace,
            "This function can only convert grid coordinates to zone coordinates"
        );
        Coordinate {
            x: self.x.div_euclid(ZONE_SIZE as i64),
            y: self.y.div_euclid(ZONE_SIZE as i64),
            format: CoordinateFormat::ZoneSpace,
        }
    }

    fn zone_to_grid_space(&self) -> Coordinate {
        assert!(
            self.format == CoordinateFormat::ZoneSpace,
            "This function can only convert zone coordinates to grid coordinates"
        );
        Coordinate {
            x: self.x * (ZONE_SIZE as i64 / (TILE_SIZE as i64)),
            y: self.y * (ZONE_SIZE as i64 / (TILE_SIZE as i64)),
            format: CoordinateFormat::GridSpace,
        }
    }

    fn grid_to_game_space(&self) -> Coordinate {
        assert!(
            self.format == CoordinateFormat::GridSpace,
            "This function can only convert grid coordinates to game coordinates"
        );
        Coordinate {
            x: self.x * (SCALED_TILE_WIDTH as i64),
            y: self.y * (SCALED_TILE_WIDTH as i64),
            format: CoordinateFormat::Game,
        }
    }

    fn game_to_centered(&self, camera_transform: &Transform) -> Coordinate {
        assert!(
            self.format == CoordinateFormat::Game,
            "This function can only convert game coordinates to centered screen coordinates"
        );
        //will need to take into account the camera's position in game space, as well as the fact that game space has the y axis flipped compared to screen space
        Coordinate {
            x: self.x - camera_transform.translation.x as i64,
            y: -(self.y - camera_transform.translation.y as i64),
            format: CoordinateFormat::ScreenCentered,
        }
    }

    fn centered_to_screen(&self, window_size: &Vec2) -> Coordinate {
        assert!(
            self.format == CoordinateFormat::ScreenCentered,
            "This function can only convert centered screen coordinates to screen coordinates"
        );
        let half_screen = Coordinate::screen_px_center(window_size);
        Coordinate {
            x: self.x + half_screen.x as i64,
            y: self.y + half_screen.y as i64,
            format: CoordinateFormat::Screen,
        }
    }

    pub fn convert(
        &self,
        target_format: CoordinateFormat,
        camera_transform: Option<&Transform>,
        window_size: Option<&Vec2>,
    ) -> Coordinate {
        match target_format {
            CoordinateFormat::Screen => self.as_screen_space(window_size, camera_transform),
            CoordinateFormat::ScreenCentered => {
                self.as_centered_space(window_size, camera_transform)
            }
            CoordinateFormat::Game => self.as_game_space(window_size, camera_transform),
            CoordinateFormat::GridSpace => self.as_grid_space(window_size, camera_transform),
            CoordinateFormat::ZoneSpace => self.as_zone_space(window_size, camera_transform),
            CoordinateFormat::Undefined => {
                warn!("Cannot convert to undefined coordinate format, returning self with format set to Undefined, this may cause issues later if this coordinate is used in calculations with other coordinates that are actually in different formats");
                Coordinate {
                    x: self.x,
                    y: self.y,
                    format: CoordinateFormat::Undefined,
                }
            }
        }
    }

    fn as_zone_space(
        &self,
        window_size: Option<&Vec2>,
        camera_transform: Option<&Transform>,
    ) -> Coordinate {
        match self.format {
            CoordinateFormat::ZoneSpace => *self,
            CoordinateFormat::GridSpace => self.grid_to_zone_space(),
            CoordinateFormat::Game => self.game_to_grid_space().grid_to_zone_space(),
            CoordinateFormat::Screen => self
                .screen_to_centered(&window_size.unwrap())
                .centered_to_game(&camera_transform.unwrap())
                .game_to_grid_space()
                .grid_to_zone_space(),
            CoordinateFormat::ScreenCentered => self
                .centered_to_game(&camera_transform.unwrap())
                .game_to_grid_space()
                .grid_to_zone_space(),
            CoordinateFormat::Undefined => {
                warn!("Cannot convert undefined coordinate to zone space without knowing the original format - assuming this is a grid space coordinate and converting to zone space accordingly, this may cause issues later if this coordinate is used in calculations with other coordinates that are actually in different formats");
                self.grid_to_zone_space()
            }
        }
    }

    fn as_grid_space(
        &self,
        window_size: Option<&Vec2>,
        camera_transform: Option<&Transform>,
    ) -> Coordinate {
        match self.format {
            CoordinateFormat::GridSpace => *self,
            CoordinateFormat::Game => self.game_to_grid_space(),
            CoordinateFormat::Screen => self
                .screen_to_centered(&window_size.unwrap())
                .centered_to_game(&camera_transform.unwrap())
                .game_to_grid_space(),
            CoordinateFormat::ScreenCentered => self
                .centered_to_game(&camera_transform.unwrap())
                .game_to_grid_space(),
            CoordinateFormat::ZoneSpace => self.zone_to_grid_space(),
            CoordinateFormat::Undefined => {
                warn!("Cannot convert undefined coordinate to grid space without knowing the original format - assuming this is a game coordinate and converting to grid space accordingly, this may cause issues later if this coordinate is used in calculations with other coordinates that are actually in different formats");
                self.game_to_grid_space()
            }
        }
    }

    fn as_game_space(
        &self,
        window_size: Option<&Vec2>,
        camera_transform: Option<&Transform>,
    ) -> Coordinate {
        match self.format {
            CoordinateFormat::Screen => self
                .screen_to_centered(&window_size.unwrap())
                .centered_to_game(&camera_transform.unwrap()),
            CoordinateFormat::ScreenCentered => self.centered_to_game(&camera_transform.unwrap()),
            CoordinateFormat::Game => *self,
            CoordinateFormat::GridSpace => self.grid_to_game_space(),
            CoordinateFormat::ZoneSpace => self.zone_to_grid_space().grid_to_game_space(),
            CoordinateFormat::Undefined => {
                warn!("Cannot convert undefined coordinate to game space without knowing the original format - assuming this is a grid space coordinate and converting to game space accordingly, this may cause issues later if this coordinate is used in calculations with other coordinates that are actually in different formats");
                self.grid_to_game_space()
            }
        }
    }

    fn as_centered_space(
        &self,
        window_size: Option<&Vec2>,
        camera_transform: Option<&Transform>,
    ) -> Coordinate {
        match self.format {
            CoordinateFormat::ScreenCentered => *self,
            CoordinateFormat::Screen => self.screen_to_centered(window_size.unwrap()),
            CoordinateFormat::Game => self.game_to_centered(camera_transform.unwrap()),
            CoordinateFormat::GridSpace => self
                .grid_to_game_space()
                .game_to_centered(camera_transform.unwrap()),
            CoordinateFormat::ZoneSpace => self
                .zone_to_grid_space()
                .grid_to_game_space()
                .game_to_centered(camera_transform.unwrap()),
            CoordinateFormat::Undefined => {
                warn!("Cannot convert undefined coordinate to centered screen space without knowing the original format - assuming this is a game coordinate and converting to centered screen space accordingly, this may cause issues later if this coordinate is used in calculations with other coordinates that are actually in different formats");
                self.game_to_centered(camera_transform.unwrap())
            }
        }
    }

    fn as_screen_space(
        &self,
        window_size: Option<&Vec2>,
        camera_transform: Option<&Transform>,
    ) -> Coordinate {
        match self.format {
            CoordinateFormat::Screen => *self,
            CoordinateFormat::ScreenCentered => self.centered_to_screen(window_size.unwrap()),
            CoordinateFormat::Game => self
                .game_to_centered(camera_transform.unwrap())
                .centered_to_screen(window_size.unwrap()),
            CoordinateFormat::GridSpace => self
                .grid_to_game_space()
                .game_to_centered(camera_transform.unwrap())
                .centered_to_screen(window_size.unwrap()),
            CoordinateFormat::ZoneSpace => self
                .zone_to_grid_space()
                .grid_to_game_space()
                .game_to_centered(camera_transform.unwrap())
                .centered_to_screen(window_size.unwrap()),
            CoordinateFormat::Undefined => {
                warn!("Cannot convert undefined coordinates to screen coordinates without knowing the original format - giving this coordinate SCREEN format but not changing the x and y values, this may cause issues later if this coordinate is used in calculations with other coordinates that are actually in screen space");
                Coordinate {
                    x: self.x,
                    y: self.y,
                    format: CoordinateFormat::Screen,
                }
            }
        }
    }
}

impl Into<bevy::prelude::Vec2> for Coordinate {
    fn into(self) -> Vec2 {
        self.as_vec2()
    }
}

///A TCoordinate, or a "typed coordinate" is a coordinate that also includes an identifying character,
///this way the coordinate is unique, as no two objects of the same type can occupy the same space,
/// and makes for an efficient Unique Identifier both objects AND zones
///
/// Objects Example:
///     TCoordinate { type_char: 'T', coord: Coordinate(0, 0) }
/// Object Types:
/// T: Tile
/// C: Collider
/// R: Editor Rect
/// P: Placeholder
///
///
/// Zones Example:
///    TCoordinate { type_char: 'F', coord: Coordinate(0, 0) }
/// Zone Types:
/// F: Foreground
/// B: Background
///
#[derive(Component, Reflect, Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
#[reflect(Component)]
pub struct TCoordinate {
    pub kind: EditorObjectKind,
    pub coord: Coordinate,
}
impl TCoordinate {
    pub fn new(kind: EditorObjectKind, coord: Coordinate) -> Self {
        Self { kind, coord }
    }

    pub fn print(&self) {
        println!(
            "TCoordinate {{ kind: {:?}, coord: {:?} }}",
            self.kind, self.coord
        );
    }
}

impl Default for TCoordinate {
    fn default() -> Self {
        Self {
            kind: EditorObjectKind::Tile,
            coord: Coordinate {
                x: 0,
                y: 0,
                format: CoordinateFormat::Game,
            },
        }
    }
}
