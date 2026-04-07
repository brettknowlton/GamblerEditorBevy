use super::*;

pub fn spawn_collider_placeholder(
    mut commands: Commands,
    crosshairs: Query<(&Transform, &Crosshair)>,
    asset_server: Res<AssetServer>,
) {
    let Ok((t, _)) = crosshairs.single() else {
        return;
    };
    let x_off = t.translation.x;
    let y_off = t.translation.y;

    let texpath = PathBuf::from("textures/tiles/collider_debug.png");
    let tex = asset_server.load(texpath);

    //display the placeholder tile
    commands.spawn((
        ColliderObject { ..default() },
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
        UIItem { ..default() },
        ColliderModeUI,
        PlaceholderObjectTag,
    ));
}

pub fn create_collidermode_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    crosshairs: Query<(&Transform, &Crosshair)>,
    _tilesheet_handle: Res<TextureHandles>,
) {
    //display the "tilemode" menu
    let texpath = PathBuf::from("textures/menus/menu1.png");
    let tex1 = asset_server.load(texpath);

    //offsets to make UI appear in the top left corner of the screen while still being anchored to the crosshair location
    let Ok((t, _)) = crosshairs.single() else {
        return;
    };

    let ui_x_off = -(DEFAULT_WINDOW_WIDTH as f32) / 2.0 + t.translation.x;
    let ui_y_off = -(DEFAULT_WINDOW_HEIGHT as f32) / 2.0 + t.translation.y;

    let ui_x = DEFAULT_WINDOW_WIDTH as f32 / 6.0;
    let ui_y = DEFAULT_WINDOW_HEIGHT as f32;
    let ui_border = 4.0;

    //spawn collidermode UI
    commands.spawn((
        Sprite {
            image: tex1,
            custom_size: Some(Vec2::new(ui_x, ui_y)),
            image_mode: bevy::sprite::SpriteImageMode::Sliced(TextureSlicer {
                border: BorderRect {
                    max_inset: Vec2::new(ui_border, ui_border),
                    ..default()
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
        UIItem { ..default() },
        ColliderModeUI,
        Anchor::CENTER,
    ));

    //spawn the mode title at the top
    commands.spawn((
        ColliderModeUI,
        Text {
            0: "Collider Mode".to_string(),

            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(3.0),
            left: Val::Px(3.0),
            ..default()
        },
        UIItem::default(),
    ));
}

/// A component that marks an entity as part
///
/// of the collider editing UI.
#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(UIItem)]
pub struct ColliderModeUI;
