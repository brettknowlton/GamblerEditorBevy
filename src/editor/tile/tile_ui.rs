use crate::ui::PlaceholderObjectTag;

use super::*;

pub fn spawn_tile_placeholder(
    mut commands: Commands,
    spritesheet: Res<PlaceholderHandle>,
    crosshairs: Query<(&Transform, &Crosshair)>,
) {
    let Ok((t, _)) = crosshairs.single() else {
        return;
    };

    let x_off = t.translation.x;
    let y_off = t.translation.y;

    let texpath = spritesheet.0.clone();
    //spawn the placeholder tile
    commands.spawn((
        Tile {},
        Sprite {
            image: texpath,
            rect: Some(Rect {
                min: Vec2::new(0.0, 0.0),
                max: Vec2::new(TILE_SIZE as f32, TILE_SIZE as f32),
            }),
            color: Color::srgba(1.0, 1.0, 1.0, 0.5),
            ..default()
        },
        Transform {
            translation: Vec3::new(x_off, y_off, 0.0),
            ..default()
        },
        CameraLockedUI { ..default() },
        TileModeUI, // give it tilemodeUI so it will just be destroyed when we exit tilemode
        PlaceholderObjectTag,
    ));
}

#[derive(Component)]
pub struct TileSelector(pub Rect);
impl Default for TileSelector {
    fn default() -> Self {
        Self(Rect::new(0.0, 0.0, 1.0, 1.0))
    }
}
