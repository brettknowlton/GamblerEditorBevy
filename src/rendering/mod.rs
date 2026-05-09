use bevy::{
    camera::{
        visibility::RenderLayers,
        ClearColorConfig, RenderTarget,
    },
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderType, TextureFormat},
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d, Material2dPlugin},
};

pub const TILE_PASS_LAYER: usize = 1;
pub const PLAYER_PASS_LAYER: usize = 2;
pub const COMPOSITE_LAYER: usize = 3;

#[derive(Component)]
pub struct MainWorldCamera;

#[derive(Component)]
struct TilePassCamera;

#[derive(Component)]
struct PlayerPassCamera;

#[derive(Component)]
struct CompositeCamera;

#[derive(Component)]
struct TileCompositeQuad;

#[derive(Component)]
struct PlayerCompositeQuad;

#[derive(Clone, Copy, Debug)]
pub struct PixelEffectParams {
    pub pixel_size: f32,
    pub color_levels: f32,
    pub dither_strength: f32,
    pub scanline_strength: f32,
    pub palette_enabled: f32,
}

impl Default for PixelEffectParams {
    fn default() -> Self {
        Self {
            pixel_size: 2.0,
            color_levels: 8.0,
            dither_strength: 0.08,
            scanline_strength: 0.12,
            palette_enabled: 0.0,
        }
    }
}

#[derive(Resource, Clone, Debug)]
pub struct PixelArtSettings {
    pub tile: PixelEffectParams,
    pub player: PixelEffectParams,
    pub palette_texture: Option<Handle<Image>>,
}

impl Default for PixelArtSettings {
    fn default() -> Self {
        Self {
            tile: PixelEffectParams {
                pixel_size: 4.0,
                color_levels: 8.0,
                dither_strength: 0.08,
                scanline_strength: 0.1,
                palette_enabled: 0.0,
            },
            player: PixelEffectParams {
                pixel_size: 3.0,
                color_levels: 10.0,
                dither_strength: 0.05,
                scanline_strength: 0.06,
                palette_enabled: 0.0,
            },
            palette_texture: None,
        }
    }
}

#[derive(Clone, Copy, Debug, ShaderType)]
pub struct PixelEffectUniform {
    pub pixel_size: f32,
    pub color_levels: f32,
    pub dither_strength: f32,
    pub scanline_strength: f32,
    pub palette_enabled: f32,
    pub _pad_a: f32,
    pub _pad_b: f32,
    pub _pad_c: f32,
}

impl From<PixelEffectParams> for PixelEffectUniform {
    fn from(value: PixelEffectParams) -> Self {
        Self {
            pixel_size: value.pixel_size.max(1.0),
            color_levels: value.color_levels.max(2.0),
            dither_strength: value.dither_strength.clamp(0.0, 1.0),
            scanline_strength: value.scanline_strength.clamp(0.0, 1.0),
            palette_enabled: value.palette_enabled.clamp(0.0, 1.0),
            _pad_a: 0.0,
            _pad_b: 0.0,
            _pad_c: 0.0,
        }
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct PixelArtMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub source_texture: Handle<Image>,
    #[uniform(2)]
    pub effect: PixelEffectUniform,
    #[texture(3)]
    #[sampler(4)]
    pub palette_texture: Option<Handle<Image>>,
}

impl Material2d for PixelArtMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/pixel_art_material.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

#[derive(Resource)]
struct PixelPipelineState {
    tile_target: Handle<Image>,
    player_target: Handle<Image>,
    tile_material: Handle<PixelArtMaterial>,
    player_material: Handle<PixelArtMaterial>,
    tile_camera: Entity,
    player_camera: Entity,
    tile_quad: Entity,
    player_quad: Entity,
    logical_size: Vec2,
}

pub struct PixelArtRenderPlugin;

impl Plugin for PixelArtRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<PixelArtMaterial>::default())
            .init_resource::<PixelArtSettings>()
            .add_systems(Update, setup_pixel_pipeline)
            .add_systems(Update, sync_pass_camera_transforms)
            .add_systems(Update, sync_material_settings)
            .add_systems(Update, debug_pixel_controls)
            .add_systems(Update, handle_resize);
    }
}

fn debug_pixel_controls(
    input: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<PixelArtSettings>,
) {
    if input.just_pressed(KeyCode::PageUp) {
        settings.tile.pixel_size = (settings.tile.pixel_size + 1.0).min(24.0);
    }
    if input.just_pressed(KeyCode::PageDown) {
        settings.tile.pixel_size = (settings.tile.pixel_size - 1.0).max(1.0);
    }

    if input.just_pressed(KeyCode::BracketRight) {
        settings.player.pixel_size = (settings.player.pixel_size + 1.0).min(24.0);
    }
    if input.just_pressed(KeyCode::BracketLeft) {
        settings.player.pixel_size = (settings.player.pixel_size - 1.0).max(1.0);
    }

    if input.just_pressed(KeyCode::Equal) {
        settings.tile.color_levels = (settings.tile.color_levels + 1.0).min(32.0);
        settings.player.color_levels = (settings.player.color_levels + 1.0).min(32.0);
    }
    if input.just_pressed(KeyCode::Minus) {
        settings.tile.color_levels = (settings.tile.color_levels - 1.0).max(2.0);
        settings.player.color_levels = (settings.player.color_levels - 1.0).max(2.0);
    }
}

fn setup_pixel_pipeline(
    mut commands: Commands,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    main_camera_query: Query<&Transform, With<MainWorldCamera>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<PixelArtMaterial>>,
    settings: Res<PixelArtSettings>,
    existing: Option<Res<PixelPipelineState>>,
) {
    if existing.is_some() {
        return;
    }

    let Ok(main_camera_t) = main_camera_query.single() else {
        return;
    };

    let Ok(window) = windows.single() else {
        return;
    };

    let logical_size = Vec2::new(window.width().max(1.0), window.height().max(1.0));
    let physical_width = window.resolution.physical_width().max(1);
    let physical_height = window.resolution.physical_height().max(1);

    let tile_target = images.add(Image::new_target_texture(
        physical_width,
        physical_height,
        TextureFormat::Rgba8Unorm,
        Some(TextureFormat::Rgba8UnormSrgb),
    ));
    let player_target = images.add(Image::new_target_texture(
        physical_width,
        physical_height,
        TextureFormat::Rgba8Unorm,
        Some(TextureFormat::Rgba8UnormSrgb),
    ));

    let tile_material = materials.add(PixelArtMaterial {
        source_texture: tile_target.clone(),
        effect: settings.tile.into(),
        palette_texture: settings.palette_texture.clone(),
    });
    let player_material = materials.add(PixelArtMaterial {
        source_texture: player_target.clone(),
        effect: settings.player.into(),
        palette_texture: settings.palette_texture.clone(),
    });

    let fullscreen_mesh = meshes.add(Rectangle::default());

    let tile_quad = commands
        .spawn((
            Mesh2d(fullscreen_mesh.clone()),
            MeshMaterial2d(tile_material.clone()),
            Transform::from_scale(Vec3::new(logical_size.x, logical_size.y, 1.0)),
            RenderLayers::layer(COMPOSITE_LAYER),
            TileCompositeQuad,
        ))
        .id();

    let player_quad = commands
        .spawn((
            Mesh2d(fullscreen_mesh),
            MeshMaterial2d(player_material.clone()),
            Transform::from_translation(Vec3::new(0.0, 0.0, 1.0))
                .with_scale(Vec3::new(logical_size.x, logical_size.y, 1.0)),
            RenderLayers::layer(COMPOSITE_LAYER),
            PlayerCompositeQuad,
        ))
        .id();

    let tile_camera = commands
        .spawn((
            Camera2d,
            Camera {
                order: -100,
                clear_color: ClearColorConfig::Custom(Color::NONE),
                ..default()
            },
            RenderTarget::Image(tile_target.clone().into()),
            Transform::from_translation(main_camera_t.translation),
            RenderLayers::layer(TILE_PASS_LAYER),
            TilePassCamera,
        ))
        .id();

    let player_camera = commands
        .spawn((
            Camera2d,
            Camera {
                order: -90,
                clear_color: ClearColorConfig::Custom(Color::NONE),
                ..default()
            },
            RenderTarget::Image(player_target.clone().into()),
            Transform::from_translation(main_camera_t.translation),
            RenderLayers::layer(PLAYER_PASS_LAYER),
            PlayerPassCamera,
        ))
        .id();

    commands.spawn((
        Camera2d,
        Camera {
            order: 0,
            ..default()
        },
        RenderLayers::layer(COMPOSITE_LAYER),
        CompositeCamera,
    ));

    commands.insert_resource(PixelPipelineState {
        tile_target,
        player_target,
        tile_material,
        player_material,
        tile_camera,
        player_camera,
        tile_quad,
        player_quad,
        logical_size,
    });
}

fn sync_pass_camera_transforms(
    main_camera_query: Query<&Transform, (With<MainWorldCamera>, Without<TilePassCamera>)>,
    mut transforms: Query<&mut Transform, Without<MainWorldCamera>>,
    state: Option<Res<PixelPipelineState>>,
) {
    let Some(state) = state else {
        return;
    };

    let Ok(main_camera_t) = main_camera_query.single() else {
        return;
    };

    if let Ok(mut t) = transforms.get_mut(state.tile_camera) {
        t.translation.x = main_camera_t.translation.x;
        t.translation.y = main_camera_t.translation.y;
        t.translation.z = main_camera_t.translation.z;
    }

    if let Ok(mut t) = transforms.get_mut(state.player_camera) {
        t.translation.x = main_camera_t.translation.x;
        t.translation.y = main_camera_t.translation.y;
        t.translation.z = main_camera_t.translation.z;
    }
}

fn sync_material_settings(
    state: Option<Res<PixelPipelineState>>,
    settings: Res<PixelArtSettings>,
    mut materials: ResMut<Assets<PixelArtMaterial>>,
) {
    let Some(state) = state else {
        return;
    };

    if let Some(tile) = materials.get_mut(&state.tile_material) {
        tile.effect = settings.tile.into();
        tile.palette_texture = settings.palette_texture.clone();
    }

    if let Some(player) = materials.get_mut(&state.player_material) {
        player.effect = settings.player.into();
        player.palette_texture = settings.palette_texture.clone();
    }
}

fn handle_resize(
    mut resize_events: MessageReader<bevy::window::WindowResized>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    state: Option<ResMut<PixelPipelineState>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<PixelArtMaterial>>,
    mut render_targets: Query<&mut RenderTarget>,
    mut transforms: Query<&mut Transform>,
) {
    let mut saw_resize = false;
    for _ in resize_events.read() {
        saw_resize = true;
    }
    if !saw_resize {
        return;
    }

    let Some(mut state) = state else {
        return;
    };

    let Ok(window) = windows.single() else {
        return;
    };

    let new_logical = Vec2::new(window.width().max(1.0), window.height().max(1.0));
    if (new_logical - state.logical_size).length_squared() < f32::EPSILON {
        return;
    }

    let physical_width = window.resolution.physical_width().max(1);
    let physical_height = window.resolution.physical_height().max(1);

    let tile_target = images.add(Image::new_target_texture(
        physical_width,
        physical_height,
        TextureFormat::Rgba8Unorm,
        Some(TextureFormat::Rgba8UnormSrgb),
    ));
    let player_target = images.add(Image::new_target_texture(
        physical_width,
        physical_height,
        TextureFormat::Rgba8Unorm,
        Some(TextureFormat::Rgba8UnormSrgb),
    ));

    state.tile_target = tile_target.clone();
    state.player_target = player_target.clone();
    state.logical_size = new_logical;

    if let Ok(mut rt) = render_targets.get_mut(state.tile_camera) {
        *rt = RenderTarget::Image(tile_target.clone().into());
    }
    if let Ok(mut rt) = render_targets.get_mut(state.player_camera) {
        *rt = RenderTarget::Image(player_target.clone().into());
    }

    if let Some(tile) = materials.get_mut(&state.tile_material) {
        tile.source_texture = tile_target;
    }
    if let Some(player) = materials.get_mut(&state.player_material) {
        player.source_texture = player_target;
    }

    if let Ok(mut t) = transforms.get_mut(state.tile_quad) {
        t.scale = Vec3::new(new_logical.x, new_logical.y, 1.0);
    }
    if let Ok(mut t) = transforms.get_mut(state.player_quad) {
        t.scale = Vec3::new(new_logical.x, new_logical.y, 1.0);
    }
}
