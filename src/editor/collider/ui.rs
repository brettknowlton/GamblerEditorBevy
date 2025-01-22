use super::*;

// pub fn update_placeholder(
//     mut ui: Query<(&mut Sprite, &mut PlaceholderObjectTag)>,
//     placeholder: ResMut<PlaceholderHandle>
// ) {
//     for (mut sprite, _) in ui.iter_mut() {
//         //update the placeholder tile to match the current tile type of our placeholderTile resource
//         //do this by updating the UVs of the sprite
//         sprite.rect = Some(Rect {
//             min: Vec2::new(
//                 (((placeholder.0.internal_type as usize) % SPRITESHEET_WIDTH) as f32) *
//                     (TILE_SIZE as f32),
//                 (((placeholder.0.internal_type as usize) / SPRITESHEET_WIDTH) as f32) *
//                     (TILE_SIZE as f32)
//             ),
//             max: Vec2::new(
//                 (((placeholder.0.internal_type as usize) % SPRITESHEET_WIDTH) as f32) *
//                     (TILE_SIZE as f32) +
//                     (TILE_SIZE as f32),
//                 (((placeholder.0.internal_type as usize) / SPRITESHEET_WIDTH) as f32) *
//                     (TILE_SIZE as f32) +
//                     (TILE_SIZE as f32)
//             ),
//         });

//         //also move the placeholder tile to the current crosshair location
//     }
// }

pub fn show_collider_placeholder(
    mut commands: Commands,
    crosshairs: Query<(&Transform, &Crosshair)>,
    asset_server: Res<AssetServer>,
) {
    let c = crosshairs.single();
    let x_off = c.0.translation.x;
    let y_off = c.0.translation.y;

    let texpath = PathBuf::from("textures/tiles/collider_debug.png");
    let tex = asset_server.load(texpath);


    //display the placeholder tile
    commands.spawn((
        Collider {
            ..default()
        },

        Sprite {
            image: tex,
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
        
        ColliderModeUI,

        PlaceholderObjectTag,
    ));
}

pub fn create_collidermode_ui(mut commands: Commands, asset_server: Res<AssetServer>, crosshairs: Query<(&Transform, &Crosshair)>, _tilesheet_handle: Res<TextureHandles>) {
    
    //display the "tilemode" menu
    let texpath = PathBuf::from("textures/menus/menu1.png");
    let tex1 = asset_server.load(texpath);

    //offsets to make UI appear in the top left corner of the screen while still being anchored to the crosshair location
    let c = crosshairs.single();

    let ui_x_off = -DEFAULT_WINDOW_WIDTH / 2.0 + c.0.translation.x;
    let ui_y_off = -DEFAULT_WINDOW_HEIGHT / 2.0 + c.0.translation.y;

    let ui_x = DEFAULT_WINDOW_WIDTH / 6.0;
    let ui_y = DEFAULT_WINDOW_HEIGHT;
    let ui_border = 4.0;

    //spawn collidermode UI
    commands.spawn((
        Sprite {
            image: tex1,
            anchor: Anchor::BottomLeft,
            custom_size: Some(Vec2::new(ui_x, ui_y)),
            image_mode: bevy::sprite::SpriteImageMode::Sliced(TextureSlicer {
                border: BorderRect {
                    bottom: ui_border,
                    left: ui_border,
                    right: ui_border,
                    top: ui_border,
                },
                sides_scale_mode: bevy::sprite::SliceScaleMode::Stretch,
                ..default()
            }),
            ..default()
        },
        Transform {
            translation: Vec3::new(ui_x_off, ui_y_off, UI_Z_LAYER - 0.1),
            ..default()
        },
        UIItem {
            ..default()
        },
        ColliderModeUI,
    ));

    //spawn the mode title at the top
    commands.spawn((
        ColliderModeUI,
        Text {
            0: "Collider Mode".to_string(),

            ..default()
        },
        Node{
            position_type: PositionType::Absolute,
            top: Val::Px(3.0),
            left: Val::Px(3.0),
            ..default()
        },
        UIItem::default(),
    ));
}