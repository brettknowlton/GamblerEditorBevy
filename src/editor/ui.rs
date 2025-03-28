use bevy::prelude::*;

use super::*;

macro_rules! send_message {
    ($key:expr, $messages:expr, $message:expr) => {
        $messages.messages.push(($key, $message.to_string()));
    };
    (_) => {
        $messages.messages.push((None, " ".to_string()));
    };
}

#[derive(Component)]
pub struct DisplayMessage;
pub fn send_messages(
    mut queued_messages: ResMut<EditorBottomBarQueuedMessages>,
    mut display_message: Query<(&DisplayMessage, &mut Text)>
) {
    //push any messages into the in-game console and leave the last one in our BottomBarMessage for display
    let item = queued_messages.messages.pop();
    {
        match item {
            Some((k, m)) => {
                let k = k.unwrap_or('i');
                println!("{}:> {}", k, m);
            }
            None => {}
        }
    }

    if let Some((_, message)) = queued_messages.messages.first() {
        let (_, mut t) = display_message.single_mut();
        **t = format!("{message}",);
    }
}

///Systems can be added for this comonent to keep all UI items moving at the same speed, and therefore always relatively positioned to eachother.
/// Useful for menus, or any thing that you want to keep moving based on the camera's location. This does not prevent movement of the object by other systems,
/// we are just also using this to TAG all UI items so we can easily find them in queries (typically for movement so far)
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[require(Transform)]
pub struct UIItem {
    pub vel_x: f32,
    pub vel_y: f32,
}

pub fn spawn_general_editor_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    //spawn a bar on the bottom using our default UI menu sprite

    let x_size = DEFAULT_WINDOW_WIDTH;
    let y_size = 30.0;

    let ui_texpath = PathBuf::from("textures/menus/menu1.png");
    let ui_tex = asset_server.load(ui_texpath);
    commands.spawn((
        UIItem::default(),
        Sprite {
            custom_size: Some(Vec2::new(x_size, y_size)),
            image_mode: SpriteImageMode::Sliced(TextureSlicer {
                border: BorderRect {
                    left: 2.0,
                    right: 2.0,
                    top: 2.0,
                    bottom: 2.0,
                },
                sides_scale_mode: SliceScaleMode::Tile { stretch_value: 2.0 },
                ..default()
            }),
            image: ui_tex,
            anchor: Anchor::BottomLeft,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(0.0),
            left: Val::Px(0.0),
            ..default()
        },
    ));

    //also spawn a DiplaysMessage component to display messages
    commands.spawn((
        DisplayMessage,
        Text {
            0: "...".to_string(),

            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(3.0),
            left: Val::Px(3.0),
            ..default()
        },
        UIItem::default(),
    ));
}

#[derive(Component, Reflect)]
#[reflect(Component)]
/// A component that marks an entity as a placeholder object, these are preview objects that are not yet placed into the scene.
/// note: this is separate from our placeholder resources, we could create many of these if we are prepping to place a lot of items in one keypress
pub struct PlaceholderObjectTag;

pub fn update_placeholder<T: SignificantComponent + Component + Default>(
    mut commands: Commands,

    state: ResMut<State<EditorState>>,
    mut placeholder: ResMut<PlaceholderHandle>,
    textures: Res<TextureHandles>,

    crosshairs: Query<(&Crosshair, &Transform)>,
    placeholders: Query<(Entity, &PlaceholderObjectTag)>,
) {
    //delete any existing placeholder objects
    for (e, _) in placeholders.iter() {
        commands.entity(e).despawn_recursive();
    }


    let m = match state.get() {
        EditorState::Editing(EditingComponent::Tile) => { 't' }
        EditorState::Editing(EditingComponent::Collider) => { 'c' }
        _ => {
            'r' //use selection rects as fallback
        }
    };
    //update the placeholder object to be the major type of the current editing mode
    placeholder.0 = textures.0.get(&m).unwrap().clone();

    let t = crosshairs.single().1.clone();

    commands.spawn((
        T::default(),
        Sprite {
            image: placeholder.0.clone(),
            rect: Some(Rect {
                min: Vec2::new(0.0, 0.0),
                max: Vec2::new(TILE_SIZE as f32, TILE_SIZE as f32),
            }),
            ..default()
        },
        Transform {
            translation: Vec3::new(t.translation.x, t.translation.y, UI_Z_LAYER),
            ..default()
        },
        UIItem {
            ..default()
        },
        ui::NormalModeUI, // give it normalModeUI so it will just be destroyed when we exit normalMode
        PlaceholderObjectTag, //tag it as a placeholder object so we can delete it when we switch from this mode.
    ));
}

pub fn trigger_placeholder_update(
    mut ev: EventReader<UpdatePlaceholderEvent>,
    mut commands: Commands,

    state: ResMut<State<EditorState>>,
    placeholder: ResMut<PlaceholderHandle>,

    // crosshairs: Query<(&Crosshair, &Transform)>,
    placeholders: Query<(Entity, &PlaceholderObjectTag)>,
) {
    for e in ev.read() {
        println!("Placeholder Update Event Triggered");
        match state.get() {
            EditorState::Editing(EditingComponent::Tile) => { 't' }
            EditorState::Editing(EditingComponent::Collider) => { 'c' }
            _ => {
                '_' //blank here as a fallback to cause a panic
            }
        };

        //update the placeholder object's texture rect to align with the rect given by the event
        for ent in placeholders.iter() {
            commands.entity(ent.0).insert(
                Sprite {
                    image: placeholder.0.clone(),
                    rect: Some(Rect {
                        min: Vec2::new(e.rect.min.x, e.rect.min.y),
                        max: Vec2::new(e.rect.max.x, e.rect.max.y),
                    }),
                    ..default()
                }
            );
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct NormalModeUI;
