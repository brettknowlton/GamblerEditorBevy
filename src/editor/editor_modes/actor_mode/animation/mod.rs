use std::time::Duration;

use bevy::prelude::*;

#[derive(Debug, Reflect, Clone, PartialEq, Eq)]
pub enum AnimBehavior {
    Loop,
    Once,
    PingPong,
}

/// Animation state that drives a Bevy [`Sprite`] with a [`TextureAtlas`] directly.
/// Attach this alongside `Sprite` on the sprite entity; `drive_sprite_animations` advances it.
#[derive(Component, Debug, Reflect, Clone)]
pub struct SpriteAnimation {
    pub first: u32,
    pub last: u32,
    pub fps: f32,
    pub behavior: AnimBehavior,
    timer: Timer,
    pingpong_forward: bool,
}

impl SpriteAnimation {
    pub fn new(first: u32, last: u32, fps: f32, behavior: AnimBehavior) -> Self {
        Self {
            first,
            last,
            fps,
            behavior,
            timer: Timer::new(
                Duration::from_secs_f32(1.0 / fps.max(0.001)),
                TimerMode::Repeating,
            ),
            pingpong_forward: true,
        }
    }

    /// Switch to a different frame range. No-op when already on the same range and behavior,
    /// so calling every frame is cheap.
    pub fn switch_to(&mut self, first: u32, last: u32, fps: f32, behavior: AnimBehavior) {
        if self.first == first && self.last == last && self.behavior == behavior {
            return;
        }
        *self = Self::new(first, last, fps, behavior);
    }

    fn next_index(&mut self, current_atlas_index: usize) -> usize {
        let local = (current_atlas_index as u32).saturating_sub(self.first);
        let count = self.last.saturating_sub(self.first) + 1;

        let next_local = match self.behavior {
            AnimBehavior::Loop => (local + 1) % count,
            AnimBehavior::Once => (local + 1).min(count - 1),
            AnimBehavior::PingPong => {
                if self.pingpong_forward {
                    if local + 1 >= count {
                        self.pingpong_forward = false;
                        local.saturating_sub(1)
                    } else {
                        local + 1
                    }
                } else if local == 0 {
                    self.pingpong_forward = true;
                    1.min(count - 1)
                } else {
                    local - 1
                }
            }
        };

        (self.first + next_local) as usize
    }
}

/// Advances every entity that has both a [`SpriteAnimation`] and a [`Sprite`] with a
/// [`TextureAtlas`]. Register once in your plugin; works for any animated sprite, not just the player.
pub fn drive_sprite_animations(
    time: Res<Time>,
    mut query: Query<(&mut SpriteAnimation, &mut Sprite)>,
) {
    for (mut anim, mut sprite) in query.iter_mut() {
        anim.timer.tick(time.delta());
        if !anim.timer.just_finished() {
            continue;
        }
        let Some(atlas) = &mut sprite.texture_atlas else {
            continue;
        };
        atlas.index = anim.next_index(atlas.index);
    }
}
