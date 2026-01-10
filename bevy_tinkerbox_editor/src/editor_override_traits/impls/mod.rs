use crate::{
    DynamicComponentUiUpdateEvent, FieldAccessPath, FieldUiRequestedFor, ImageNodeSansHandle,
    UiCtxt,
    editor_override_traits::{
        EditorFieldUI, EditorHeaderUI, EditorPerFieldUI, ReflectEditorFieldUI,
        ReflectEditorHeaderUI, ReflectEditorPerFieldUI,
    },
    widgets::general::{FormControl, FormElement, FormElementMarker},
};
use bevy::{
    asset::io::file::FileAssetReader, ecs::world::DeferredWorld, image::ImageLoader, prelude::*,
    ui_widgets::observe,
};
use bevy_file_dialog::{EntityFileDialogExt, EntityScopedDialogEvent};
pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        PreStartup,
        manually_registering_trait_data_for_fun_and_profit,
    );
    app.add_systems(Update, transform_editor_presentation);
}

fn manually_registering_trait_data_for_fun_and_profit(reg: ResMut<AppTypeRegistry>) {
    let mut registry = reg.write();
    registry.register_type_data::<bool, ReflectEditorFieldUI>();
    registry.register_type_data::<Handle<Image>, ReflectEditorFieldUI>();
    registry.register_type_data::<Transform, ReflectEditorPerFieldUI>();
    registry.register_type_data::<Transform, ReflectEditorHeaderUI>();
}

impl EditorPerFieldUI for Transform {
    fn construct_per_field_ui(&self, ctxt: &UiCtxt, commands: &mut Commands) {
        let watcher = |modify: fn(Mut<TransformEditorFormControl>)| {
            observe(
                move |source: On<Pointer<Click>>,
                      traverse: Query<&FormElement>,
                      control: Query<Entity, With<TransformEditorFormControl>>,
                      mut commands: Commands| {
                    if let Some(root) = traverse
                        .related::<FormElement>(source.entity)
                        .and_then(|x| control.get(x).ok())
                    {
                        commands
                            .entity(root)
                            .entry::<TransformEditorFormControl>()
                            .and_modify(modify);
                    }
                },
            )
        };
        match ctxt.path() {
            "translation" => {
                commands.entity(ctxt.ui_anchor()).with_child((
                    Node::default(),
                    children![(
                        Node {
                            width: px(16.),
                            height: px(16.),
                            ..default()
                        },
                        FormElementMarker,
                        TranslateToggle,
                        watcher(|mut form| {
                            if form.global_show_widget {
                                form.show_translation_gizmo = !form.show_translation_gizmo;
                            }
                        }),
                        ImageNodeSansHandle::from_path("lucide/move-white.png".to_owned())
                    )],
                ));
            }
            "rotation" => {
                commands.entity(ctxt.ui_anchor()).with_child((
                    Node::default(),
                    children![(
                        Node {
                            width: px(16.),
                            height: px(16.),
                            ..default()
                        },
                        FormElementMarker,
                        RotateToggle,
                        watcher(|mut form| {
                            if form.global_show_widget {
                                form.show_rotation_gizmo = !form.show_rotation_gizmo;
                            }
                        }),
                        ImageNodeSansHandle::from_path("lucide/rotate-ccw-white.png".to_owned())
                    )],
                ));
            }
            "scale" => {
                commands.entity(ctxt.ui_anchor()).with_child((
                    Node::default(),
                    children![(
                        Node {
                            width: px(16.),
                            height: px(16.),
                            ..default()
                        },
                        FormElementMarker,
                        ScaleToggle,
                        watcher(|mut form| {
                            if form.global_show_widget {
                                form.show_scale_gizmo = !form.show_scale_gizmo;
                            }
                        }),
                        ImageNodeSansHandle::from_path("lucide/maximize-2-white.png".to_owned())
                    )],
                ));
            }
            _ => {}
        }
        ctxt.next(commands);
    }
}

#[derive(Component)]
struct GlobalShow;
#[derive(Component)]
struct DimmsToggle;
#[derive(Component)]
struct TranslateToggle;
#[derive(Component)]
struct RotateToggle;
#[derive(Component)]
struct ScaleToggle;

#[derive(Clone, Reflect, PartialEq, Eq, Default)]
pub enum Dimensionality {
    #[default]
    Is2D,
    Is3D,
}
#[derive(Component, Default)]
pub struct TransformEditorFormControl {
    pub global_show_widget: bool,
    pub world_dimensions: Dimensionality,
    pub show_scale_gizmo: bool,
    pub show_rotation_gizmo: bool,
    pub show_translation_gizmo: bool,
}

fn transform_editor_presentation(
    transform_control_states: Query<
        (&TransformEditorFormControl, &FormControl),
        Changed<TransformEditorFormControl>,
    >,
    //TODO - This is an overwhelmingly stupid way to do this, I really need to figure something more graceful.
    mut buttons: Query<
        (
            &mut ImageNode,
            Has<GlobalShow>,
            Has<DimmsToggle>,
            Has<TranslateToggle>,
            Has<RotateToggle>,
            Has<ScaleToggle>,
        ),
        (With<FormElement>, Without<TransformEditorFormControl>),
    >,
) {
    for (state, form_elements) in transform_control_states.iter() {
        for entity in form_elements.collection() {
            if let Ok((mut node, isshow, isdimms, istranslate, isrotate, isscale)) =
                buttons.get_mut(*entity)
            {
                {
                    let lit = if isshow && state.global_show_widget {
                        true
                    } else if isdimms && state.world_dimensions == Dimensionality::Is2D {
                        true
                    } else if istranslate
                        && state.global_show_widget
                        && state.show_translation_gizmo
                    {
                        true
                    } else if isrotate && state.global_show_widget && state.show_rotation_gizmo {
                        true
                    } else if isscale && state.global_show_widget && state.show_scale_gizmo {
                        true
                    } else {
                        false
                    };
                    node.color = if lit {
                        Color::from(Srgba::GREEN)
                    } else {
                        Color::from(Srgba::WHITE)
                    }
                }
            }
        }
    }
}

impl EditorHeaderUI for Transform {
    fn construct_header_ui(&self, ui_anchor: Entity, commands: &mut Commands) {
        let watcher = |modify: fn(Mut<TransformEditorFormControl>)| {
            observe(
                move |source: On<Pointer<Click>>,
                      traverse: Query<&FormElement>,
                      control: Query<Entity, With<TransformEditorFormControl>>,
                      mut commands: Commands| {
                    if let Some(root) = traverse
                        .related::<FormElement>(source.entity)
                        .and_then(|x| control.get(x).ok())
                    {
                        commands
                            .entity(root)
                            .entry::<TransformEditorFormControl>()
                            .and_modify(modify);
                    }
                },
            )
        };
        commands
            .entity(ui_anchor)
            .insert(TransformEditorFormControl::default());
        commands.entity(ui_anchor).with_child((
            Node::default(),
            children![
                (
                    Node {
                        width: px(16.),
                        height: px(16.),
                        ..default()
                    },
                    FormElementMarker,
                    GlobalShow,
                    watcher(|mut form| {
                        form.global_show_widget = !form.global_show_widget;
                    }),
                    ImageNodeSansHandle::from_path("lucide/eye-white.png".to_owned())
                ),
                (
                    Node {
                        width: px(16.),
                        height: px(16.),
                        ..default()
                    },
                    FormElementMarker,
                    DimmsToggle,
                    watcher(|mut form| {
                        use Dimensionality::*;
                        form.world_dimensions = match form.world_dimensions {
                            Is2D => Is3D,
                            Is3D => Is2D,
                        };
                    }),
                    ImageNodeSansHandle::from_path("lucide/move-3d-white.png".to_owned())
                )
            ],
        ));
    }
}

impl EditorFieldUI for bool {
    fn construct_field_ui(&self, entity: Entity, commands: &mut Commands) {
        let click_watcher = observe(
            |source: On<Pointer<Click>>, world: DeferredWorld, mut commands: Commands| {
                let my_cap = world
                    .entity(source.entity)
                    .get_components::<&FieldAccessPath>()
                    .unwrap();

                let component = world
                    .get_reflect(my_cap.owning_entity, my_cap.component_type_id)
                    .unwrap();
                let state = my_cap.path.element::<bool>(component).unwrap();

                commands.trigger(DynamicComponentUiUpdateEvent::new(
                    source.entity,
                    Box::new(!state.clone()),
                    my_cap.clone(),
                ));
            },
        );

        let world_watcher = observe(
            |source: On<FieldUiRequestedFor>, world: DeferredWorld, mut commands: Commands| {
                let my_cap = world
                    .entity(source.component_ui_root)
                    .get_components::<&FieldAccessPath>()
                    .unwrap();
                let component = world
                    .get_reflect(my_cap.owning_entity, my_cap.component_type_id)
                    .unwrap();
                let state = my_cap.path.element::<bool>(component).unwrap();
                commands.entity(source.component_ui_root).insert(if *state {
                    BackgroundColor::from(Srgba::GREEN)
                } else {
                    BackgroundColor::from(Srgba::BLACK)
                });
            },
        );
        commands.entity(entity).insert((
            Node {
                width: px(12.0),
                height: px(12.0),
                border: UiRect::all(px(1.)),
                ..default()
            },
            BorderColor::all(Srgba::WHITE),
            if *self {
                BackgroundColor::from(Srgba::GREEN)
            } else {
                BackgroundColor::from(Srgba::BLACK)
            },
            click_watcher,
            world_watcher,
        ));
    }
}
impl EditorPerFieldUI for Sprite {
    fn construct_per_field_ui(&self, ctxt: &UiCtxt, commands: &mut Commands) {
        match ctxt.path() {
            "image" => {}
            "texture_atlas" => {}
            "color" => {}
            "custom_size" => {}
            "rect" => {}
            "image_mode" => {}
            _ => {}
        };
        ctxt.next(commands);
    }
}

impl EditorFieldUI for Handle<Image> {
    fn construct_field_ui(&self, entity: Entity, commands: &mut Commands) {
        commands.entity(entity).insert((
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
                            let root = FileAssetReader::get_base_path().join("assets");
                            let path = full_path.strip_prefix(root).unwrap();
                            let handle: Handle<Image> = assets.load(path.to_owned());
                            s_commands.trigger(DynamicComponentUiUpdateEvent::new(
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
                |src: On<FieldUiRequestedFor>, world: DeferredWorld, mut s_commands: Commands| {
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
