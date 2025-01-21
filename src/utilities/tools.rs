use crate::EditorObject;

use super::*;
use bevy::math::Rect;

pub trait SignificantComponent {
    fn get_coordinate(&self) -> &TCoordinate;

    fn use_rectangle_tool(rect: Rect, commands: Commands);
    fn place(commands: &mut Commands, item: EditorObject, coord: Coordinate);
    fn remove<T: SignificantComponent + Component>(
        commands: &mut Commands,
        coord: Coordinate,
        from: Query<(Entity, &T)>
    ) {
        //check if a tile already exists at this location and remove it if it does
        if let Some(item) = from.iter().find(|(_, t)| t.get_coordinate() == coord) {
            //remove the old tile
            commands.entity(item.0).despawn();
        }
    }

    fn create_placeholder<T: Component + SignificantComponent>(new: &T, commands: Commands){
        match new.get_coordinate().type_char {
            'C' => {
                commands.insert_resource();
            }
            'T' => {
                commands.insert_resource(PlaceholderObject(EditorObject::default()));
            }
            _ => {}
        }
    }
}
