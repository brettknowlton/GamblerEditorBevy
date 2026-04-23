use bevy::prelude::*;

use super::menu::CameraLockedUI;

use crate::{
    coordinate::TCoordinate, editor_object::significant_component::SignificantComponent, Crosshair,
    EditorState, PlaceholderHandle, TextureHandles, TILE_SIZE, UI_Z_LAYER,
};

#[derive(Message)]
pub struct UpdatePlaceholderMessage {
    pub tcoord: TCoordinate,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
/// A component that marks an entity as a placeholder object, these are preview objects that are not yet placed into the scene.
/// note: this is separate from our placeholder resources, we could create many of these if we are prepping to place a lot of items in one keypress
pub struct PlaceholderObjectTag;

pub fn update_placeholder<T: SignificantComponent + Component + Default>(
    mut commands: Commands,

    state: ResMut<State<EditorState>>,
    mut placeholder: ResMut<PlaceholderHandle>,
    textures: Res<TextureHandles>,

    crosshairs: Query<(&Crosshair, &Transform)>,
    placeholders: Query<(Entity, &PlaceholderObjectTag)>,
) {
    //delete any existing placeholder objects
    for (e, _) in placeholders.iter() {
        commands.entity(e).despawn();
    }

    if let Some(m) = state.get().get_editing_kind() {
        //update the placeholder object to be the major type of the current editing mode
        let Some(texture) = textures.0.get(&m) else {
            return;
        };
        placeholder.0 = texture.clone();
    }

    let Ok((_, t)) = crosshairs.single() else {
        return;
    };

    commands.spawn((
        T::default(),
        Sprite {
            image: placeholder.0.clone(),
            rect: Some(Rect {
                min: Vec2::new(0.0, 0.0),
                max: Vec2::new(TILE_SIZE as f32, TILE_SIZE as f32),
            }),
            ..default()
        },
        Transform {
            translation: Vec3::new(t.translation.x, t.translation.y, UI_Z_LAYER),
            ..default()
        },
        CameraLockedUI { ..default() },
        PlaceholderObjectTag, //tag it as a placeholder object so we can delete it when we switch from this mode.
    ));
}

pub fn trigger_placeholder_update(
    mut ev: MessageReader<UpdatePlaceholderMessage>,
    mut commands: Commands,

    placeholder: ResMut<PlaceholderHandle>,

    // crosshairs: Query<(&Crosshair, &Transform)>,
    placeholders: Query<(Entity, &PlaceholderObjectTag)>,
) {
    for _ in ev.read() {
        println!("Placeholder Update Event Triggered");
        //update the placeholder object's texture rect to align with the rect given by the event
        for ent in placeholders.iter() {
            commands.entity(ent.0).insert(Sprite {
                image: placeholder.0.clone(),
                ..default()
            });
        }
    }
}
