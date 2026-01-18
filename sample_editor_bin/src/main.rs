use bevy::prelude::*;
use bevy_tinkerbox_editor::{ComponentEditorPlugin, MainEditorCamera, SceneViewCamera};
// We have to do this to get all our stuff to reflect over
#[allow(unused_imports)]
use sample_project_lib::*;

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins.set(AssetPlugin {
                file_path: "../sample_project_bin/assets".to_owned(),
                ..default()
            }),
            ComponentEditorPlugin,
        ))
        .add_systems(Startup, setup)
        .run()
}
fn setup(mut commands: Commands) {
    commands.spawn((Camera2d, MainEditorCamera, SceneViewCamera));
}
