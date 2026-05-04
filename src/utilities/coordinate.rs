use serde::{Deserialize, Serialize};

use std::ops::Add;

use bevy::prelude::*;

use crate::{SCALED_TILE_WIDTH, TILE_SCALE, TILE_SIZE, ZONE_SIZE};

/// Snap to the containing grid cell using Euclidean floor math.
///
/// This preserves exact multiples on both sides of the axis:
/// - `64 -> 64`
/// - `-64 -> -64`
/// - `-1 -> -64`
pub fn snap_value_to_grid(value: i64, grid_size: i64) -> i64 {
    assert!(grid_size > 0, "grid_size must be > 0");
    value.div_euclid(grid_size) * grid_size
}

/// Keeps both raw world input and snapped output so placement logic can reason
/// about the original coordinate and the grid-aligned coordinate independently.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoordinateSnapResult {
    pub world: Coordinate,
    pub snapped: Coordinate,
}

#[derive(
    Component, Reflect, Deserialize, Serialize, Debug, Hash, Eq, PartialEq, Clone, Copy, Default,
)]
pub enum CoordinateSpace {
    ///The default format is just the x and y values, this is used for most things, and is generally assumed to be in world space unless otherwise specified
    #[default]
    Undefined,
    Screen,         //raw screen coordinates, with (0, 0) being the top right of the screen
    ScreenCentered, //raw screen coordinates, with (0, 0) being the center of the screen
    WorldSpace, //raw screen coordinates, with (0, 0) being the center of the screen, but with the Y axis flipped
    GridSpace,  //coordinates in grid space
    ZoneSpace,  //coordinates in zone space
}

pub trait CoordinateFormatConversion {
    fn convert(
        &self,
        target_format: CoordinateSpace,
        camera_transform: Option<&GlobalTransform>,
        window_size: Option<&Vec2>,
    ) -> Self
    where
        Self: Sized,
    {
        match target_format {
            CoordinateSpace::Screen => self.as_screen_space(window_size, camera_transform),
            CoordinateSpace::ScreenCentered => {
                self.as_centered_space(window_size, camera_transform)
            }
            CoordinateSpace::WorldSpace => self.as_world_space(window_size, camera_transform),
            CoordinateSpace::GridSpace => self.as_grid_space(window_size, camera_transform),
            CoordinateSpace::ZoneSpace => self.as_zone_space(window_size, camera_transform),
            CoordinateSpace::Undefined => {
                warn!("Cannot convert to undefined coordinate format, returning self with format set to Undefined, this may cause issues later if this coordinate is used in calculations with other coordinates that are actually in different formats");
                self.with_format(CoordinateSpace::Undefined)
            }
        }
    }

    fn into_points(&self) -> Vec<Coordinate>;

    fn with_new_points(&self, points: Vec<Coordinate>) -> Self;

    /// Creates a copy of this item with the coordinate format changed to the given format, without changing the x and y values,
    /// this is useful for when you know the coordinate is actually in a specific format but it is currently set to Undefined,
    /// this allows you to change the format without having to convert the x and y values,
    /// which may cause issues if the coordinate is actually in a different format than you think
    fn with_format(&self, format: CoordinateSpace) -> Self
    where
        Self: Sized,
    {
        let points = self
            .into_points()
            .iter_mut()
            .map(|point| {
                *point = Coordinate {
                    x: point.x,
                    y: point.y,
                    format,
                };
                *point
            })
            .collect::<Vec<Coordinate>>();
        self.with_new_points(points)
    }

    /// Adds the tile scale to the x and y values of this coordinate, without changing the format
    /// useful for when you want to get the coordinate of the opposite corner of a tile, given the coordinate of one corner
    /// panics if the coordinate is not in world space, as this is only intended to be used for world space coordinates that are snapped to the grid,
    /// which should always be in multiples of the tile size
    fn add_tile_scale(&self) -> Self
    where
        Self: Sized,
    {
        assert!(
            self.into_points()[0].format == CoordinateSpace::WorldSpace,
            "Can only add tile scale to coordinates in world space format, but got {:?}",
            self.into_points()[0].format
        );

        let points = self
            .into_points()
            .iter_mut()
            .map(|point| {
                *point = Coordinate {
                    x: point.x + (TILE_SIZE as i64 * TILE_SCALE as i64),
                    y: point.y + (TILE_SIZE as i64 * TILE_SCALE as i64),
                    format: point.format,
                };
                *point
            })
            .collect::<Vec<Coordinate>>();

        self.with_new_points(points)
    }

    fn screen_px_center(window_size: &Vec2) -> Vec2 {
        Vec2::new(window_size.x / 2.0, window_size.y / 2.0)
    }

    fn as_zone_space(
        &self,
        window_size: Option<&Vec2>,
        camera_transform: Option<&GlobalTransform>,
    ) -> Self
    where
        Self: Sized,
    {
        let points = self.into_points();
        let mut new_points: Vec<Coordinate> = vec![];

        for point in points {
            let new_point = match point.format {
                CoordinateSpace::ZoneSpace => point,
                CoordinateSpace::GridSpace => point.grid_to_zone_space(),
                CoordinateSpace::WorldSpace => point.game_to_grid_space().grid_to_zone_space(),
                CoordinateSpace::Screen => point
                    .screen_to_centered(&window_size.unwrap())
                    .centered_to_game(camera_transform.unwrap())
                    .game_to_grid_space()
                    .grid_to_zone_space(),
                CoordinateSpace::ScreenCentered => point
                    .centered_to_game(camera_transform.unwrap())
                    .game_to_grid_space()
                    .grid_to_zone_space(),
                CoordinateSpace::Undefined => {
                    warn!("Cannot convert undefined coordinate to zone space without knowing the original format - assuming this is a grid space coordinate and converting to zone space accordingly, this may cause issues later if this coordinate is used in calculations with other coordinates that are actually in different formats");
                    point.grid_to_zone_space()
                }
            };
            new_points.push(new_point);
        }
        self.with_new_points(new_points)
    }

    fn as_grid_space(
        &self,
        window_size: Option<&Vec2>,
        camera_transform: Option<&GlobalTransform>,
    ) -> Self
    where
        Self: Sized,
    {
        let points = self.into_points();
        let mut new_points: Vec<Coordinate> = vec![];

        for point in points {
            let new_point = match point.format {
                CoordinateSpace::GridSpace => point,
                CoordinateSpace::WorldSpace => point.game_to_grid_space(),
                CoordinateSpace::Screen => point
                    .screen_to_centered(&window_size.unwrap())
                    .centered_to_game(&camera_transform.unwrap())
                    .game_to_grid_space(),
                CoordinateSpace::ScreenCentered => point
                    .centered_to_game(&camera_transform.unwrap())
                    .game_to_grid_space(),
                CoordinateSpace::ZoneSpace => point.zone_to_grid_space(),
                CoordinateSpace::Undefined => {
                    warn!("Cannot convert undefined coordinate to grid space without knowing the original format - assuming this is a game coordinate and converting to grid space accordingly, this may cause issues later if this coordinate is used in calculations with other coordinates that are actually in different formats");
                    point.game_to_grid_space()
                }
            };
            new_points.push(new_point);
        }

        self.with_new_points(new_points)
    }

    fn as_world_space(
        &self,
        window_size: Option<&Vec2>,
        camera_transform: Option<&GlobalTransform>,
    ) -> Self
    where
        Self: Sized,
    {
        let points = self.into_points();
        let mut new_points: Vec<Coordinate> = vec![];

        for point in points {
            let new_point = match point.format {
                CoordinateSpace::Screen => point
                    .screen_to_centered(&window_size.unwrap())
                    .centered_to_game(&camera_transform.unwrap()),
                CoordinateSpace::ScreenCentered => {
                    point.centered_to_game(&camera_transform.unwrap())
                }
                CoordinateSpace::WorldSpace => point,
                CoordinateSpace::GridSpace => point.grid_to_game_space(),
                CoordinateSpace::ZoneSpace => point.zone_to_grid_space().grid_to_game_space(),
                CoordinateSpace::Undefined => {
                    warn!("Cannot convert undefined coordinate to game space without knowing the original format - assuming this is a grid space coordinate and converting to game space accordingly, this may cause issues later if this coordinate is used in calculations with other coordinates that are actually in different formats");
                    point.grid_to_game_space()
                }
            };
            new_points.push(new_point);
        }

        self.with_new_points(new_points)
    }

    fn as_centered_space(
        &self,
        window_size: Option<&Vec2>,
        camera_transform: Option<&GlobalTransform>,
    ) -> Self
    where
        Self: Sized,
    {
        let points = self.into_points();
        let mut new_points: Vec<Coordinate> = vec![];

        for point in points {
            let new_point = match point.format {
                CoordinateSpace::ScreenCentered => point,
                CoordinateSpace::Screen => point.screen_to_centered(window_size.unwrap()),
                CoordinateSpace::WorldSpace => point.game_to_centered(camera_transform.unwrap()),
                CoordinateSpace::GridSpace => point
                    .grid_to_game_space()
                    .game_to_centered(camera_transform.unwrap()),
                CoordinateSpace::ZoneSpace => point
                    .zone_to_grid_space()
                    .grid_to_game_space()
                    .game_to_centered(camera_transform.unwrap()),
                CoordinateSpace::Undefined => {
                    warn!("Cannot convert undefined coordinate to centered screen space without knowing the original format - assuming this is a game coordinate and converting to centered screen space accordingly, this may cause issues later if this coordinate is used in calculations with other coordinates that are actually in different formats");
                    point.game_to_centered(camera_transform.unwrap())
                }
            };
            new_points.push(new_point);
        }

        self.with_new_points(new_points)
    }

    fn as_screen_space(
        &self,
        window_size: Option<&Vec2>,
        camera_transform: Option<&GlobalTransform>,
    ) -> Self
    where
        Self: Sized,
    {
        let points = self.into_points();
        let mut new_points: Vec<Coordinate> = vec![];

        for point in points {
            let new_point = match point.format {
                CoordinateSpace::Screen => point,
                CoordinateSpace::ScreenCentered => point.centered_to_screen(window_size.unwrap()),
                CoordinateSpace::WorldSpace => point
                    .game_to_centered(camera_transform.unwrap())
                    .centered_to_screen(window_size.unwrap()),
                CoordinateSpace::GridSpace => point
                    .grid_to_game_space()
                    .game_to_centered(camera_transform.unwrap())
                    .centered_to_screen(window_size.unwrap()),
                CoordinateSpace::ZoneSpace => point
                    .zone_to_grid_space()
                    .grid_to_game_space()
                    .game_to_centered(camera_transform.unwrap())
                    .centered_to_screen(window_size.unwrap()),
                CoordinateSpace::Undefined => {
                    warn!("Cannot convert undefined coordinates to screen coordinates without knowing the original format - giving this coordinate SCREEN format but not changing the x and y values, this may cause issues later if this coordinate is used in calculations with other coordinates that are actually in screen space");
                    point.with_format(CoordinateSpace::Undefined)
                }
            };
            new_points.push(new_point);
        }
        self.with_new_points(new_points)
    }

    /// modifies all points for this item to be in screen space,
    /// this is useful for when you want to use the coordinate for UI purposes, but you should be careful when using this to make sure you know what format
    /// this coordinate is actually in before calling this function,
    /// as it may cause issues if you use it in calculations with other coordinates that are in different formats
    fn in_screen_space(&self) -> Self
    where
        Self: Sized,
    {
        self.convert(CoordinateSpace::Screen, None, None)
    }

    /// Creates a new Coordinate in centered screen space with the given x and y values
    fn in_screen_centered(&self) -> Self
    where
        Self: Sized,
    {
        self.convert(CoordinateSpace::ScreenCentered, None, None)
    }

    /// Creates a new Coordinate in world space with the given x and y values
    fn in_world_space(&self) -> Self
    where
        Self: Sized,
    {
        self.convert(CoordinateSpace::WorldSpace, None, None)
    }

    /// Creates a new Coordinate in grid space with the given x and y values
    fn in_grid_space(&self) -> Self
    where
        Self: Sized,
    {
        self.convert(CoordinateSpace::GridSpace, None, None)
    }

    /// Creates a new Coordinate in zone space with the given x and y values
    fn in_zone_space(&self) -> Self
    where
        Self: Sized,
    {
        self.convert(CoordinateSpace::ZoneSpace, None, None)
    }

    fn get_format(&self) -> CoordinateSpace {
        self.into_points()[0].format
    }

    fn translate(&self, dx: f32, dy: f32, coord_space: CoordinateSpace) -> Self
    where
        Self: Sized,
    {
        //first, convert to the desired coordinate space, then translate, then convert back to the original coordinate space
        let original_format = self.get_format();

        let scaled = self.convert(coord_space, None, None);

        let points = scaled.into_points();

        let new_points = points
            .iter()
            .map(|point| Coordinate {
                x: point.x + dx as i64,
                y: point.y + dy as i64,
                format: coord_space,
            })
            .collect::<Vec<Coordinate>>();

        self.with_new_points(new_points)
            .convert(original_format, None, None)
    }
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
    pub format: CoordinateSpace,
}

impl Add for Coordinate {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        assert!(
            self.format == rhs.format,
            "Cannot add two Coordinates with different formats, got {:?} and {:?}",
            self.format,
            rhs.format
        );
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            format: self.format,
        }
    }
}

impl CoordinateFormatConversion for Coordinate {
    fn into_points(&self) -> Vec<Coordinate> {
        return vec![*self];
    }

    fn with_new_points(&self, points: Vec<Coordinate>) -> Self
    where
        Self: Sized,
    {
        assert!(
            points.len() == 1,
            "Currently only supports converting single-point coordinates, but got {} points",
            points.len()
        );
        points[0]
    }
}

impl Coordinate {
    /// Creates a new Coordinate with the given x and y values and format
    pub fn new(x: i64, y: i64, format: CoordinateSpace) -> Self {
        Self { x, y, format }
    }

    pub fn new_screen_space(x: i64, y: i64) -> Self {
        Self::new(x, y, CoordinateSpace::Screen)
    }

    /// Creates a new Coordinate in centered screen space with the given x and y values
    pub fn new_screen_centered(x: i64, y: i64) -> Self {
        Self::new(x, y, CoordinateSpace::ScreenCentered)
    }

    /// Creates a new Coordinate in world space with the given x and y values
    pub fn new_world_space(x: i64, y: i64) -> Self {
        Self::new(x, y, CoordinateSpace::WorldSpace)
    }

    /// Creates a new Coordinate in grid space with the given x and y values
    pub fn new_grid_space(x: i64, y: i64) -> Self {
        Self::new(x, y, CoordinateSpace::GridSpace)
    }

    /// Creates a new Coordinate in zone space with the given x and y values
    pub fn new_zone_space(x: i64, y: i64) -> Self {
        Self::new(x, y, CoordinateSpace::ZoneSpace)
    }

    /// Creates a new Coordinate from a Vec3, with the format set to Undefined
    pub fn from_vec3(v: Vec3) -> Self {
        Self::new(v.x as i64, v.y as i64, CoordinateSpace::Undefined)
    }

    pub fn from_vec2(v: Vec2) -> Self {
        Self::new(v.x as i64, v.y as i64, CoordinateSpace::Undefined)
    }

    fn screen_to_centered(&self, window_size: &Vec2) -> Coordinate {
        assert!(
            self.format == CoordinateSpace::Screen,
            "This function can only convert screen coordinates to centered screen coordinates"
        );
        let half_screen = Coordinate::screen_px_center(window_size);
        Coordinate {
            x: self.x - half_screen.x as i64,
            y: self.y - half_screen.y as i64,
            format: CoordinateSpace::ScreenCentered,
        }
    }

    fn centered_to_game(&self, camera_transform: &GlobalTransform) -> Coordinate {
        assert!(
            self.format == CoordinateSpace::ScreenCentered,
            "This function can only convert centered screen coordinates to game coordinates"
        );
        Coordinate {
            x: self.x + camera_transform.translation().x as i64,
            y: -(self.y + camera_transform.translation().y as i64),
            format: CoordinateSpace::WorldSpace,
        }
    }

    fn game_to_grid_space(&self) -> Coordinate {
        assert!(
            self.format == CoordinateSpace::WorldSpace,
            "This function can only convert game coordinates to grid coordinates"
        );
        Coordinate {
            x: self.x.div_euclid(SCALED_TILE_WIDTH as i64),
            y: self.y.div_euclid(SCALED_TILE_WIDTH as i64),
            format: CoordinateSpace::GridSpace,
        }
    }

    fn grid_to_zone_space(&self) -> Coordinate {
        assert!(
            self.format == CoordinateSpace::GridSpace,
            "This function can only convert grid coordinates to zone coordinates"
        );
        Coordinate {
            x: self.x.div_euclid(ZONE_SIZE as i64),
            y: self.y.div_euclid(ZONE_SIZE as i64),
            format: CoordinateSpace::ZoneSpace,
        }
    }

    fn zone_to_grid_space(&self) -> Coordinate {
        assert!(
            self.format == CoordinateSpace::ZoneSpace,
            "This function can only convert zone coordinates to grid coordinates"
        );
        Coordinate {
            x: self.x * (ZONE_SIZE as i64 / (TILE_SIZE as i64)),
            y: self.y * (ZONE_SIZE as i64 / (TILE_SIZE as i64)),
            format: CoordinateSpace::GridSpace,
        }
    }

    fn grid_to_game_space(&self) -> Coordinate {
        assert!(
            self.format == CoordinateSpace::GridSpace,
            "This function can only convert grid coordinates to game coordinates"
        );
        Coordinate {
            x: self.x * (SCALED_TILE_WIDTH as i64),
            y: self.y * (SCALED_TILE_WIDTH as i64),
            format: CoordinateSpace::WorldSpace,
        }
    }

    fn game_to_centered(&self, camera_transform: &GlobalTransform) -> Coordinate {
        assert!(
            self.format == CoordinateSpace::WorldSpace,
            "This function can only convert game coordinates to centered screen coordinates"
        );
        //will need to take into account the camera's position in game space, as well as the fact that game space has the y axis flipped compared to screen space
        Coordinate {
            x: self.x - camera_transform.translation().x as i64,
            y: -(self.y - camera_transform.translation().y as i64),
            format: CoordinateSpace::ScreenCentered,
        }
    }

    fn centered_to_screen(&self, window_size: &Vec2) -> Coordinate {
        assert!(
            self.format == CoordinateSpace::ScreenCentered,
            "This function can only convert centered screen coordinates to screen coordinates"
        );
        let half_screen = Coordinate::screen_px_center(window_size);
        Coordinate {
            x: self.x + half_screen.x as i64,
            y: self.y + half_screen.y as i64,
            format: CoordinateSpace::Screen,
        }
    }

    /// Converts this coordinate to a Vec2, ignoring the format,
    /// this is useful for when you want to use this coordinate in calculations that don't care about the format,
    /// but you should be careful when using this to make sure you know what format this coordinate is actually in,
    /// as it may cause issues if you use it in calculations with other coordinates that are in different formats
    pub fn as_vec2(&self) -> Vec2 {
        Vec2::new(self.x as f32, self.y as f32)
    }

    /// Snaps this coordinate to the nearest grid point, based on the TILE_SIZE and TILE_SCALE constants, without changing the format
    /// will panic if given coordinate is not in world space.
    pub fn snap_to_grid(&self) -> Coordinate {
        assert!(
            self.format == CoordinateSpace::WorldSpace,
            "Can only snap coordinates in WorldSpace format to the grid, but got {:?}",
            self.format
        );

        Coordinate {
            x: snap_value_to_grid(self.x, (TILE_SIZE * TILE_SCALE) as i64),
            y: snap_value_to_grid(self.y, (TILE_SIZE * TILE_SCALE) as i64),
            format: CoordinateSpace::GridSpace,
        }
    }

    /// Returns both the original world coordinate and the snapped coordinate.
    pub fn snap_to_grid_with_world_tracking(&self) -> CoordinateSnapResult {
        assert!(
            self.format == CoordinateSpace::WorldSpace,
            "Can only snap coordinates in WorldSpace format to the grid, but got {:?}",
            self.format
        );

        CoordinateSnapResult {
            world: *self,
            snapped: self.snap_to_grid(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snap_preserves_exact_negative_multiples() {
        assert_eq!(snap_value_to_grid(-64, 64), -64);
        assert_eq!(snap_value_to_grid(-512, 64), -512);
    }

    #[test]
    fn snap_floors_toward_negative_infinity() {
        assert_eq!(snap_value_to_grid(-1, 64), -64);
        assert_eq!(snap_value_to_grid(63, 64), 0);
        assert_eq!(snap_value_to_grid(64, 64), 64);
    }

    #[test]
    fn track_world_and_snapped_coordinates() {
        let world = Coordinate::new_world_space(-1, -1);
        let tracked = world.snap_to_grid_with_world_tracking();

        assert_eq!(tracked.world, world);
        assert_eq!(tracked.snapped.x, -64);
        assert_eq!(tracked.snapped.y, -64);
        assert_eq!(tracked.snapped.format, CoordinateSpace::GridSpace);
    }
}

impl Into<bevy::prelude::Vec2> for Coordinate {
    fn into(self) -> Vec2 {
        self.as_vec2()
    }
}
