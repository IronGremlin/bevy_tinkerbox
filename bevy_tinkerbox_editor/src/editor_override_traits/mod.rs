use bevy::{app::App, ecs::system::Commands, reflect::reflect_trait};

use crate::ui_context_core::UiCtxt;

pub mod impls;
#[reflect_trait]
pub trait EditorHeaderUI {
    fn construct_header_ui(&self, ctxt: &UiCtxt, commands: &mut Commands);
}

#[reflect_trait]
pub trait EditorPerFieldUI {
    fn construct_per_field_ui(&self, ctxt: &UiCtxt, commands: &mut Commands);
}

#[reflect_trait]
pub trait EditorFieldUI {
    fn construct_field_ui(&self, ctxt: &UiCtxt, commands: &mut Commands);
}

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(impls::plugin);
}
