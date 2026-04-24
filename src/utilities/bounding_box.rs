use bevy::{log::warn, prelude::Rect, reflect::Reflect};

use crate::coordinate::{Coordinate, CoordinateSpace, CoordinateFormatConversion};

/// A simple struct to represent a bounding box, which is a rectangle that can be used to check for collisions or to group objects together.
/// The bounding box is defined by its top-left and bottom-right coordinates, and can be used to check if a point is within the box or to combine two boxes into one that encompasses both.
/// Coordinates are expected to be in world space, and the bounding box is axis-aligned (not rotated).
#[derive(Debug, Clone, Copy, Reflect)]
pub struct BoundingBox {
    rect: Rect,
    coord: Coordinate,
}

impl Default for BoundingBox {
    fn default() -> Self {
        Self {
            rect: Rect::new(0.0, 0.0, 1.0, 1.0),
            coord: Coordinate::default(),
        }
    }
}

impl CoordinateFormatConversion for BoundingBox {
    fn into_points(&self) -> Vec<Coordinate> {
        vec![
            Coordinate::new(
                self.rect.min.x as i64,
                self.rect.min.y as i64,
                self.coord.format,
            ),
            Coordinate::new(
                self.rect.max.x as i64,
                self.rect.max.y as i64,
                self.coord.format,
            ),
            self.coord,
        ]
    }

    fn with_new_points(&self, points: Vec<Coordinate>) -> Self
    where
        Self: Sized,
    {
        if points.len() < 3 {
            warn!("Not enough points provided to create a bounding box - returning default");
            return Self::default();
        }
        let rect = Rect::from_corners(
            bevy::prelude::Vec2::new(points[0].x as f32, points[0].y as f32),
            bevy::prelude::Vec2::new(points[1].x as f32, points[1].y as f32),
        );

        Self {
            rect,
            coord: points[2],
        }
    }
}

impl BoundingBox {
    //creates a new bounding box of a default size at (0,0)
    pub fn new() -> Self {
        let rect = Rect::new(0.0, 0.0, 1.0, 1.0);
        Self {
            rect,
            coord: Coordinate::new_grid_space(0, 0),
        }
    }

    /// Translates the bounding box by the given amount(s)
    pub fn translate(&self, dx: f32, dy: f32, coord_space: CoordinateSpace) -> Self {
        let rect = Rect::from_corners(
            self.rect.min + bevy::prelude::Vec2::new(dx, dy),
            self.rect.max + bevy::prelude::Vec2::new(dx, dy),
        );
        Self {
            rect,
            coord: self.coord + Coordinate::new(dx as i64, dy as i64, coord_space),
        }
    }

    /// Creates a bounding box from two coordinates, treating them as opposite corners of the box
    pub fn from_coordinates(top_left: Coordinate, bottom_right: Coordinate) -> Self {
        let rect = Rect::from_corners(
            bevy::prelude::Vec2::new(top_left.x as f32, top_left.y as f32),
            bevy::prelude::Vec2::new(bottom_right.x as f32, bottom_right.y as f32),
        );
        Self {
            rect,
            coord: top_left,
        }
    }

    /// Creates a bounding box from a single coordinate,
    /// treating it as the top-left corner of a box with a default size (e.g., 1x1 or TILE_SIZE * TILE_SCALE x TILE_SIZE * TILE_SCALE)
    pub fn from_coordinate(coord: Coordinate) -> Self {
        let rect = Rect::from_corners(
            bevy::prelude::Vec2::new(coord.x as f32, coord.y as f32),
            bevy::prelude::Vec2::new((coord.x + 1) as f32, (coord.y + 1) as f32),
        );
        Self { rect, coord }
    }

    pub fn contains(&self, point: Coordinate) -> bool {
        self.rect
            .contains(bevy::prelude::Vec2::new(point.x as f32, point.y as f32))
    }

    pub fn combine(&self, other: &BoundingBox) -> Option<BoundingBox> {
        //ensure the rectangles are touching or overlapping
        if self.overlaps(other) && self.is_aligned_with(other) {
            //combine the rectangles into one that encompasses both
            let min_x = self.rect.min.x.min(other.rect.min.x);
            let min_y = self.rect.min.y.min(other.rect.min.y);
            let max_x = self.rect.max.x.max(other.rect.max.x);
            let max_y = self.rect.max.y.max(other.rect.max.y);

            let rect = Rect::from_corners(
                bevy::prelude::Vec2::new(min_x, min_y),
                bevy::prelude::Vec2::new(max_x, max_y),
            );

            Some(Self {
                rect,
                coord: self.coord,
            })
        } else {
            warn!("Attempted to combine non-overlapping bounding boxes - returning None");
            None
        }
    }

    pub fn is_aligned_with(&self, other: &BoundingBox) -> bool {
        //check if the rectangles are aligned on either the x or y axis
        let self_min = self.rect.min;
        let self_max = self.rect.max;
        let other_min = other.rect.min;
        let other_max = other.rect.max;

        if (self_min.x == other_min.x && self_max.x == other_max.x)
            || (self_min.y == other_min.y && self_max.y == other_max.y)
        {
            return true;
        }
        false
    }

    pub fn overlaps(&self, other: &BoundingBox) -> bool {
        let self_min = self.rect.min;
        let self_max = self.rect.max;
        let other_min = other.rect.min;
        let other_max = other.rect.max;

        if self_max.x < other_min.x || self_min.x > other_max.x {
            return false;
        }
        if self_max.y < other_min.y || self_min.y > other_max.y {
            return false;
        }
        true
    }
}
