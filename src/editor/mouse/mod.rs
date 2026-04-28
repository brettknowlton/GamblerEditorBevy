pub mod mouse_state;
use mouse_state::MouseState;

use bevy::{input::mouse::MouseButton, prelude::*, window::PrimaryWindow};
use bevy_egui::EguiContexts;




pub fn listen_click_events(
    input: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut egui_contexts: EguiContexts,
    mut mouse_state: ResMut<MouseState>,
) {
    let ui_consumed_pointer = egui_contexts
        .ctx_mut()
        .map(|ctx| ctx.wants_pointer_input() || ctx.is_pointer_over_area())
        .unwrap_or(false);

    if ui_consumed_pointer {
        if input.just_released(MouseButton::Left)
            || input.just_released(MouseButton::Right)
            || input.just_released(MouseButton::Middle)
        {
            mouse_state.end_drag();
        }
        return;
    }

    if input.just_pressed(MouseButton::Left) {
        mouse_state.start_drag(
            MouseButton::Left,
            window.cursor_position().unwrap_or_default(),
        );
    }
    if input.just_released(MouseButton::Left) {
        mouse_state.end_drag();
    }

    if input.just_pressed(MouseButton::Right) {
        mouse_state.start_drag(
            MouseButton::Right,
            window.cursor_position().unwrap_or_default(),
        );
    }
    if input.just_released(MouseButton::Right) {
        mouse_state.end_drag();
    }

    if input.just_pressed(MouseButton::Middle) {
        mouse_state.start_drag(
            MouseButton::Middle,
            window.cursor_position().unwrap_or_default(),
        );
    }
    if input.just_released(MouseButton::Middle) {
        mouse_state.end_drag();
    }
}
