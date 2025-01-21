use bevy::prelude::*;
use tools::SignificantComponent;

use super::*;

pub fn spawn_general_editor_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    //spawn a bar on the bottom using our default UI menu sprite

    let x_offset = -DEFAULT_WINDOW_WIDTH / 2.0;
    let y_offset = -DEFAULT_WINDOW_HEIGHT / 2.0;

    let x_size = DEFAULT_WINDOW_WIDTH;
    let y_size = 30.0;
    
    let ui_texpath = PathBuf::from("textures/menus/menu1.png");
    let ui_tex = asset_server.load(ui_texpath);
    commands.spawn((
        UIItem::default(),
        Sprite {
            custom_size: Some(Vec2::new(x_size, y_size)),
            image_mode: SpriteImageMode::Sliced(TextureSlicer{
                border: BorderRect {
                    left: 2.,
                    right: 2.0,
                    top: 2.0,
                    bottom: 2.0,
                },
                sides_scale_mode: SliceScaleMode::Tile { stretch_value: (2.) },
                ..default()

            }),
            image: ui_tex,
            anchor: Anchor::BottomLeft,
            ..default()
        },
        Transform {
            translation: Vec3::new(x_offset, y_offset, UI_Z_LAYER),
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
        Node{
            position_type: PositionType::Absolute,
            bottom: Val::Px(3.0),
            left: Val::Px(3.0),
            ..default()
        },
        UIItem::default(),
    ));
    
}

pub fn send_messages(mut queued_messages: ResMut<EditorBottomBarQueuedMessages>, mut display_message: Query<(&DisplayMessage, &mut Text)>) {
    //push any messages into the in-game console and leave the last one in our BottomBarMessage for display
    let item = queued_messages.messages.pop(); {
        match item {
            Some((k, m)) => {
                let k = k.unwrap_or(' ');
                println!("{}:> {}", k, m);

                let (_, mut t) = display_message.single_mut();
                **t = format!(":]>{m}",);
            },
            None => {}
        }
    };


    // if let Some((_, message)) = queued_messages.messages.first() {
    //     let (_, mut t) = display_message.single_mut();
    //     **t = format!("{message}",);
    // }

}

macro_rules! send_message {
    ($key:expr, $messages:expr, $message:expr) => {
        $messages.messages.push(($key, $message.to_string()));
    };
    (_) => {
        $messages.messages.push((None, " ".to_string()));
    };
}

///Tagging component for the placeholder object we want to replace the current placeholder object with
#[derive(Component)]
pub struct NextPlaceholder;

pub fn update_placeholder<T: SignificantComponent, Component>(
    mut placeholder: ResMut<PlaceholderObject::<T>>,
    replacement: Query<(Entity, &NextPlaceholder)>,
) {
    //update the placeholder object
    let (entity, component) = replacement.single().unwrap();
    let mut placeholder = placeholder.0;
    placeholder.0 = component.clone();
}

// pub fn despawn_placeholder<T: SignificantComponent>(
//     mut commands: Commands,
//     placeholder: Res<PlaceholderObject<T>>,
//     query: Query<Entity, With<T>>,
// ) {
//     //despawn the placeholder object
// }



pub fn create_placeholder(
    mut commands: Commands,
    spritesheet: Res<TilesheetHandle>,
    crosshairs: Query<(&Transform, &Crosshair)>,

    cur_state: State<EditorState>,
) {
    match cur_state.get() {
        EditorState::Editing(x) => {
            match x{

            }
        }
        _ => {}
    }
}


// pub fn create_tilemode_ui(mut commands: Commands, asset_server: Res<AssetServer>, crosshairs: Query<(&Transform, &Crosshair)>, tilesheet_handle: Res<TilesheetHandle>) {


#[derive(Component)]
pub struct PlaceholderObject<T: SignificantComponent>(pub T);






// #[derive(Component)]
// pub struct NormalModeObject{
    
// };
// impl SignificantComponent for NormalModeObject {
//     fn get_coordinate(&self) -> Coordinate {
//         Coordinate(0, 0)
//     }
// }

#[derive(Component)]
pub struct DisplayMessage;