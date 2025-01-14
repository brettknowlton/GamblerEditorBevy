use super::*;

pub fn update_placeholder(
    mut ui: Query<(&mut Sprite, &mut PlaceholderObject)>,
    placeholder: ResMut<PlaceholderTile>
) {
    for (mut sprite, _) in ui.iter_mut() {
        //update the placeholder tile to match the current tile type of our placeholderTile resource
        //do this by updating the UVs of the sprite
        sprite.rect = Some(Rect {
            min: Vec2::new(
                (((placeholder.0.tile_type as usize) % SPRITESHEET_WIDTH) as f32) *
                    (TILE_SIZE as f32),
                (((placeholder.0.tile_type as usize) / SPRITESHEET_WIDTH) as f32) *
                    (TILE_SIZE as f32)
            ),
            max: Vec2::new(
                (((placeholder.0.tile_type as usize) % SPRITESHEET_WIDTH) as f32) *
                    (TILE_SIZE as f32) +
                    (TILE_SIZE as f32),
                (((placeholder.0.tile_type as usize) / SPRITESHEET_WIDTH) as f32) *
                    (TILE_SIZE as f32) +
                    (TILE_SIZE as f32)
            ),
        });

        //also move the placeholder tile to the current crosshair location
    }
}

pub fn show_placeholder(
    mut commands: Commands,
    spritesheet: Res<TilesheetHandle>,
    crosshairs: Query<(&Transform, &Crosshair)>
) {
    let c = crosshairs.single();
    let x_off = c.0.translation.x;
    let y_off = c.0.translation.y;

    let texpath = spritesheet.0.clone();
    //display the placeholder tile
    commands.spawn((
        Tile {
            tile_type: 0,
            coordinate: Coordinate(0, 0),
        },
        Sprite {
            image: texpath,
            rect: Some(Rect {
                min: Vec2::new(0.0, 0.0),
                max: Vec2::new(TILE_SIZE as f32, TILE_SIZE as f32),
            }),
            ..default()
        },
        Transform {
            translation: Vec3::new(x_off, y_off, 0.0),
            ..default()
        },
        UIItem {
            ..default()
        },
        TileModeUI,
        PlaceholderObject,
    ));
}
