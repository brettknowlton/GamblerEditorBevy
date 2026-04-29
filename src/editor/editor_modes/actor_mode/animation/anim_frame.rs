use bevy::prelude::*;
use bevy::reflect::Reflect;

#[derive(Debug, Reflect, Clone)]
pub struct AnimationFrame {
    /// The UV rect for this frame, this is used to determine which part of the spritesheet to use when rendering this frame
    pub uv_rect: Rect,
    /// The duration of this frame in seconds, this is used to determine how long to display this frame before moving on to the next one
    pub duration: f32,
    ///The size in pixels of this frame in the animation, this is used to calculate the UV rect for each frame based on the layout
    pub frame_size: Vec2,
}

impl AnimationFrame {
    pub fn new() -> Self {
        Self {
            uv_rect: Rect::default(),
            duration: 0.1,
            frame_size: Vec2::new(1.0, 1.0),
        }
    }

    pub fn get_sprite(&self, spritesheet: &Handle<Image>) -> Sprite {
        //placeholder until we implement the animation map
        Sprite {
            image: spritesheet.clone(),
            rect: Some(self.uv_rect),
            ..Default::default()
        }
    }

    pub fn cut_from(sheet_layout: Vec2, index: u64, frame_size: Vec2, duration: f32) -> Self {
        let x = (index % sheet_layout.x as u64) as f32 * frame_size.x;
        let y = (index / sheet_layout.x as u64) as f32 * frame_size.y;
        let new_frame = Self {
            uv_rect: Rect {
                min: Vec2::new(x, y),
                max: Vec2::new(x + frame_size.x, y + frame_size.y),
            },
            duration,
            frame_size,
        };
        println!(
            "cutting frame at index {}: uv_rect: {:?}, frame_size: {:?}",
            index, new_frame.uv_rect, new_frame.frame_size
        );
        new_frame
    }
    pub fn with_duration(mut self, duration: f32) -> Self {
        self.duration = duration;
        self
    }

    pub fn as_sprite(&self, spritesheet: Handle<Image>) -> Sprite {
        Sprite {
            image: spritesheet,
            rect: Some(self.uv_rect),
            ..Default::default()
        }
    }
}
