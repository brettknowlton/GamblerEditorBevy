use crate::EditorObject;

use super::*;
use bevy::math::Rect;

pub trait SignificantComponent {
    fn place_rectangle(rect: Rect, commands: Commands);
    fn place<T: SignificantComponent + Component + Default>(
        commands: &mut Commands,
        item: EditorObject,
        item_type: char,
        coord: Coordinate,
        from: &Query<(Entity, &EditorObject), With<T>>,
    ) {
        //check if an item of the same kind already exists at this location and remove it if it does
        if let Some(item) = from
            .iter()
            .find(|(_, t)| t.coordinate == TCoordinate::new(item_type, coord))
        {
            //remove the old item
            commands.entity(item.0).despawn();
        }

        let item_rect = Rect::new(
            //creating a hitbox with a 1-tile square size in the place location.
            coord.0 as f32 * TILE_SCALE as f32,
            coord.1 as f32 * TILE_SCALE as f32,
            coord.0 as f32 + 1.0 * TILE_SCALE as f32,
            coord.1 as f32 + 1.0 * TILE_SCALE as f32,
        );

        let pos = Vec3::new((coord.0 + (TILE_SIZE*TILE_SCALE / 2) as i64) as f32, (coord.1 + (TILE_SIZE*TILE_SCALE / 2) as i64) as f32, -5.0);
        let eo = EditorObject {
            coordinate: TCoordinate::new(item_type, coord),
            internal_type: item.internal_type,
            zone_id: item.zone_id,
        };


        commands.spawn((
            T::from_rect(item_rect, coord),
            Visibility::default(),
            Transform {
                translation: pos,
                scale: Vec3::new(TILE_SCALE as f32, TILE_SCALE as f32, 1.0),
                ..default()
            },
            eo.clone(),
        ));
    }

    fn from_rect(rect: Rect, coord: Coordinate) -> Self;

    fn remove<T: SignificantComponent + Component>(
        commands: &mut Commands,
        coord: Coordinate,
        from: &Query<(Entity, &EditorObject), With<T>>,
    ) {
        //check if a tile already exists at this location and remove it if it does
        if let Some(item) = from
            .iter()
            .find(|(_, t)| t.get_coordinate() == TCoordinate::new(t.get_major_type(), coord))
        {
            //remove the old tile
            commands.entity(item.0).despawn();
        }
    }
}

#[derive(Reflect, Debug, Clone, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}
