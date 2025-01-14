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


pub fn create_tilemode_ui(mut commands: Commands, asset_server: Res<AssetServer>, crosshairs: Query<(&Transform, &Crosshair)>, tilesheet_handle: Res<TilesheetHandle>) {
    
    //display the "tilemode" menu
    let texpath = PathBuf::from("textures/menus/menu1.png");
    let tex1 = asset_server.load(texpath);

    //offsets to make UI appear in the top left corner of the screen while still being anchored to the crosshair location
    let c = crosshairs.single();

    let ui_x_off = -WINDOW_WIDTH / 2.0 + c.0.translation.x;
    let ui_y_off = -WINDOW_HEIGHT / 2.0 + c.0.translation.y;

    let ui_x = WINDOW_WIDTH / 6.0;
    let ui_y = WINDOW_HEIGHT;

    //spawn tilemodeUI
    commands.spawn((
        Sprite {
            image: tex1,
            anchor: Anchor::BottomLeft,
            custom_size: Some(Vec2::new(ui_x, ui_y)),
            image_mode: bevy::sprite::SpriteImageMode::Sliced(TextureSlicer {
                border: BorderRect {
                    bottom: 4.0,
                    left: 4.0,
                    right: 4.0,
                    top: 4.0,
                },
                sides_scale_mode: bevy::sprite::SliceScaleMode::Stretch,
                ..default()
            }),
            ..default()
        },
        Transform {
            translation: Vec3::new(ui_x_off, ui_y_off, 0.0),
            ..default()
        },
        UIItem {
            ..default()
        },
        TileModeUI,
    ));


    let tex = tilesheet_handle.0.clone();
    commands.spawn(
        (
            UIItem {
                ..default()
            },
            Sprite {
                image: tex,
                anchor: Anchor::BottomLeft,
                custom_size: Some(Vec2::new(WINDOW_WIDTH / 6.0, WINDOW_HEIGHT)),
                image_mode: bevy::sprite::SpriteImageMode::Sliced(TextureSlicer {
                    border: BorderRect {
                        bottom: 4.0,
                        left: 4.0,
                        right: 4.0,
                        top: 4.0,
                    },
                    sides_scale_mode: bevy::sprite::SliceScaleMode::Stretch,
                    ..default()
                }),
                ..default()
            },
            Transform {
                translation: Vec3::new(ui_x_off + (ui_x / 2.), ui_y_off, 0.0),
                ..default()
            },
        )
    );
}