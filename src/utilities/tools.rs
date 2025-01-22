use crate::EditorObject;

use super::*;
use bevy::math::Rect;

pub trait SignificantComponent {
    fn place_rectangle(rect: Rect, commands: Commands);
    fn place<T: SignificantComponent + Component + Default>(commands: &mut Commands, item: EditorObject, coord: Coordinate, from: &Query<(Entity, &EditorObject), With<T>>) {

        //check if a tile already exists at this location and remove it if it does
        if let Some(item) = from.iter().find(|(_, t)| t.coordinate == TCoordinate::new('t', coord)) {
            //remove the old tile
            commands.entity(item.0).despawn();
        }

        commands.spawn((
            T::default(),
            Transform {
                translation: Vec3::new(coord.0 as f32, coord.1 as f32, -5.0),
                scale: Vec3::new(TILE_SCALE as f32, TILE_SCALE as f32, 1.0),
                ..default()
            },
            EditorObject {
                coordinate: TCoordinate::new('T', coord),
                internal_type: item.internal_type,
            },
        ));
    }
    
    fn remove<T: SignificantComponent + Component>(commands: &mut Commands, coord: Coordinate, from: &Query<(Entity, &EditorObject), With<T>>){
        //check if a tile already exists at this location and remove it if it does
        if let Some(item) = from.iter().find(|(_, t)| t.get_coordinate() == TCoordinate::new(t.get_major_type(), coord)) {
            //remove the old tile
            commands.entity(item.0).despawn();
        }
    }
}
