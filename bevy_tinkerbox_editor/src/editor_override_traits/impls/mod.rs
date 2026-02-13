use crate::{
    FieldAccessPath, ImageNodeSansHandle, RefreshInputFields, UpdateComponentFieldValue,
    editor_override_traits::{
        EditorFieldUI, ReflectEditorFieldUI, ReflectEditorHeaderUI, ReflectEditorPerFieldUI,
    },
    ui_context_core::{ComponentUiStepContext, UiCtxt, field_layout},
};
use bevy::{
    asset::io::file::FileAssetReader,
    ecs::world::DeferredWorld,
    feathers::controls::checkbox,
    image::ImageLoader,
    prelude::*,
    ui::Checked,
    ui_widgets::{ValueChange, checkbox_self_update, observe},
};
use bevy_file_dialog::{EntityFileDialogExt, EntityScopedDialogEvent};
pub mod children;
pub mod sprite;
pub mod transform;
pub(super) fn plugin(app: &mut App) {
    app.add_plugins(MeshPickingPlugin);
    app.add_plugins((transform::plugin, sprite::plugin));

    app.add_systems(
        PreStartup,
        manually_registering_trait_data_for_fun_and_profit,
    );
}

fn manually_registering_trait_data_for_fun_and_profit(reg: ResMut<AppTypeRegistry>) {
    let mut registry = reg.write();
    registry.register_type_data::<bool, ReflectEditorFieldUI>();
    registry.register_type_data::<Handle<Image>, ReflectEditorFieldUI>();
    registry.register_type_data::<Transform, ReflectEditorPerFieldUI>();
    registry.register_type_data::<Transform, ReflectEditorHeaderUI>();
    registry.register_type_data::<Name, ReflectEditorFieldUI>();
    registry.register_type_data::<Children, ReflectEditorFieldUI>();
    registry.register_type_data::<Sprite, ReflectEditorPerFieldUI>();
}

impl EditorFieldUI for Name {
    fn construct_field_ui(&self, ctxt: &UiCtxt, commands: &mut Commands) {
        commands.entity(ctxt.ui_anchor()).insert(field_layout());

        let name_struct_info = ctxt.value_type_info().as_struct().unwrap();
        let name_field_info = name_struct_info
            .field("name")
            .and_then(|t| t.type_info())
            .unwrap();

        let next_step = ComponentUiStepContext {
            local_ui_focus: ctxt.ui_anchor(),
            local_type_info: name_field_info,
            local_path: "name".to_owned(),
            local_name: "name".to_owned(),
        };
        ctxt.override_step(next_step, commands);
    }
}

impl EditorFieldUI for bool {
    fn construct_field_ui(&self, ctxt: &UiCtxt, commands: &mut Commands) {
        let click_watcher = observe(checkbox_self_update);
        let watcher = observe(
            |source: On<ValueChange<bool>>, world: DeferredWorld, mut commands: Commands| {
                let my_cap = world
                    .entity(source.event_target())
                    .get_components::<&FieldAccessPath>()
                    .unwrap();

                let component = world
                    .get_reflect(my_cap.owning_entity, my_cap.component_type_id)
                    .unwrap();
                let state = my_cap.path.element::<bool>(component).unwrap();

                commands.trigger(UpdateComponentFieldValue::new(
                    source.event_target(),
                    Box::new(!state.clone()),
                    my_cap.clone(),
                ));
            },
        );

        let world_watcher = observe(
            |source: On<RefreshInputFields>, world: DeferredWorld, mut commands: Commands| {
                let my_cap = world
                    .entity(source.component_ui_root)
                    .get_components::<&FieldAccessPath>()
                    .unwrap();
                let component = world
                    .get_reflect(my_cap.owning_entity, my_cap.component_type_id)
                    .unwrap();
                let state = my_cap.path.element::<bool>(component).unwrap();
                if *state {
                    commands.entity(source.component_ui_root).insert(Checked);
                } else {
                    commands
                        .entity(source.component_ui_root)
                        .remove::<Checked>();
                }
            },
        );
        if *self {
            commands.entity(ctxt.ui_anchor()).insert((
                checkbox(Checked, Spawn(())),
                click_watcher,
                watcher,
                world_watcher,
            ));
        } else {
            commands.entity(ctxt.ui_anchor()).insert((
                checkbox((), Spawn(())),
                click_watcher,
                watcher,
                world_watcher,
            ));
        };
    }
}

impl EditorFieldUI for Handle<Image> {
    fn construct_field_ui(&self, ctxt: &UiCtxt, commands: &mut Commands) {
        let entity = ctxt.ui_anchor();
        commands.entity(ctxt.ui_anchor()).insert((
            Node {
                display: Display::Grid,
                grid_auto_flow: GridAutoFlow::Column,
                column_gap: px(3.),
                width: percent(100.),
                ..default()
            },
            observe(
                |event: On<EntityScopedDialogEvent>,
                 world: DeferredWorld,
                 mut s_commands: Commands| {
                    let my_cap = world
                        .entity(event.event_target())
                        .components::<&FieldAccessPath>()
                        .clone();
                    let assets = world.resource::<AssetServer>();
                    use bevy_file_dialog::EntityScopedDialogResult::*;
                    match event.clone().result {
                        Pick(file_pick) => {
                            let full_path = file_pick.path.clone().to_owned();
                            //TODO - figure out how to do this without hard-coding this string.
                            let root = FileAssetReader::get_base_path()
                                .parent()
                                .unwrap()
                                .join("sample_project_bin")
                                .join("assets");

                            let fp_ = full_path.clone();
                            let fp = fp_.to_string_lossy();
                            let r_ = root.clone();
                            let r = r_.to_string_lossy();

                            let path = full_path
                                .strip_prefix(root)
                                .map_err(move |e| {
                                    info!("Could not strip {r} path from {fp}",);
                                    info!("{:?}", e.to_string());
                                    e
                                })
                                .unwrap();
                            let handle: Handle<Image> = assets.load(path.to_owned());
                            s_commands.trigger(UpdateComponentFieldValue::new(
                                event.event_target(),
                                Box::new(handle),
                                my_cap.clone(),
                            ));
                        }
                        _ => {}
                    };
                },
            ),
            observe(
                |src: On<RefreshInputFields>, world: DeferredWorld, mut s_commands: Commands| {
                    let my_cap = world
                        .entity(src.component_ui_root)
                        .get_components::<&FieldAccessPath>()
                        .unwrap();
                    let component = world
                        .get_reflect(my_cap.owning_entity, my_cap.component_type_id)
                        .unwrap();
                    let image = my_cap.path.element::<Handle<Image>>(component).unwrap();
                    let path = image
                        .path()
                        .map(|x| x.to_string())
                        .unwrap_or("<<unkown>>".to_owned());
                    let text_node = world
                        .entity(src.event_target())
                        .components::<&Children>()
                        .get(1)
                        .unwrap();
                    let text_entity = world
                        .entity(*text_node)
                        .components::<&Children>()
                        .get(0)
                        .unwrap();
                    s_commands
                        .entity(*text_entity)
                        .entry::<Text>()
                        .and_modify(|mut txt| {
                            txt.0 = path;
                        });
                },
            ),
            children![
                (Node {
                    width: px(5.0),
                    ..default()
                },),
                (
                    Node::default(),
                    BackgroundColor(Color::from(Srgba::gray(0.3))),
                    children![(
                        Text::new(
                            self.path()
                                .map(|n| n.to_string())
                                .unwrap_or("<<unknown>>".to_owned())
                        ),
                        TextLayout::new_with_justify(Justify::Center),
                        TextFont::from_font_size(8.)
                    )]
                ),
                (
                    Node {
                        width: px(12.0),
                        height: px(12.0),
                        border: UiRect::all(px(1.)),
                        ..default()
                    },
                    ImageNodeSansHandle::from_path("lucide/folder-white.png".to_owned()),
                    BorderColor::all(Srgba::WHITE),
                    observe(move |_: On<Pointer<Click>>, mut s_commands: Commands| {
                        s_commands
                            .entity(entity)
                            .with_dialog()
                            .add_filter("image", ImageLoader::SUPPORTED_FILE_EXTENSIONS)
                            .set_title("Select Image")
                            .pick_file_path();
                    }),
                )
            ],
        ));
    }
}
