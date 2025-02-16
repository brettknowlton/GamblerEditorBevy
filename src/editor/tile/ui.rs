use bevy::asset;

use super::*;

pub fn spawn_tile_placeholder(
    mut commands: Commands,
    spritesheet: Res<PlaceholderHandle>,
    crosshairs: Query<(&Transform, &Crosshair)>
) {
    let c = crosshairs.single();
    let x_off = c.0.translation.x;
    let y_off = c.0.translation.y;

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
        UIItem {
            ..default()
        },
        TileModeUI, // give it tilemodeUI so it will just be destroyed when we exit tilemode
        PlaceholderObjectTag,
    ));
}


pub fn create_tilemode_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    crosshairs: Query<(&Transform, &Crosshair)>,
    tilesheet_handle: Res<PlaceholderHandle>,
    textures: Res<TextureHandles>
) {
    //offsets to make UI appear in the top left corner of the screen while still being anchored to the crosshair location
    let c = crosshairs.single();

    let ui_x_off = -DEFAULT_WINDOW_WIDTH / 2.0 + c.0.translation.x;
    let ui_y_off = -DEFAULT_WINDOW_HEIGHT / 2.0 + c.0.translation.y;

    let ui_panel_width = DEFAULT_WINDOW_WIDTH / 6.0;
    let ui_panel_height = DEFAULT_WINDOW_HEIGHT;
    let ui_border = 4.0;

    //display the "tilemode" menu
    let texpath = PathBuf::from("textures/menus/menu1.png");
    let tex1 = asset_server.load(texpath);
    commands.spawn((
        TileModeUI,
        Sprite {
            image: tex1,
            anchor: Anchor::BottomLeft,
            custom_size: Some(Vec2::new(ui_panel_width, ui_panel_height)),
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
            translation: Vec3::new(ui_x_off, ui_y_off, 0.0),
            ..default()
        },
        UIItem {
            ..default()
        },
    ));

    //spawn the display tilesheet, align with the top right of the UI, inset by the border of the UI
    let tex = textures.0.get(&'t').unwrap().clone();
    commands.spawn((
        TileModeUI,
        UIItem {
            ..default()
        },
        Sprite {
            image: tex,
            custom_size: Some(
                Vec2::new(
                    ui_panel_width- UI_BORDER_REAL,
                    (MAX_SPRITESHEET_ITEMS / SPRITESHEET_WIDTH * (TILE_SCALE as u64)) as f32 * UI_SCALE,
                    
                )
            ),
            anchor: Anchor::TopLeft,
            ..default()
        },
        Transform {
            translation: Vec3::new(ui_x_off + UI_BORDER_PX, (ui_panel_height / 2.) - DEFAULT_TEXT_HEIGHT, -0.01),
            scale: Vec3::new(1.0, 1.0, 0.0),
            ..default()
        },
    ));

    //spawn the tilemode text at the top left of the UI
    commands.spawn((
        TileModeUI,
        Text {
            0: "Tile Mode".to_string(),

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

    let tex = textures.0.get(&'r').unwrap().clone();
    //create a tile selection component that will be used to select tiles from the tilesheet
    commands.spawn((
        TileModeUI,
        TileSelector {
            0: Rect {
                min: Vec2::new(0.0, 0.0),
                max: Vec2::new(TILE_SIZE as f32, TILE_SIZE as f32),
            },
        },
        Sprite {
            image: tex,
            rect: Some(Rect {
                min: Vec2::new(0.0, 0.0),
                max: Vec2::new(TILE_SIZE as f32, TILE_SIZE as f32),
            }),
            ..default()
        },
    ));
}

#[derive(Component)]
pub struct TileSelector(pub Rect);
impl Default for TileSelector {
    fn default() -> Self {
        Self(Rect::new(0.0, 0.0, 1.0, 1.0))
    }
}
