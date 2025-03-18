use super::*;

pub fn player_physics(
    time: Res<Time>,
    mut query: Query<(&mut Player, &mut Transform)>,
){
    for (mut player, mut transform) in query.iter_mut() {




        //apply gravity
        player.velocity.y -= GRAVITY * time.delta_secs();

        //apply friction
        player.velocity.x *= 1.0 - FRICTION * time.delta_secs();
        player.velocity.y *= 1.0 - FRICTION * time.delta_secs();

        transform.translation += Vec3::new(player.velocity.x, player.velocity.y, 0.0);//appending player velocity to player position

        player.animate(time.delta_secs());
    }


}


#[derive(Component, Debug)]
pub struct Player {
    pub state: PlayerState,
    pub current_animation: AnimationDef,
    pub facing: Direction,
    pub velocity: Vec2,


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
            velocity: Vec2::new(0.0, 0.0),
            current_animation: AnimationDef {
                frame_size: Vec2::new(36.0, 45.0),
                layout: Vec2::new(1., 1.),
                frame_count: 1,
                frame_duration: 1.,
                current_frame: 0,
                frame_timer: 0.0,
            },
        }
    }
}

pub fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    println!("spawning player...");
    let path = PathBuf::from("textures\\player\\PlayerHD.png");
    let player_sprite = asset_server.load(path);
    commands.spawn((
        Player {
            ..Default::default()
        },
        Sprite {
            image: player_sprite,
            custom_size: Some(Vec2::new(SCALED_PLAYER_WIDTH as f32, SCALED_PLAYER_HEIGHT  as f32)),
            ..Default::default()
        },
        Transform {
            translation: Vec3::new(0.0, 0.0, 1.),
            ..Default::default()
        },
    ));
}
