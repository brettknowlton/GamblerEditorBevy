use crate::EditorObject;

use super::*;
use bevy::math::Rect;
use bevy_rapier2d::prelude::Sensor;
use bevy_rapier2d::prelude::*;

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

        let pos = Vec3::new(coord.0 as f32, coord.1 as f32, -5.0);
        let mut ec = commands.spawn((
            T::from_rect(item_rect, coord),
            Visibility::default(),
            Transform {
                translation: pos,
                scale: Vec3::new(TILE_SCALE as f32, TILE_SCALE as f32, 1.0),
                ..default()
            },
            EditorObject {
                coordinate: TCoordinate::new(item_type, coord),
                internal_type: item.internal_type,
                zone_id: item.zone_id,
            },
        ));

        //insert specific rapier components based on the item type here
        match item_type {
            'c' => {
                ec.insert((
                    Collider::cuboid((TILE_SIZE /2)as f32, (TILE_SIZE/2) as f32),
                    Restitution::coefficient(0.05),
                ));
            },
            't' => {
                println!("Adding tile, no rigidbody");
            },
            'a' => {
                println!("Adding actor, dynamic rigidbody");
                ec.insert((RigidBody::Dynamic, Collider::cuboid(1., 1.)));
            },
            _ => panic!("Invalid item type"),
        }
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
