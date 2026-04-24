use std::ops::DerefMut;

use crate::{direction::Direction, DEFAULT_GENERAL_SCALE_FACTOR, EPSILON, FRICTION, GRAVITY};

use bevy::sprite::Anchor;
use bevy_rapier2d::prelude::{Collider, RigidBody};

use super::*;
/// Width of the player source image in pixels
pub const PLAYER_SIZE_X: u32 = 72;

/// Height of the player source image in pixels
pub const PLAYER_SIZE_Y: u32 = 90;

/// Scale factor for the player
pub const PLAYER_SCALE: u32 = DEFAULT_GENERAL_SCALE_FACTOR; //by default player has normal scaling

/// Total pixel size the player width takes up IN GAME
pub const SCALED_PLAYER_WIDTH: u32 = PLAYER_SIZE_X * PLAYER_SCALE;
/// Total pixel size the player height takes up IN GAME
pub const SCALED_PLAYER_HEIGHT: u32 = PLAYER_SIZE_Y * PLAYER_SCALE;

/// Horizontal offset for the player's hitbox
pub const PLAYER_HB_X_OFFSET: u32 = SCALED_PLAYER_WIDTH / 3;

/// Vertical offset for the player's hitbox
pub const PLAYER_HB_Y_OFFSET: u32 = SCALED_PLAYER_HEIGHT / 3;

/// Force applied to the player when walking
pub const PLAYER_WALK_FORCE: u32 = 200;
/// Maximum walking speed for the player
pub const MAX_PLAYER_WALK_SPEED: u32 = 300;

/// Force applied to the player when jumping
pub const PLAYER_JUMP_FORCE: f32 = 550.;
/// How long a vertical jump force can be applied to a player
pub const PLAYER_JUMP_GRACE_PERIOD: f32 = 0.3;


pub fn player_controls(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Player, &KinematicCharacterControllerOutput)>,
    time: Res<Time>,
) {
    for (mut player, kpco) in query.iter_mut() {
        // Reset or increment the air timer based on ground state
        if kpco.grounded {
            player.air_timer = 0.05;
        } else {
            player.air_timer += time.delta_secs();
        }

        // Jump
        if keyboard_input.pressed(KeyCode::KeyW) {
            player.trying_jump = true;
            if player.air_timer < PLAYER_JUMP_GRACE_PERIOD {
                if keyboard_input.just_pressed(KeyCode::KeyW) && kpco.grounded {
                    player.velocity.y = PLAYER_JUMP_FORCE as f32;
                };
            }
        } else {
            player.trying_jump = false;
        }

        // Move Down (optional, depending on your game logic)
        if keyboard_input.pressed(KeyCode::KeyS) {
            continue;
        }

        // Move Left
        if keyboard_input.pressed(KeyCode::KeyA) {
            player.trying_walk_left = true;
            player.velocity.x -= PLAYER_WALK_FORCE as f32;
        } else {
            player.trying_walk_left = false;
        }

        // Move Right
        if keyboard_input.pressed(KeyCode::KeyD) {
            player.trying_walk_right = true;
            player.velocity.x += PLAYER_WALK_FORCE as f32;
        } else {
            player.trying_walk_right = false;
        }
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

    if kpco.grounded {
        //ON GROUND PHYSICS
        //apply a small amount of gravity to the player if on the ground
        //this is to prevent the player from getting stuck in the ground
        let g = Vec2::new(0.0, -(GRAVITY as f32) * 0.1);
        player.velocity += g * time.delta_secs();

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
        let g;
        if player.trying_jump && player.air_timer < PLAYER_JUMP_GRACE_PERIOD {
            //while trying to jump higher we suspend the effect of gravity somewhat
            g = Vec2::new(0.0, -(GRAVITY as f32) * 0.1);
        } else {
            g = Vec2::new(0.0, -(GRAVITY as f32));
        }

        //apply a small amount of friction to the player if in the air
        //clamp velocity.x to double max walk speed if on ground
        if player.velocity.x > 2. * MAX_PLAYER_WALK_SPEED as f32 {
            player.velocity.x = 2. * MAX_PLAYER_WALK_SPEED as f32;
        } else if player.velocity.x < -(2. * MAX_PLAYER_WALK_SPEED as f32) {
            player.velocity.x = -(2. * MAX_PLAYER_WALK_SPEED as f32);
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
}

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
pub struct AnimationDef {
    pub frame_size: Vec2,
    pub layout: Vec2,
    pub frame_count: u32,
    pub frame_duration: f32,
    pub current_frame: u32,
    pub frame_timer: f32,
}

#[derive(Component, Debug, Reflect)]
pub struct Player {
    pub state: PlayerState,
    pub current_animation: AnimationDef,
    pub facing: Direction,
    pub air_timer: f32,
    pub velocity: Vec2,

    pub trying_jump: bool,
    pub trying_walk_left: bool,
    pub trying_walk_right: bool,
}

impl Player {
    pub fn animate(&mut self, time: f32) {
        self.current_animation.frame_timer += 1.0 / time;
        if self.current_animation.frame_timer >= self.current_animation.frame_duration {
            self.current_animation.current_frame += 1;
            self.current_animation.frame_timer = 0.0;
        }
        if self.current_animation.current_frame >= self.current_animation.frame_count {
            self.current_animation.current_frame = 0;
        }
    }

    pub fn respawn(
        mut commands: Commands,
        player_entity: Entity,
        asset_server: Res<AssetServer>,
        crosshair: Single<&Transform, (With<Crosshair>, Without<Camera2d>)>,
    ) {
        commands.entity(player_entity).despawn();
        Player::spawn_player(commands, asset_server, crosshair);
    }

    pub fn spawn_player(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        crosshair: Single<&Transform, (With<Crosshair>, Without<Camera2d>)>,
    ) {
        println!("spawning player...");
        let path = PathBuf::from("textures/player/PlayerHD.png");
        let player_sprite = asset_server.load(path);
        let crosshair_position = crosshair.clone().translation;

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
                ..default()
            },
            Player {
                ..Default::default()
            },
        ));
        ec.with_child((
            Sprite {
                image: player_sprite,
                ..Default::default()
            },
            Transform {
                translation: Vec3::new(0.0, 0.0 + (PLAYER_HB_Y_OFFSET as f32) / 4.0, 1.0),
                ..Default::default()
            },
            Anchor::CENTER,
        ));

        //update the kindematic character controller to use the collider
    }
}

impl Default for Player {
    fn default() -> Self {
        Self {
            state: PlayerState::Idle,
            facing: Direction::Down,
            current_animation: AnimationDef {
                frame_size: Vec2::new(36.0, 45.0),
                layout: Vec2::new(1.0, 1.0),
                frame_count: 1,
                frame_duration: 1.0,
                current_frame: 0,
                frame_timer: 0.0,
            },
            air_timer: 0.0,
            velocity: Vec2::new(0.0, 0.0),
            trying_jump: false,
            trying_walk_left: false,
            trying_walk_right: false,
        }
    }
}
