use bevy::prelude::*;

use crate::coordinate::Coordinate;

macro_rules! send_message {
    ($key:expr, $messages:expr, $message:expr) => {
        $messages.messages.push(($key, $message.to_string()));
    };
    (_) => {
        $messages.messages.push((None, " ".to_string()));
    };
}
pub(crate) use send_message;

pub fn bottom_bar_plugin(app: &mut App) {
    app.init_resource::<EditorBottomBarDisplayed>()
        .init_resource::<EditorBottomBarMessage>()
        .init_resource::<EditorBottomBarQueuedMessages>()
        //begin update system to send debug messages (to bottom bar and to console)
        .add_systems(Update, send_messages);
}

pub fn send_place_eo_message(
    message_queue: &mut EditorBottomBarQueuedMessages,
    label: &str,
    coord: Coordinate,
) {
    send_message!(
        Some('i'),
        message_queue,
        format!("Placed {label} at: ({}, {})", coord.x, coord.y)
    );
}

pub fn send_remove_eo_message(
    message_queue: &mut EditorBottomBarQueuedMessages,
    label: &str,
    coord: Coordinate,
) {
    send_message!(
        Some('i'),
        message_queue,
        format!("Removing {label} at: ({}, {})", coord.x, coord.y)
    );
}

pub fn send_mode_exit_message(message_queue: &mut EditorBottomBarQueuedMessages, label: &str) {
    send_message!(
        Some('i'),
        message_queue,
        format!("Exiting {label} Editing Mode")
    );
}

pub fn send_messages(
    mut queued_messages: ResMut<EditorBottomBarQueuedMessages>,
    mut display_message: ResMut<EditorBottomBarDisplayed>,
) {
    if let Some((_, message)) = queued_messages.messages.first() {
        display_message.text = format!("{message}",);
    }
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
}

#[derive(Resource)]
pub struct EditorBottomBarDisplayed {
    pub text: String,
}
impl Default for EditorBottomBarDisplayed {
    fn default() -> Self {
        Self {
            text: "".to_string(),
        }
    }
}

#[derive(Resource)]
pub struct EditorBottomBarQueuedMessages {
    pub messages: Vec<(Option<char>, String)>,
}
impl Default for EditorBottomBarQueuedMessages {
    fn default() -> Self {
        Self { messages: vec![] }
    }
}

#[derive(Resource)]
pub struct EditorBottomBarMessage {
    pub text: String,
}
impl Default for EditorBottomBarMessage {
    fn default() -> Self {
        Self {
            text: "".to_string(),
        }
    }
}
