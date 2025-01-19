use bevy::{prelude::*, text::cosmic_text::Selection};
use tools::SignificantComponent;
use super::*;
use crate::EditorObject;


#[derive(Resource, Debug, Component)]
/// A placeholder object is a temporary object that is used to represent an object that will be created later,
///  we display this as a half-alpha sprite of the sprite that would-be placed..
pub struct PlaceholderObject(pub EditorObject);


/// A handle to the tilesheet image.
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct TilesheetHandle(pub Handle<Image>);

#[derive(Resource)]
pub struct EditorBottomBarDisplayed{pub text: String}
impl Default for EditorBottomBarDisplayed {
    fn default() -> Self {
        Self{text: "".to_string()}
    }
}

#[derive(Resource)]
pub struct EditorBottomBarQueuedMessages{pub messages: Vec<(Option<char>, String)>}
impl Default for EditorBottomBarQueuedMessages {
    fn default() -> Self {
        Self{messages: vec![]}
    }
}

#[derive(Resource)]
pub struct EditorBottomBarMessage{pub text: String}
impl Default for EditorBottomBarMessage {
    fn default() -> Self {
        Self{text: "".to_string()}
    }
}

#[derive(Component, Default, Debug, Clone)]
pub struct SelectionRect {
    pub start: Coordinate,
    pub end: Option<Coordinate>,
}
impl SelectionRect {
    pub fn new(start: Coordinate) -> Self {
        Self { start, end: Some(start) }
    }//pressing the control key down will call start
    pub fn start(start: Coordinate) -> Self {
        SelectionRect {
            start,
            end: None,
        }
    }

    //releasing the control key will call end
    pub fn end(&mut self, end: Coordinate) {
        self.end = Some(end);
    }
}


#[derive(Resource, Default, Debug, Clone)]
pub struct ActiveSelection {
    pub selection_rect: Option<SelectionRect>,
}

impl ActiveSelection {

    pub fn set_start(mut self, start: Coordinate) {
        self.selection_rect = Some(SelectionRect::start(start));
    }
    pub fn set_end(mut self, end: Coordinate) {
        if let Some(ref mut rect) = self.selection_rect {
            rect.end = Some(end);
        }
    }

    pub fn from(rect: SelectionRect) -> Self {
        Self {
            selection_rect: Some(rect),
        }
    }

    //optionally, we can call end_and_make to create a bevy component
    pub fn end_and_make<T: Component + SignificantComponent>(self, end: Coordinate, commands: Commands){
        let end = end;
        let start = self.selection_rect.unwrap().start;

        let rect = Rect::from_corners(start.into(), end.into());
        T::use_rectangle_tool(rect, commands);

    }

    pub fn create_bevy_component<T: Component + SignificantComponent>(self, commands: Commands){
        if let Some(end) = self.selection_rect.clone().unwrap().end {
            let rect = Rect::from_corners(self.selection_rect.unwrap().start.into(), end.into());
            T::use_rectangle_tool(rect, commands);
        }
    }

}