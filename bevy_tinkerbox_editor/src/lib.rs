use bevy::prelude::*;

pub mod editor;
mod theme;
mod widget_functions;

pub struct ComponentEditorPlugin;
impl Plugin for ComponentEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(editor::plugin);
    }
}

#[cfg(test)]
mod tests {}
