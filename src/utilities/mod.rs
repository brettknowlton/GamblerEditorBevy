use bevy::prelude::*;

pub mod resources;



//Helper Functions
pub fn despawn_all<T: Component>(mut commands: Commands, to_despawn: Query<Entity, With<T>>) {
    for e in to_despawn.iter() {
        commands.entity(e).despawn_recursive();
    }
}


//Helper Components
#[derive(Component, Debug, Hash, Eq, PartialEq, Clone, Copy, Default)]
pub(crate) struct Coordinate(pub i64, pub i64);
