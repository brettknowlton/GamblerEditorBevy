use super::*;


pub fn player_controls(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Player, &mut Transform)>,
) {
    for (mut player, _) in query.iter_mut() {
        if keyboard_input.just_pressed(KeyCode::KeyW) {
            
            if player.on_ground == true {
                player.velocity.y += PLAYER_JUMP_FORCE as f32;
                player.facing = Direction::Up;
                player.on_ground = false;
            }
            
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            player.on_ground = false;
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            player.velocity.x -= PLAYER_WALK_SPEED as f32;
            player.facing = Direction::Left;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            player.velocity.x += PLAYER_WALK_SPEED as f32;
            player.facing = Direction::Right;
        }
    }
}

pub fn player_physics(
    time: Res<Time>,
    mut query: Query<(&mut Player, &mut Transform)>,
){
    for (mut player, mut transform) in query.iter_mut() {

        //apply gravity
        player.velocity.y += player.acceleration.y - GRAVITY * time.delta_secs();
        player.acceleration.y = 0.0;

        player.velocity.x += player.acceleration.x;
        player.acceleration.x = 0.0;

        //apply friction
        player.velocity.x *= 1.0 - FRICTION * time.delta_secs();
        player.velocity.y *= 1.0 - FRICTION * time.delta_secs();

        //clamp velocity
        player.velocity.x = player.velocity.x.clamp(-(MAX_PLAYER_WALK_SPEED as f32), MAX_PLAYER_WALK_SPEED as f32);
        player.velocity.y = player.velocity.y.clamp(-(MAX_PLAYER_WALK_SPEED as f32), MAX_PLAYER_WALK_SPEED as f32);

        transform.translation += Vec3::new(player.velocity.x, player.velocity.y, 0.0);//appending player velocity to player position

        player.animate(time.delta_secs());
    }
}

pub fn move_player_to_cursor(cursor_transform: Transform, player_transform: &mut Transform) {
    //since everything is anchored bottom left, we will need to adjust the player's position to be centered on the cursor
    player_transform.translation = cursor_transform.translation + Vec3::new(-(SCALED_PLAYER_WIDTH as f32) / 2., -(SCALED_PLAYER_HEIGHT as f32) / 2., 0.0);
    player_transform.translation.z = 1.0;
}

#[derive(Component, Debug)]
pub struct Player {
    pub state: PlayerState,
    pub current_animation: AnimationDef,
    pub facing: Direction,
    pub velocity: Vec2,
    pub acceleration: Vec2,

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
            velocity: Vec2::new(0.0, -1.0),
            acceleration: Vec2::new(0.0, 0.0),
            current_animation: AnimationDef {
                frame_size: Vec2::new(36.0, 45.0),
                layout: Vec2::new(1., 1.),
                frame_count: 1,
                frame_duration: 1.,
                current_frame: 0,
                frame_timer: 0.0,
            },
            on_ground: false,
        }
    }
}

pub fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    println!("spawning player...");
    let path = PathBuf::from("textures/player/PlayerHD.png");
    let player_sprite = asset_server.load(path);
    commands.spawn((
        Player {
            ..Default::default()
        },
        Sprite {
            image: player_sprite,
            anchor: bevy::sprite::Anchor::BottomLeft,
            ..Default::default()
        },
        Transform {
            translation: Vec3::new(0.0, 200.0, 1.),
            scale: Vec3::new(TILE_SCALE as f32, TILE_SCALE as f32, 1.),
            ..Default::default()
        },
    ));
}

pub fn do_player_collision(mut players: Query<(Entity, &mut Player, &mut Transform)>, colliders: Query<(Entity, &Collider, &Transform), Without<Player>>) {
    for (_, mut player, mut player_transform) in players.iter_mut() {


        let player_rect = Rect::new(
            player_transform.translation.x + PLAYER_HB_X_OFFSET as f32,
            player_transform.translation.y,
            player_transform.translation.x + SCALED_PLAYER_WIDTH as f32 - PLAYER_HB_X_OFFSET as f32,
            player_transform.translation.y + (SCALED_PLAYER_HEIGHT - PLAYER_HB_Y_OFFSET) as f32,
        );
        let mut collisions = Vec::new();
        for (_, _, collider_transform) in colliders.iter() {
            let collider_rect = Rect::new(
                collider_transform.translation.x,
                collider_transform.translation.y,
                collider_transform.translation.x + SCALED_TILE_WIDTH as f32,
                collider_transform.translation.y + SCALED_TILE_HEIGHT as f32,
            );
            let intersect = collider_rect.intersect(player_rect);
            if ! intersect.is_empty() {
                //collision detected
                collisions.push((intersect, collider_rect));
            }
        }

        collisions.iter().for_each(|(intersection, collider_rect)| {
            println!("Handling collision between player: {:?} and collider: {:?}", player_rect, collider_rect);
            //handle collision
            if intersection.width() > intersection.height() {
                //player is colliding with the top or bottom of the collider
                if player.velocity.y > 0.0 {
                    //player is colliding with the bottom of the collider (jumping case)
                    player_transform.translation.y = collider_rect.min.y - player_rect.height();
                    player.velocity.y = 0.0;
                    player.acceleration.y = 0.0;
                }
                if player.velocity.y < 0.0 {
                    //player is colliding with the top of the collider (falling case)
                    player_transform.translation.y = collider_rect.max.y;
                    player.velocity.y = 0.0;
                    player.on_ground = true;
                }
            }
            
            if intersection.width() < intersection.height() {
                //player is colliding with the left or right of the collider 
                if player.velocity.x > 0.0 {
                    //player is colliding with the left of the collider (player moving right)
                    player_transform.translation.x = collider_rect.min.x - SCALED_PLAYER_WIDTH as f32 + PLAYER_HB_X_OFFSET as f32;
                    player.velocity.x = 0.0;
                }
                if player.velocity.x < 0.0 {
                    //player is colliding with the right of the collider (player moving left)
                    player_transform.translation.x = collider_rect.max.x - PLAYER_HB_X_OFFSET as f32;
                    player.velocity.x = 0.0;
                }
            }
        }

        );
    }
}
