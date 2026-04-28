use crate::{editor_modes::significant_component::SignificantComponent, TILE_SCALE};

use super::*;

#[derive(Component, Default, Debug, Clone, Reflect)]
pub struct SelectionRect {
    pub start: Coordinate,
    pub end: Option<Coordinate>,
}

impl SelectionRect {
    pub fn new(start: Coordinate) -> Self {
        Self {
            start,
            end: Some(start),
        }
    }

    pub fn start(start: Coordinate) -> Self {
        SelectionRect { start, end: None }
    }

    pub fn end(&mut self, end: Coordinate) {
        self.end = Some(end);
    }
}

impl SignificantComponent for SelectionRect {
    fn relevant_editor_object(&self) -> crate::EditorObjectKind {
        return crate::EditorObjectKind::Other;
    }
    fn to_type_string(&self) -> String {
        "selection_rect".to_string()
    }

    fn place<T: SignificantComponent + Component>(
        commands: &mut Commands,
        item: EditorObject,
        _from: &Query<(Entity, &EditorObject), With<T>>,
    ) {
        commands.spawn((
            SelectionRect {
                start: item.coordinate,
                end: Some(item.coordinate.add_tile_scale()),
            },
            Transform {
                translation: Vec3::new(item.coordinate.x as f32, item.coordinate.y as f32, -5.0),
                scale: Vec3::new(TILE_SCALE as f32, TILE_SCALE as f32, 1.0),
                ..default()
            },
            item,
        ));
    }

    fn place_rectangle(_rect: Rect, _commands: Commands) {
        todo!();
    }

    fn at_coordinate(_coord: Coordinate) -> Self {
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
}
