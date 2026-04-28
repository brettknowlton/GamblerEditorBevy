use bevy::reflect::Reflect;

mod anim_frame;

mod anim_def;

pub mod anim_map;

#[derive(Debug, Reflect, Clone)]
pub enum AnimBehavior {
    Loop,
    Once,
    PingPong,
}
