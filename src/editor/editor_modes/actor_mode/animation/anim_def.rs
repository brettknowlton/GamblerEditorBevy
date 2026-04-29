use bevy::prelude::*;
use bevy::reflect::Reflect;

use super::anim_frame::AnimationFrame;
use super::AnimBehavior;

#[derive(Component, Debug, Reflect, Clone)]
pub struct AnimationDefenition {
    /// A vector of frames, each frame is an option in case we create an animation map before we have all the frames ready,
    /// this allows us to set the current animation to one that is not fully defined yet and fill in the frames later
    pub frames: Vec<AnimationFrame>,
    pub spritesheet: Handle<Image>,
    /// The animation's position in the greater spritesheet,
    pub start_end: Vec2,

    /// The layout of the frames in the spritesheet, this is used to calculate the UV rect for each frame based on the frame size and the start position
    pub frame_count: u32,
    pub current_frame_index: u32,
    pub frame_timer: f32,

    /// The behavior of the animation when it reaches the end,
    /// this is used to determine what to do when the animation reaches the end of its frames
    pub behavior: AnimBehavior,
}

impl AnimationDefenition {
    pub fn get_current_frame(&self) -> &AnimationFrame {
        if let Some(frame) = self.frames.get(self.current_frame_index as usize) {
            frame
        } else {
            panic!(
                "Current frame index out of bounds. Current index: {}, frames length: {}",
                self.current_frame_index,
                self.frames.len()
            );
        }
    }
}
