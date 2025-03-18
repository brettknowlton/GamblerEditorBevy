use tools::SignificantComponent;

use crate::EditorObject;

use super::*;


#[derive(Component, Default, Debug, Clone)]
pub struct SelectionRect {
    pub start: Coordinate,
    pub end: Option<Coordinate>,
}
impl SelectionRect {
    pub fn new(start: Coordinate) -> Self {
        Self { start, end: Some(start) }
    }

    pub fn start(start: Coordinate) -> Self {
        SelectionRect {
            start,
            end: None,
        }
    }

    pub fn end(&mut self, end: Coordinate) {
        self.end = Some(end);
    }
}

impl SignificantComponent for SelectionRect {

    fn place<T: SignificantComponent + Component>(commands: &mut Commands, item: crate::EditorObject, item_type: char, coord: Coordinate, from: &Query<(Entity, &EditorObject), With<T>>) {
        commands.spawn((
            SelectionRect {
                start: coord,
                end: Some(coord.add_tile_scale()),
            },
            Transform {
                translation: Vec3::new(coord.0 as f32, coord.1 as f32, -5.0),
                scale: Vec3::new(TILE_SCALE as f32, TILE_SCALE as f32, 1.0),
                ..default()
            },
            EditorObject {
                internal_type: item.get_internal_type(),
                coordinate: TCoordinate::new('T', coord),
                zone_id: item.zone_id,
            },
            
        ));
    }
    
    fn place_rectangle(rect: Rect, commands: Commands) {
        todo!();
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

    // //optionally, we can call end_and_make to create a bevy component
    // pub fn end_and_make<T: Component + SignificantComponent>(self, end: Coordinate, commands: Commands){
    //     let end = end;
    //     let start = self.selection_rect.unwrap().start;

    //     let rect = Rect::from_corners(start.into(), end.into());
    //     T::use_rectangle_tool(rect, commands);

    // }

    // pub fn create_bevy_component<T: Component + SignificantComponent>(self, commands: Commands){
    //     if let Some(end) = self.selection_rect.clone().unwrap().end {
    //         let rect = Rect::from_corners(self.selection_rect.unwrap().start.into(), end.into());
    //         T::use_rectangle_tool(rect, commands);
    //     }
    // }

}