use bevy::{prelude::*, ui_widgets::observe};

use crate::{
    ComponentUiFor, ComponentUisFor, EntityUiRoot, ImageNodeSansHandle,
    editor_override_traits::{EditorHeaderUI, EditorPerFieldUI},
    ui_context_core::UiCtxt,
    widgets::general::{FormControl, FormControlSubject, FormElement, FormElementMarker},
};
pub(super) fn plugin(app: &mut App) {
    app.add_observer(transform_editor_widget_spawner);
    app.add_systems(Update, transform_editor_presentation);
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
        (Entity, &TransformEditorFormControl, &FormControl),
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
    mut commands: Commands,
    find_root: Query<(Option<&ChildOf>, Option<&EntityUiRoot>)>,
) {
    for (form_holder, state, form_elements) in transform_control_states.iter() {
        let mut cursor = find_root.get(form_holder);
        while cursor.is_ok() {
            match cursor {
                Ok((_, Some(uiroot))) => {
                    commands.trigger(ConstructTransformWidgetFor {
                        entity: uiroot.component_holder,
                        op: if state.global_show_widget && state.show_translation_gizmo {
                            info!("ja bro: {:?}", uiroot.component_holder);
                            Op::Create
                        } else {
                            Op::Destroy
                        },
                    });
                    break;
                }
                Ok((Some(parent), _)) => {
                    cursor = find_root.get(parent.0);
                    continue;
                }
                Ok((_, _)) => {
                    info!("Failed Entity UI Root traversal");
                    break;
                }
                Err(_) => {
                    info!("Failed Entity UI Root traversal");
                    break;
                }
            }
        }
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
    fn construct_header_ui(&self, ctxt: &UiCtxt, commands: &mut Commands) {
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
            .entity(ctxt.ui_anchor())
            .insert(TransformEditorFormControl::default())
            .insert(FormControlSubject);
        commands.entity(ctxt.ui_anchor()).with_child((
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
#[derive(Component)]
struct TransformWidget;
#[derive(EntityEvent)]
struct ConstructTransformWidgetFor {
    entity: Entity,
    op: Op,
}
#[derive(Eq, PartialEq)]
enum Op {
    Create,
    Destroy,
}
fn transform_editor_widget_spawner(
    event: On<ConstructTransformWidgetFor>,
    mut commands: Commands,
    world_target: Query<&ComponentUisFor>,
    global_transforms: Query<&GlobalTransform>,
    gidgets: Query<&TransformWidget>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if event.op == Op::Destroy {
        let Ok(uis_for) = world_target.get(event.event_target()) else {
            return;
        };
        for n in uis_for.iter() {
            if let Ok(_gidget) = gidgets.get(n) {
                commands.entity(n).despawn();
                return;
            }
        }

        return;
    }
    let up_arrow_m = meshes.add(Triangle2d::new(
        Vec2::new(0., 2.),
        Vec2::new(-1., 0.),
        Vec2::new(1., 0.),
    ));
    let right_arrow_m = meshes.add(Triangle2d::new(
        Vec2::new(0., 1.),
        Vec2::new(0., -1.),
        Vec2::new(2., 0.),
    ));
    let horizontal_shaft_m = meshes.add(Rectangle::new(4., 0.5));
    let vertical_shaft_m = meshes.add(Rectangle::new(0.5, 4.));
    let drag_box_m = meshes.add(Rectangle::new(3., 3.));
    let vert_drag_border_m = meshes.add(Segment2d::new(Vec2::new(0., 0.), Vec2::new(0., 3.)));
    let horiz_drag_border_m = meshes.add(Segment2d::new(Vec2::new(0., 0.), Vec2::new(3., 0.)));
    let h_color = Srgba::GREEN;
    let v_color = Srgba::RED;
    let box_border_color = Srgba::rgb_u8(235, 221, 80);
    let box_interior_color = box_border_color.clone().with_alpha(0.3);
    let mut shape = |m: Handle<Mesh>, mmc: Srgba, pos: (f32, f32, f32)| {
        let (x, y, z) = pos;
        (
            Mesh2d(m),
            MeshMaterial2d(materials.add(Color::from(mmc))),
            Transform::from_xyz(x, y, z),
        )
    };
    #[derive(Clone, Copy)]
    enum Swizz {
        X,
        Y,
        XY,
    }
    fn drag_op(s: Swizz) -> impl Fn(Vec2, Vec3) -> Vec3 {
        move |delta: Vec2, translation: Vec3| match s {
            Swizz::X => Vec2::new(delta.x, 0.).extend(translation.z),
            Swizz::Y => Vec2::new(0., delta.y * -1.).extend(translation.z),
            Swizz::XY => Vec2::new(delta.x, delta.y * -1.).extend(translation.z),
        }
    }

    fn drag_watch(
        swizz: Swizz,
    ) -> impl Fn(On<Pointer<Drag>>, Query<&ComponentUiFor>, Query<&ChildOf>, Query<&mut Transform>)
    {
        move |src: On<Pointer<Drag>>,
              actual_target: Query<&ComponentUiFor>,
              parent: Query<&ChildOf>,
              mut x_form: Query<&mut Transform>| {
            let poppa = parent.get(src.event_target()).unwrap().0;
            let trgt = actual_target.get(poppa).unwrap().target;
            let trgs: Vec<Entity> = vec![poppa, trgt];
            let mut iter = x_form.iter_many_mut(&trgs);
            while let Some(mut transform) = iter.fetch_next() {
                let c_delta = src.event.delta;
                let xform_delta = drag_op(swizz)(c_delta, transform.translation);
                (*transform).translation = transform.translation + xform_delta;
            }
        }
    }

    let starting_pos = global_transforms
        .get(event.event_target())
        .map(|x| x.translation())
        .unwrap_or(Vec3::ZERO);

    let widget = commands
        .spawn((
            (
                Transform::from_translation(starting_pos).with_scale(Vec3::splat(5.)),
                InheritedVisibility::VISIBLE,
                TransformWidget,
            ),
            children![
                (
                    shape(drag_box_m, box_interior_color, (2.5, 2.5, 0.)),
                    observe(drag_watch(Swizz::XY)),
                ),
                shape(horiz_drag_border_m.clone(), box_border_color, (1., 1., 0.1)),
                shape(horiz_drag_border_m, box_border_color, (1., 4., 0.1)),
                shape(vert_drag_border_m.clone(), box_border_color, (1., 1., 0.1)),
                shape(vert_drag_border_m, box_border_color, (4., 1., 0.1)),
                shape(vertical_shaft_m, Srgba::WHITE, (0., 2.5, 0.)),
                shape(horizontal_shaft_m, Srgba::WHITE, (2.5, 0., 0.)),
                (
                    shape(up_arrow_m, v_color, (0., 5., 0.)),
                    observe(drag_watch(Swizz::Y))
                ),
                (
                    shape(right_arrow_m, h_color, (5., 0., 0.)),
                    observe(drag_watch(Swizz::X))
                ),
            ],
        ))
        .id();

    commands
        .entity(event.event_target())
        .add_one_related::<ComponentUiFor>(widget);
}
