/// Traits which supply UI widget definition for Components.


use bevy::{app::App, ecs::system::Commands, reflect::reflect_trait};

use crate::ui_context_core::UiCtxt;

pub mod impls;
/// A trait for defining custom header data on a component.
#[reflect_trait]
pub trait EditorHeaderUI {
    fn construct_header_ui(&self, ctxt: &UiCtxt, commands: &mut Commands);
}
/// A trait for defining per-property overrides on component types.
///
/// Useful for types who wish to largely rely on the reflection generated UI,
/// but have some fields for which that wouldn't work.
///
#[reflect_trait]
pub trait EditorPerFieldUI {
    fn construct_per_field_ui(&self, ctxt: &UiCtxt, commands: &mut Commands);
}
/// A trait which allows completely overriding the reflection UI for a given type.
///
/// Note this can also be used to completely override the UI for an entire component.
///
#[reflect_trait]
pub trait EditorFieldUI {
    fn construct_field_ui(&self, ctxt: &UiCtxt, commands: &mut Commands);
}

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(impls::plugin);
}
