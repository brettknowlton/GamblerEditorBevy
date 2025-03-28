use crate::collider::ColliderObject;
use avian2d::prelude::*;
use super::*;
use crate::utilities::physics::*;

pub fn player_controls(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Player, &mut Velocity, &mut RigidBody)>,
) {
    for (mut player, mut velocity, rb) in query.iter_mut() {
        if *rb == RigidBody::Dynamic {
            // Jump
            if keyboard_input.pressed(KeyCode::KeyW) && player.on_ground {
                if player.air_timer < PLAYER_JUMP_GRACE_PERIOD {
                    velocity.y += PLAYER_JUMP_FORCE as f32;
                    player.on_ground = false;
                }
            }

            // Move Down (optional, depending on your game logic)
            if keyboard_input.pressed(KeyCode::KeyS) {
                player.on_ground = false;
            }

            // Move Left
            if keyboard_input.pressed(KeyCode::KeyA) {
                velocity.x -= PLAYER_WALK_SPEED as f32;
            }

            // Move Right
            if keyboard_input.pressed(KeyCode::KeyD) {
                velocity.x += PLAYER_WALK_SPEED as f32;
            }
        }
    }
}


pub fn player_physics(
    time: Res<Time>,
    mut players: Query<(Entity, &mut Player, &mut Transform, &mut Collider)>,
    mut colliders: Query<(Entity, &mut Collider), With<ColliderObject>>,
) {
    for (pe, mut player, t, c) in players.iter_mut() {
        let mut is_on_ground = true;

        player.on_ground = is_on_ground;

        if player.on_ground {
            player.air_timer = 0.0;
        } else {
            player.air_timer += time.delta_secs();
        }
    }
}


pub fn move_player_to_cursor(cursor_transform: Transform, player_transform: &mut Transform) {
    player_transform.translation = cursor_transform.translation;
    player_transform.translation.z = 1.0;
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
    pub on_ground: bool,
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
}

impl Default for Player {
    fn default() -> Self {
        Self {
            state: PlayerState::Idle,
            facing: Direction::Down,
            current_animation: AnimationDef {
                frame_size: Vec2::new(36.0, 45.0),
                layout: Vec2::new(1., 1.),
                frame_count: 1,
                frame_duration: 1.,
                current_frame: 0,
                frame_timer: 0.0,
            },
            air_timer: 0.0,
            on_ground: false,
        }
    }
}

pub fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    crosshairs: Query<&Transform, With<Crosshair>>,
) {
    println!("spawning player...");
    let path = PathBuf::from("textures/player/PlayerHD.png");
    let player_sprite = asset_server.load(path);
    let mut ec = commands.spawn(
        ((
            Transform {
                translation: Vec3::new(0.0, 100.0, 0.0),
                scale: Vec3::new(PLAYER_SCALE as f32, PLAYER_SCALE as f32, 1.),
                ..Default::default()
            },
            RigidBody::Dynamic,
            Collider::rectangle(
                ((PLAYER_SIZE_X) - (PLAYER_HB_X_OFFSET )) as f32,
                ((PLAYER_SIZE_Y) - (PLAYER_HB_Y_OFFSET / 2)) as f32,
            ),
            LockedAxes::ROTATION_LOCKED,
            Velocity::default(),
            Player {
                ..Default::default()
            },
        )),
    );
    ec.with_child((
        Sprite {
            image: player_sprite,
            anchor: bevy::sprite::Anchor::Center,
            ..Default::default()
        },
        Transform {
            translation: Vec3::new(0.0, 0.0 + (PLAYER_HB_Y_OFFSET as f32 / 4.), 1.),
            ..Default::default()
        },
    ));
}
