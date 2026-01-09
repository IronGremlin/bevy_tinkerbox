use std::any::TypeId;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {}

#[derive(Component)]
pub struct RideAlongComponent(pub TypeId);

pub fn view_only_component(name: String, type_id: TypeId) -> impl Bundle {
    (
        Node { ..default() },
        RideAlongComponent(type_id),
        children![(Text::new(name), TextFont::from_font_size(11.),)],
    )
}
