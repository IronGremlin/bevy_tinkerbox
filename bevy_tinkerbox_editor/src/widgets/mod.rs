use bevy::app::App;

pub mod add_entity_button;
pub mod component_browser;
pub mod field_input;
pub mod general;
pub mod view_only_component;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        component_browser::plugin,
        general::plugin,
        field_input::plugin,
    ));
}
