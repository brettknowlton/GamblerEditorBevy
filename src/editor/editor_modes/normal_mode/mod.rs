use bevy::prelude::*;

use crate::{
    coordinate::Coordinate, editor_modes::editor_mode::EditorModePlugin,
    significant_component::SignificantComponent, CustomInput, EditorObjectKind, EditorState,
};

pub struct NormalModePlugin;

#[derive(Component, Reflect, Debug, Clone, Default)]
pub struct NormalObject;
impl SignificantComponent for NormalObject {
    fn place_rectangle(_rect: Rect, _commands: Commands) {
        todo!()
    }

    fn at_coordinate(_coord: Coordinate) -> Self {
        todo!()
    }

    fn relevant_editor_object(&self) -> super::EditorObjectKind {
        EditorObjectKind::Other
    }
    fn to_type_string(&self) -> String {
        "normal_object(nothing)".to_string()
    }
}

impl Plugin for NormalModePlugin {
    fn build(&self, app: &mut App) {
        Self::build_plugin::<NormalObject>(app);
    }
}

impl EditorModePlugin for NormalModePlugin {
    fn mode() -> EditorState {
        EditorState::Normal
    }

    fn exit_mode(mut bottom_bar: ResMut<crate::message_display::MessageDisplay>) {
        bottom_bar.send_mode_exit_message("Normal");
    }

    fn get_mode_kb() -> Vec<(CustomInput, String)> {
        vec![
            (
                CustomInput::Combo(vec![KeyCode::ControlLeft, KeyCode::KeyT]),
                "Test Scene".into(),
            ),
            (CustomInput::Single(KeyCode::KeyQ), "Quit Edit Mode".into()),
        ]
    }
}
