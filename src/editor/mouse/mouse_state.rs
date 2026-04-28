use bevy::prelude::*;

use crate::{
    coordinate::Coordinate,
    message_display::MessageDisplay,
    significant_component::SignificantComponent,
    EditorObject, EditorObjectKind,
};

#[derive(Resource)]
pub struct MouseState {
    dragging_btn: Option<MouseButton>,
    start_pos: Option<Vec2>,
    pub mode: MouseToolKind,
}

impl Default for MouseState {
    fn default() -> Self {
        Self {
            dragging_btn: None,
            start_pos: None,
            mode: MouseToolKind::Pointer,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
/// An enum representing the different mouse tools available in the editor, such as pointer for placing/removing objects and eyedropper for sampling existing objects.
pub enum MouseToolKind {
    Pointer,
    Pencil,
    Eraser,
    Eyedropper,
}

impl MouseState {
    fn left_click<T: SignificantComponent>(
        &self,
        commands: &mut Commands,
        at_coordinate: Coordinate,
        items: Query<(Entity, &EditorObject), With<T>>,
        current_item_kind: EditorObjectKind,
        mut bottom_bar: ResMut<MessageDisplay>,
    ) -> Option<EditorObjectKind> {
        match self.mode {
            MouseToolKind::Pointer | MouseToolKind::Pencil => {
                let to_place = EditorObject::new(current_item_kind, at_coordinate);
                T::place(commands, to_place, &items);
                bottom_bar.send_place_eo_message(&current_item_kind.to_string(), at_coordinate);
                None
            }
            MouseToolKind::Eraser => {
                T::remove(commands, at_coordinate, current_item_kind, &items);
                bottom_bar.send_remove_eo_message(&current_item_kind.to_string(), at_coordinate);
                None
            }
            MouseToolKind::Eyedropper => {
                if let Some((_, obj)) = items.iter().find(|(_, obj)| obj.coordinate == at_coordinate) {
                    bottom_bar.send_message(format!("Eyedropped: {:?} at {:?}", obj.get_major_type(), obj.coordinate));
                    Some(obj.kind)
                } else {
                    bottom_bar.send_message(format!("Eyedropped: nothing at {:?}", at_coordinate));
                    None
                }
            }
        }
    }

    fn right_click<T: SignificantComponent>(
        &self,
        commands: &mut Commands,
        at_coordinate: Coordinate,
        items: Query<(Entity, &EditorObject), With<T>>,
        current_item_kind: EditorObjectKind,
        mut bottom_bar: ResMut<MessageDisplay>,
    ) -> Option<EditorObjectKind> {
        match self.mode {
            MouseToolKind::Pointer | MouseToolKind::Eraser => {
                T::remove(commands, at_coordinate, current_item_kind, &items);
                bottom_bar.send_remove_eo_message(&current_item_kind.to_string(), at_coordinate);
                None
            }
            MouseToolKind::Pencil => {
                // Right-click on pencil also places (mirrors left-click)
                let to_place = EditorObject::new(current_item_kind, at_coordinate);
                T::place(commands, to_place, &items);
                bottom_bar.send_place_eo_message(&current_item_kind.to_string(), at_coordinate);
                None
            }
            MouseToolKind::Eyedropper => {
                if let Some((_, obj)) = items.iter().find(|(_, obj)| obj.coordinate == at_coordinate) {
                    bottom_bar.send_message(format!("Eyedropped: {:?} at {:?}", obj.get_major_type(), obj.coordinate));
                    Some(obj.kind)
                } else {
                    bottom_bar.send_message(format!("Eyedropped: nothing at {:?}", at_coordinate));
                    None
                }
            }
        }
    }
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
    pub fn start_drag(&mut self, btn: MouseButton, pos: Vec2) {
        self.dragging_btn = Some(btn);
        self.start_pos = Some(pos);
    }

    /// Ends the current drag action, resetting the dragging state to indicate that no mouse button is currently being used for dragging.
    ///
    /// This method should be called when the mouse button used for dragging is released, or when the drag action is otherwise completed or cancelled.
    /// After calling this method, `is_dragging()` will return false and `dragging_button()` will return None until a new drag action is started.
    pub fn end_drag(&mut self) {
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
    pub fn drag_action<T: SignificantComponent + Component + Default>(
        &self,
        mut commands: Commands,
        at_coordinate: Coordinate,
        current_item_kind: EditorObjectKind,
        bottom_bar: ResMut<MessageDisplay>,
        items: Query<(Entity, &EditorObject), With<T>>,
    ) -> Option<EditorObjectKind> {
        match self.dragging_btn {
            Some(MouseButton::Left) => {
                self.left_click::<T>(&mut commands, at_coordinate, items, current_item_kind, bottom_bar)
            }
            Some(MouseButton::Right) => {
                self.right_click::<T>(&mut commands, at_coordinate, items, current_item_kind, bottom_bar)
            }
            _ => None,
        }
    }
}
