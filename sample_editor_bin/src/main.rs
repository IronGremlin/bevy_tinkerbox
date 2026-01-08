use bevy::prelude::*;
use bevy_tinkerbox_editor::ComponentEditorPlugin;
// We have to do this to get all our stuff to reflect over
#[allow(unused_imports)]
use sample_project_lib::*;

fn main() -> AppExit {
    App::new()
        .add_plugins((DefaultPlugins, ComponentEditorPlugin))
        .add_systems(Startup, setup)
        .add_systems(PostStartup, bevy_tinkerbox_editor::spawn_editor)
        .run()
}
fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}
