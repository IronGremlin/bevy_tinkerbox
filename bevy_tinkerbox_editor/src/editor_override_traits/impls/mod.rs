use bevy::{ecs::world::DeferredWorld, prelude::*, ui_widgets::observe};

use crate::{
    DynamicComponentUiUpdateEvent, FieldAccessPath, FieldUiRequestedFor, ImageNodeSansHandle,
    UiCtxt,
    editor_override_traits::{
        EditorFieldUI, EditorHeaderUI, EditorPerFieldUI, ReflectEditorFieldUI,
        ReflectEditorHeaderUI, ReflectEditorPerFieldUI,
    },
    widgets::general::{FormControl, FormElement, FormElementMarker},
};
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
