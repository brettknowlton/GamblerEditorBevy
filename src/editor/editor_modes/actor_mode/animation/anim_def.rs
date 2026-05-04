use bevy::prelude::*;
use bevy::reflect::Reflect;

use super::AnimBehavior;

#[derive(Component, Debug, Reflect, Clone)]
pub struct AnimationDefenition {
    pub start_index: u32,
    pub stop_index: u32,
    pub current_frame_index: u32,
    pub frame_timer: f32,
    pub frame_duration: f32,
    pub behavior: AnimBehavior,
    pub pingpong_forward: bool,
}

impl AnimationDefenition {
    pub fn frame_count(&self) -> u32 {
        if self.stop_index < self.start_index {
            0
        } else {
            self.stop_index - self.start_index + 1
        }
    }

    pub fn current_atlas_index(&self) -> usize {
        (self.start_index + self.current_frame_index) as usize
    }

    pub fn reset(&mut self) {
        self.current_frame_index = 0;
        self.frame_timer = 0.0;
        self.pingpong_forward = true;
    }
}
