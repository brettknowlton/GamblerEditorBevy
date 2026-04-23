use bevy::{input::mouse::MouseButton, prelude::*, window::PrimaryWindow};
use bevy_egui::EguiContexts;

use crate::{
    MouseToolKind, coordinate::Coordinate, editor_object::{EditorObject, EditorObjectKind, significant_component::SignificantComponent, tile::Tile}, message_display::MessageDisplay
};

#[derive(Resource)]
pub struct Dragging {
    dragging_btn: Option<MouseButton>,
    start_pos: Option<Vec2>,
}

impl Default for Dragging {
    fn default() -> Self {
        Self {
            dragging_btn: None,
            start_pos: None,
        }
    }
}

impl Dragging {
    /// Returns true if the user is currently dragging with any mouse button, false otherwise.
    pub fn is_dragging(&self) -> bool {
        self.dragging_btn.is_some()
    }

    /// Returns the mouse button that is currently being used for dragging, or None if not dragging.
    pub fn dragging_button(&self) -> Option<MouseButton> {
        self.dragging_btn
    }

    /// Starts a drag action with the specified mouse button and starting position.
    ///
    /// # Arguments
    ///
    /// * `btn` - The mouse button that is being used for dragging.
    /// * `pos` - The starting position of the drag action, typically the cursor position at the time of the mouse button press.
    fn start_drag(&mut self, btn: MouseButton, pos: Vec2) {
        self.dragging_btn = Some(btn);
        self.start_pos = Some(pos);
    }

    /// Ends the current drag action, resetting the dragging state to indicate that no mouse button is currently being used for dragging.
    ///
    /// This method should be called when the mouse button used for dragging is released, or when the drag action is otherwise completed or cancelled.
    /// After calling this method, `is_dragging()` will return false and `dragging_button()` will return None until a new drag action is started.
    fn end_drag(&mut self) {
        self.dragging_btn = None;
        self.start_pos = None;
    }

    /// Handles the drag action based on the current mouse tool and the mouse button being used for dragging.
    ///
    /// This method should be called during a drag action to perform the appropriate actions based on the editor's current mouse tool and the state of the dragging.
    ///
    /// # Arguments
    ///
    /// * `commands` - A mutable reference to the Bevy Commands, used to issue commands for placing or removing objects in the scene.
    /// * `mouse_mode` - The current mouse tool mode, which determines how the drag action should be interpreted (e.g., pointer for placing/removing, eyedropper for sampling).
    /// * `snapped_coord` - The coordinate where the drag action is occurring, typically snapped to the grid if grid snapping is enabled.
    /// * `to_place` - An identifier for what specific object to place, used when placing new objects during a drag action.
    /// * `message_queue` - A mutable reference to the editor's bottom bar message queue, used to send messages about actions taken during the drag (e.g., placing or removing objects).
    /// * `tiles` - A query of existing tile entities in the scene, used to determine what objects are currently present when performing actions like placement or removal.
    ///
    /// The behavior of this method will depend on the current `mouse_mode` and which mouse button is being used for dragging. For example:
    /// - If in pointer mode and dragging with the left mouse button, it may place a new tile at the specified coordinate.
    /// - If in pointer mode and dragging with the right mouse button, it may remove an existing tile at the specified coordinate.
    /// - If in eyedropper mode, it may sample information from an existing object at the specified coordinate without modifying the scene.
    pub fn drag_action(
        &self,
        mut commands: &mut Commands,
        mouse_mode: MouseToolKind,
        snapped_coord: Coordinate,
        to_place: u64,
        mut bottom_bar: ResMut<MessageDisplay>,
        tiles: &Query<(Entity, &EditorObject), With<Tile>>,
    ) {
        match mouse_mode {
            MouseToolKind::Pointer => match self.dragging_btn {
                Some(MouseButton::Left) => {
                    let to_place = EditorObject::new(
                        EditorObjectKind::Tile,
                        to_place,
                        snapped_coord,
                        EditorObjectKind::Other,
                    );
                    Tile::place(&mut commands, to_place, &tiles);
                    bottom_bar.send_place_eo_message("tile", snapped_coord);
                }
                Some(MouseButton::Right) => {
                    Tile::remove(&mut commands, snapped_coord, EditorObjectKind::Tile, &tiles);
                    bottom_bar.send_remove_eo_message("tiles", snapped_coord);
                }
                _ => {}
            },
            MouseToolKind::Eyedropper => {}
        }
    }
}

pub fn listen_click_events(
    input: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut egui_contexts: EguiContexts,
    mut dragging: ResMut<Dragging>,
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
            dragging.end_drag();
        }
        return;
    }

    if input.just_pressed(MouseButton::Left) {
        dragging.start_drag(
            MouseButton::Left,
            window.cursor_position().unwrap_or_default(),
        );
    }
    if input.just_released(MouseButton::Left) {
        dragging.end_drag();
    }

    if input.just_pressed(MouseButton::Right) {
        dragging.start_drag(
            MouseButton::Right,
            window.cursor_position().unwrap_or_default(),
        );
    }
    if input.just_released(MouseButton::Right) {
        dragging.end_drag();
    }

    if input.just_pressed(MouseButton::Middle) {
        dragging.start_drag(
            MouseButton::Middle,
            window.cursor_position().unwrap_or_default(),
        );
    }
    if input.just_released(MouseButton::Middle) {
        dragging.end_drag();
    }
}
