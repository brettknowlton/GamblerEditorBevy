use crate::{SCALED_TILE_HEIGHT, SCALED_TILE_WIDTH, TILE_SCALE};

use super::*;

pub trait SignificantComponent: Component + Default + Reflect {
    fn relevant_editor_object(&self) ->EditorObjectKind;
    fn to_type_string(&self) -> String;
    
    fn place_rectangle(rect: Rect, commands: Commands);
    fn at_coordinate(coord: Coordinate) -> Self;

    fn place<T: SignificantComponent>(
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
        let mut should_place = true;

        if let Some(item) = editor_objects.iter().find(|(_, t)| {
            if t.coordinate == item.coordinate && t.get_major_type() == item.get_major_type() {
                if let Some(t_internal) = t.get_internal_type() {
                    if let Some(item_internal) = item.get_internal_type() {
                        if t_internal == item_internal {
                            should_place = false;
                            return false;
                        }
                    }
                }
                true
            } else {
                false
            }
        }) {
            //remove the old item
            commands.entity(item.0).despawn();
        }
        if should_place {
            // calculate the position for the Transform component, this will be in the center of the item's hitbox locked to the grid
            let pos = Vec3::new(
                (item.coordinate.x + (SCALED_TILE_WIDTH / 2) as i64) as f32,
                (item.coordinate.y + (SCALED_TILE_HEIGHT / 2) as i64) as f32,
                -5.0,
            );
            println!("item's position offset calculated: {:?}", pos);

            commands.spawn((
                T::at_coordinate(item.coordinate),
                Visibility::default(),
                Transform {
                    translation: pos,
                    scale: Vec3::new(TILE_SCALE as f32, TILE_SCALE as f32, 1.0),
                    ..default()
                },
                item.clone(),
            ));
        }
    }

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
