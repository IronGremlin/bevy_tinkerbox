use bevy::{
    app::App,
    ecs::{entity::Entity, system::Commands},
    reflect::reflect_trait,
};

use crate::UiCtxt;
pub mod impls;
#[reflect_trait]
pub trait EditorHeaderUI {
    fn construct_header_ui(&self, ui_anchor: Entity, commands: &mut Commands);
}

#[reflect_trait]
pub trait EditorPerFieldUI {
    fn construct_per_field_ui(&self, ctxt: &UiCtxt, commands: &mut Commands);
}

#[reflect_trait]
pub trait EditorFieldUI {
    fn construct_field_ui(&self, entity: Entity, commands: &mut Commands);
}

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(impls::plugin);
}
