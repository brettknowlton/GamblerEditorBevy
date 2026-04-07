use crate::{EditorObject, SCALED_TILE_HEIGHT, SCALED_TILE_WIDTH};

use super::*;
use bevy::{math::Rect, sprite::Anchor};

pub trait SignificantComponent {
    fn place_rectangle(rect: Rect, commands: Commands);
    fn place<T: SignificantComponent + Component + Default>(
        commands: &mut Commands,
        item: EditorObject,
        editor_objects: &Query<(Entity, &EditorObject), With<T>>,
    ) {
        println!(
            "Placing item of type {:?} at coordinate {:?}",
            item.get_major_type(),
            item.coordinate
        );

        //check if an item of the same kind already exists at this location and remove it if it does
        if let Some(item) = editor_objects.iter().find(|(_, t)| {
            t.coordinate == item.coordinate && t.get_major_type() == item.get_major_type()
        }) {
            //remove the old item
            commands.entity(item.0).despawn();
        }

        //create a rectangle representing the item's hitbox (one grid space)
        let item_rect = Rect::new(
            item.coordinate.0 as f32 * TILE_SCALE as f32,
            item.coordinate.1 as f32 * TILE_SCALE as f32,
            (item.coordinate.0 as f32 + 1.0) * TILE_SCALE as f32,
            (item.coordinate.1 as f32 + 1.0) * TILE_SCALE as f32,
        );
        println!("item's rectangle calculated: {:?}", item_rect);

        // calculate the position for the Transform component, this will be in the center of the item's hitbox locked to the grid
        let pos = Vec3::new(
            (item.coordinate.0 + (SCALED_TILE_WIDTH / 2) as i64) as f32,
            (item.coordinate.1 + (SCALED_TILE_HEIGHT / 2) as i64) as f32,
            -5.0,
        );
        println!("item's position offset calculated: {:?}", pos);

        commands.spawn((
            T::from_rect(item_rect, item.coordinate),
            Visibility::default(),
            Transform {
                translation: pos,
                scale: Vec3::new(TILE_SCALE as f32, TILE_SCALE as f32, 1.0),
                ..default()
            },
            item.clone(),
        ));
    }

    fn from_rect(rect: Rect, coord: Coordinate) -> Self;

    fn remove<T: SignificantComponent + Component>(
        commands: &mut Commands,
        coord: Coordinate,
        kind: EditorObjectKind,
        editor_objects: &Query<(Entity, &EditorObject), With<T>>,
    ) {
        //check if a tile already exists at this location and remove it if it does
        //check if an item of the same kind already exists at this location and remove it if it does
        if let Some(item) = editor_objects
            .iter()
            .find(|(_, t)| t.coordinate == coord && t.get_major_type() == kind)
        {
            //remove the old item
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
