use bevy::prelude::*;

use crate::editor::EditorObjectKind;

pub mod resources;
pub mod selection;
pub mod tools;
pub mod coordinate;


//Helper Functions
pub fn despawn_all<T: Component>(mut commands: Commands, to_despawn: Query<Entity, With<T>>) {
    for e in to_despawn.iter() {
        commands.entity(e).despawn();
    }
}

