use std::ops::DerefMut;

use crate::{
    DEFAULT_GENERAL_SCALE_FACTOR, EPSILON, FRICTION, GRAVITY, actor_mode::animation::{AnimBehavior, SpriteAnimation}, direction::Direction, rendering::MainWorldCamera
};

const SHEET_COLS: u32 = 18;
const ANIM_FPS: f32 = 10.0;
const ANIM_IDLE_FIRST: u32 = 0;
const ANIM_IDLE_LAST: u32 = 9;
const ANIM_RUN_FIRST: u32 = 10;
const ANIM_RUN_LAST: u32 = 17; // atlas has 18 frames (0-17); 10-17 = 8 run frames

use bevy::sprite::Anchor;
use bevy::camera::visibility::RenderLayers;
use bevy_rapier2d::prelude::{Collider, RigidBody};
use crate::rendering::PLAYER_PASS_LAYER;

use super::*;
/// Width of the player source image in pixels
pub const PLAYER_SIZE_X: u32 = 27;

/// Height of the player source image in pixels
pub const PLAYER_SIZE_Y: u32 = 38;

/// Scale factor for the player
pub const PLAYER_SCALE: u32 = DEFAULT_GENERAL_SCALE_FACTOR; //by default player has normal scaling

/// Total pixel size the player width takes up IN GAME
pub const SCALED_PLAYER_WIDTH: u32 = PLAYER_SIZE_X * PLAYER_SCALE;
/// Total pixel size the player height takes up IN GAME
pub const SCALED_PLAYER_HEIGHT: u32 = PLAYER_SIZE_Y * PLAYER_SCALE;

/// Horizontal offset for the player's hitbox
pub const PLAYER_HB_X_OFFSET: u32 = SCALED_PLAYER_WIDTH / 4;

/// Vertical offset for the player's hitbox
pub const PLAYER_HB_Y_OFFSET: u32 = SCALED_PLAYER_HEIGHT / 4;

/// Force applied to the player when walking
pub const PLAYER_WALK_FORCE: u32 = 400;
/// Maximum walking speed for the player
pub const MAX_PLAYER_WALK_SPEED: u32 = 500;

/// Force applied to the player when jumping
pub const PLAYER_JUMP_FORCE: f32 = 700.;
/// How long a vertical jump force can be applied to a player
pub const PLAYER_JUMP_GRACE_PERIOD: f32 = 0.3;
/// Extra gravity multiplier while falling to keep descent snappy without reducing jump apex.
pub const PLAYER_FALL_GRAVITY_MULTIPLIER: f32 = 1.8;

#[derive(Component, Debug, Reflect)]
pub enum PlayerState {
    Idle,
    Walking,
    Running,
    Attacking,
    Hurt,
    Dead,
}

#[derive(Component, Debug, Reflect)]
pub struct Player {
    pub state: PlayerState,
    pub facing: Direction,
    pub air_timer: f32,
    pub velocity: Vec2,

    pub trying_jump: bool,
    pub jump_queued: bool,
    pub trying_walk_left: bool,
    pub trying_walk_right: bool,
}

#[derive(Component, Debug, Default, Reflect)]
pub struct PlayerSpriteTag;

impl Player {
    pub fn new() -> Self {
        Self {
            state: PlayerState::Idle,
            facing: Direction::Right,
            air_timer: 0.0,
            velocity: Vec2::ZERO,
            trying_jump: false,
            jump_queued: false,
            trying_walk_left: false,
            trying_walk_right: false,
        }
    }

    pub fn player_controls(
        keyboard_input: Res<ButtonInput<KeyCode>>,
        mut query: Query<&mut Player>,
    ) {
        for mut player in query.iter_mut() {
            // Capture raw intent in Update so quick key taps are not missed between fixed ticks.
            player.trying_jump = keyboard_input.pressed(KeyCode::KeyW);
            if keyboard_input.just_pressed(KeyCode::KeyW) {
                player.jump_queued = true;
            }

            player.trying_walk_left = keyboard_input.pressed(KeyCode::KeyA);
            player.trying_walk_right = keyboard_input.pressed(KeyCode::KeyD);
        }
    }

    pub fn player_physics(
        mut player_q: Single<(
            &mut Player,
            &KinematicCharacterControllerOutput,
            &mut KinematicCharacterController,
        )>,
        time: Res<Time>,
    ) {
        let (player, kpco, controller) = player_q.deref_mut();

        // Reset/increment air timer in fixed-step with physics state.
        if kpco.grounded {
            player.air_timer = 0.0;
        } else {
            player.air_timer += time.delta_secs();
        }

        // Apply jump once in fixed-step so gameplay is deterministic.
        if player.jump_queued && (kpco.grounded || player.air_timer < PLAYER_JUMP_GRACE_PERIOD) {
            player.velocity.y = PLAYER_JUMP_FORCE;
            player.jump_queued = false;
        }

        // Horizontal acceleration from intent.
        let horizontal_axis =
            (player.trying_walk_right as i8 - player.trying_walk_left as i8) as f32;
        if horizontal_axis != 0.0 {
            player.velocity.x +=
                horizontal_axis * PLAYER_WALK_FORCE as f32 * time.delta_secs() * 64.0;
        }

        if kpco.grounded {
            //ON GROUND PHYSICS

            //fix y velocity so the player doesnt drop off edges
            if player.velocity.y < -1.0 {
                player.velocity.y = -1.0;
            }
            //clamp velocity.x to max walk speed if on ground
            if player.velocity.x > MAX_PLAYER_WALK_SPEED as f32 {
                player.velocity.x = MAX_PLAYER_WALK_SPEED as f32;
            } else if player.velocity.x < -(MAX_PLAYER_WALK_SPEED as f32) {
                player.velocity.x = -(MAX_PLAYER_WALK_SPEED as f32);
            }
        } else {
            //IN AIR PHYSICS
            //apply gravity to the player if in the air
            let gravity_scale;
            if player.trying_jump && player.air_timer < PLAYER_JUMP_GRACE_PERIOD {
                //while trying to jump higher we suspend the effect of gravity somewhat
                gravity_scale = 0.1;
            } else if player.velocity.y < 0.0 {
                gravity_scale = PLAYER_FALL_GRAVITY_MULTIPLIER;
            } else {
                gravity_scale = 1.0;
            }
            let g = Vec2::new(0.0, -(GRAVITY as f32) * gravity_scale);

            //apply a small amount of friction to the player if in the air
            //clamp velocity.x to double max walk speed if on ground
            if player.velocity.x > 1.5 * MAX_PLAYER_WALK_SPEED as f32 {
                player.velocity.x = 1.5 * MAX_PLAYER_WALK_SPEED as f32;
            } else if player.velocity.x < -(1.5 * MAX_PLAYER_WALK_SPEED as f32) {
                player.velocity.x = -(1.5 * MAX_PLAYER_WALK_SPEED as f32);
            }

            player.velocity += g;
        }

        //apply friction to the player not trying to move
        if !(player.trying_walk_left || player.trying_walk_right) {
            let f = 1.0 - (FRICTION / 4.);
            player.velocity.x *= f;
        }

        controller.translation = Some(player.velocity * time.delta_secs());

        //if velocity value has fallen below EPSILON then set it to 0
        if player.velocity.x.abs() < EPSILON {
            player.velocity.x = 0.0;
        }
        if player.velocity.y.abs() < EPSILON {
            player.velocity.y = 0.0;
        }

        // Clear consumed jump presses after physics consumes intent.
        if !player.trying_jump {
            player.jump_queued = false;
        }
    }

    /// Reads movement state from the physics output and switches the child sprite's animation.
    pub fn update_player_animation_state(
        mut player_query: Query<(&mut Player, &KinematicCharacterControllerOutput, &Children)>,
        mut anim_query: Query<&mut SpriteAnimation, With<PlayerSpriteTag>>,
    ) {
        for (mut player, kpco, children) in player_query.iter_mut() {
            let is_moving = kpco.grounded && player.velocity.x.abs() > EPSILON;
            player.state = if is_moving {
                PlayerState::Walking
            } else {
                PlayerState::Idle
            };

            for child in children.iter() {
                if let Ok(mut anim) = anim_query.get_mut(child) {
                    if is_moving {
                        anim.switch_to(ANIM_RUN_FIRST, ANIM_RUN_LAST, ANIM_FPS, AnimBehavior::Loop);
                    } else {
                        anim.switch_to(ANIM_IDLE_FIRST, ANIM_IDLE_LAST, ANIM_FPS, AnimBehavior::Loop);
                    }
                    break;
                }
            }
        }
    }

    pub fn respawn(
        mut commands: Commands,
        player_entity: Entity,
        asset_server: Res<AssetServer>,
        texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
        crosshair: Single<&Transform, (With<Crosshair>, (Without<Camera2d>, Without<MainWorldCamera>))>,
    ) {
        commands.entity(player_entity).despawn();
        Player::spawn_player(commands, asset_server, texture_atlas_layouts, crosshair);
    }

    pub fn spawn_player(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
        crosshair: Single<&Transform, (With<Crosshair>, (Without<Camera2d>, Without<MainWorldCamera>))>,
    ) {
        println!("spawning player...");
        let crosshair_position = crosshair.translation;

        let spritesheet: Handle<Image> =
            asset_server.load("textures/player/player_anims-sheet.png");
        let atlas_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::new(PLAYER_SIZE_X, PLAYER_SIZE_Y),
            SHEET_COLS,
            1,
            None,
            None,
        ));

        let mut ec = commands.spawn((
            Transform {
                translation: crosshair_position,
                scale: Vec3::new(PLAYER_SCALE as f32, PLAYER_SCALE as f32, 1.0),
                ..Default::default()
            },
            RigidBody::KinematicPositionBased,
            Collider::cuboid(
                (PLAYER_SIZE_X / 2 - PLAYER_HB_X_OFFSET / 2) as f32,
                (PLAYER_SIZE_Y / 2 - PLAYER_HB_Y_OFFSET / 4) as f32,
            ),
            KinematicCharacterController {
                up: Vec2::Y,
                translation: Some(Vec2::new(0.0, 0.0)),
                autostep: Some(CharacterAutostep {
                    max_height: CharacterLength::Relative(0.3),
                    min_width: CharacterLength::Relative(0.5),
                    include_dynamic_bodies: true,
                }),
                snap_to_ground: Some(CharacterLength::Relative(0.1)),
                ..default()
            },
            Player::new(),
        ));
        ec.with_child((
            Sprite {
                image: spritesheet,
                texture_atlas: Some(TextureAtlas {
                    layout: atlas_layout,
                    index: ANIM_IDLE_FIRST as usize,
                }),
                ..Default::default()
            },
            PlayerSpriteTag,
            RenderLayers::layer(PLAYER_PASS_LAYER),
            SpriteAnimation::new(ANIM_IDLE_FIRST, ANIM_IDLE_LAST, ANIM_FPS, AnimBehavior::Loop),
            Transform {
                translation: Vec3::new(0.0, PLAYER_HB_Y_OFFSET as f32 / 4.0, 1.0),
                ..Default::default()
            },
            Anchor::CENTER,
        ));
    }
}

impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}
