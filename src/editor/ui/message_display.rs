use bevy::prelude::*;

use crate::coordinate::Coordinate;

macro_rules! send_message {
    ($key:expr, $messages:expr, $message:expr) => {
        $messages.push(($key, $message.to_string()));
    };
    (_) => {
        $messages.push((None, " ".to_string()));
    };
}

#[derive(Message)]
pub struct BottomBarUpdate;

pub struct BottomBarPlugin;

impl Plugin for BottomBarPlugin {
    fn build(&self, app: &mut App) {
        Self::bottom_bar_plugin(app);
    }
}

impl BottomBarPlugin {
    pub fn bottom_bar_plugin(app: &mut App) {
        app.init_resource::<MessageDisplay>()
            .init_resource::<EditorBottomBarMessage>()
            //begin update system to send debug messages (to bottom bar and to console)
            .add_systems(Update, Self::send_messages);
    }
    pub fn send_messages(mut bottom_bar: ResMut<MessageDisplay>) {
        if let Some((_, message)) = bottom_bar.queue.first() {
            bottom_bar.displayed = format!("{message}",);
        }
        //push any messages into the in-game console and leave the last one in our BottomBarMessage for display
        let item = bottom_bar.queue.pop();
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
}

#[derive(Resource)]
pub struct MessageDisplay {
    pub displayed: String,
    pub queue: Vec<(Option<char>, String)>,
}

impl MessageDisplay {
    pub fn send_message(&mut self, message: impl Into<String>) {
        send_message!(Some('i'), self.queue, message.into());
    }

    pub fn send_place_eo_message(&mut self, label: &str, coord: Coordinate) {
        self.send_message(format!("Placed {label} at: ({}, {})", coord.x, coord.y));
    }

    pub fn send_remove_eo_message(&mut self, label: &str, coord: Coordinate) {
        self.send_message(format!("Removing {label} at: ({}, {})", coord.x, coord.y));
    }

    pub fn send_mode_exit_message(&mut self, label: &str) {
        self.send_message(format!("Exiting {label} Editing Mode"));
    }

    pub fn send_mode_enter_message(&mut self, label: &str) {
        self.send_message(format!("Entering {label} Editing Mode"));
    }

    pub fn send_setting_update_message(&mut self, label: &str) {
        self.send_message(format!("Updating {label} Setting"));
    }
}

impl Default for MessageDisplay {
    fn default() -> Self {
        Self {
            displayed: "".to_string(),
            queue: vec![],
        }
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
