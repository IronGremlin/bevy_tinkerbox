use std::{any::TypeId, collections::VecDeque};

use bevy::{
    ecs::{
        component::ComponentId, lifecycle::HookContext, reflect::ReflectCommandExt,
        relationship::Relationship, world::DeferredWorld,
    },
    feathers::{
        FeathersPlugin,
        dark_theme::create_dark_theme,
        theme::{ThemeBackgroundColor, UiTheme},
        tokens,
    },
    input_focus::tab_navigation::{TabGroup, TabNavigationPlugin},
    platform::collections::HashSet,
    prelude::*,
    reflect::{DynamicEnum, DynamicStruct, DynamicTuple, DynamicTupleStruct, DynamicVariant},
    ui_widgets::{CheckboxPlugin, RadioGroupPlugin, ScrollbarPlugin, observe},
};
use bevy_file_dialog::FileDialogPlugin;
use bevy_ui_text_input::TextInputPlugin;

use crate::{
    asset_extensions::asset_tracking::ResourceHandles,
    ui_context_core::{
        ComponentIdentifer, EntityUiRoot, FieldAccessPath, RefreshInputFields, SelectedEntityUiRoot,
    },
    widgets::{
        add_entity_button::add_entity_button,
        component_browser::ComponentBrowserWidgetRoot,
        field_input::ValueInputInput,
        general::*,
        scene_actions::{
            load_scene_dialog, load_scene_with_path, save_scene_dialog, save_scene_with_path,
        },
        view_only_component::{RideAlongComponent, view_only_component},
    },
};

mod asset_extensions;
pub mod drag_snap;
mod editor_override_traits;
mod theme;
mod ui_context_core;
pub mod widgets;

pub struct ComponentEditorPlugin;
impl Plugin for ComponentEditorPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Text2d>();
        app.insert_resource(UiTheme(create_dark_theme()));
        //TODO - it feels really weird to be initializing the text input plugin here -
        // but it's also not clear which sub module should own it.
        app.add_plugins((
            TextInputPlugin,
            TabNavigationPlugin,
            ScrollbarPlugin,
            CheckboxPlugin,
            RadioGroupPlugin,
            FeathersPlugin,
            FileDialogPlugin::default(),
        ));
        app.add_plugins((
            asset_extensions::plugin,
            drag_snap::plugin,
            editor_override_traits::plugin,
            widgets::plugin,
            ui_context_core::plugin,
            theme::plugin,
        ));
        app.init_state::<LoadingStatus>();
        app.init_resource::<AppTypeRegistry>();
        app.add_systems(Startup, editor_initialization);
        app.add_observer(on_update_event_dynamic);
        app.add_observer(cascade_field_updates);

        app.configure_sets(OnEnter(LoadingStatus::Complete), EditorConstructionSet);
        app.add_systems(
            OnEnter(LoadingStatus::Complete),
            spawn_editor.in_set(EditorConstructionSet),
        );
    }
}
#[derive(Component)]
pub struct MainEditorCamera;
#[derive(Component)]
pub struct SceneViewCamera;

#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum LoadingStatus {
    #[default]
    Pending,
    Complete,
}
fn editor_initialization(
    resource_handles: Res<ResourceHandles>,
    mut advance: ResMut<NextState<LoadingStatus>>,
) {
    if resource_handles.is_all_done() {
        advance.set(LoadingStatus::Complete);
    }
}
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct EditorConstructionSet;

pub fn spawn_editor(
    mut commands: Commands,
    //TODO - Fix this
    q: Query<Entity, With<Camera2d>>,
) {
    commands.spawn((
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            position_type: PositionType::Absolute,
            left: px(0),
            top: px(0),
            right: px(0),
            bottom: px(0),
            padding: UiRect::all(px(3)),
            row_gap: px(6),
            ..Default::default()
        },
        BackgroundColor(Color::NONE),
        //Whoa man do NOT forget to do this or you'll have a real bad time lmao
        Pickable {
            should_block_lower: false,
            is_hoverable: true,
        },
        UiTargetCamera(q.single().unwrap()),
        TabGroup::default(),
        children![
            (
                Node {
                    width: percent(100.),
                    height: px(16.),
                    ..default()
                },
                children![
                    (
                        Node {
                            width: percent(50.),
                            height: px(16.),
                            border: UiRect::all(px(2.)),
                            ..default()
                        },
                        BorderColor::from(Srgba::WHITE),
                        observe(save_scene_dialog),
                        observe(save_scene_with_path),
                        children![(Text::new("Save"), TextFont::from_font_size(10.))]
                    ),
                    (
                        Node {
                            width: percent(50.),
                            height: px(16.),
                            border: UiRect::all(px(2.)),
                            ..default()
                        },
                        BorderColor::from(Srgba::WHITE),
                        observe(load_scene_dialog),
                        observe(load_scene_with_path),
                        children![(Text::new("Load"), TextFont::from_font_size(10.))]
                    )
                ],
            ),
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    ..default()
                },
                Pickable {
                    should_block_lower: false,
                    is_hoverable: true,
                },
                children![
                    scroll_area_demo(as_bundle((
                        Node {
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        ThemeBackgroundColor(tokens::WINDOW_BG),
                        children![
                            add_entity_button(),
                            (
                                Node {
                                    display: Display::Grid,
                                    ..default()
                                },
                                SelectedEntityUiRoot,
                            )
                        ],
                    ))),
                    (
                        Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: px(3),
                            ..default()
                        },
                        ComponentBrowserWidgetRoot,
                        children![],
                    ),
                ],
            )
        ],
    ));
}

#[derive(Component, Clone)]
#[relationship(relationship_target = ComponentUisFor)]
#[component(on_despawn = component_ui_despawner)]
pub struct ComponentUiFor {
    #[relationship]
    pub target: Entity,
}
fn component_ui_despawner(mut world: DeferredWorld, context: HookContext) {
    let mut find_join_point_state = world
        .try_query::<(Entity, Option<&EntityUiRoot>, Option<&ChildOf>)>()
        .unwrap();
    let find_join_point = world.query(&mut find_join_point_state);

    let mut traversal_cursor = find_join_point.get(context.entity);
    let mut traversal_result: Result<Entity, String> =
        Err("Could not find ancestor with UiRoot".to_owned());
    while traversal_cursor.is_ok() {
        match traversal_cursor {
            Ok((ui_for, Some(_), _)) => {
                traversal_result = Ok(ui_for);
                break;
            }
            Ok((_, None, Some(parent))) => {
                traversal_cursor = find_join_point.get(parent.get());
            }
            _ => {
                // We're only here if we hit the last ancestor and never found our UI root OR
                // we tried to access a dead entity. We don't really care about why this happened exactly so we can just fall back to the default error on our result.
                break;
            }
        }
    }
    match traversal_result {
        Err(e) => {
            info!({ e })
        }
        Ok(ui) => {
            //TODO - GOOD GOD man, please clean this up
            let ui_root = world.entity(ui).get_components::<&EntityUiRoot>().unwrap();
            let type_registration = world
                .entity(context.entity)
                .components::<&ComponentIdentifer>()
                .0
                .clone();
            let c_name = type_registration.type_info().type_path_table().short_path();
            let type_id = type_registration.type_id();

            let mut new_root = ui_root.clone();
            new_root.desired_component_set.remove(&type_id);
            info!(
                "selected len: {:?} | rac len {:?}",
                new_root.desired_component_set.len(),
                new_root.ride_along_components.len()
            );

            let mut dead_reqs = world
                .required_components(type_id)
                .iter()
                .map(|x| *x)
                .collect::<HashSet<TypeId>>();
            dead_reqs.insert(type_id);
            let live_reqs: HashSet<TypeId> = new_root
                .desired_component_set
                .iter()
                .flat_map(|x| world.required_components(*x))
                .collect();
            new_root.ride_along_components = new_root
                .ride_along_components
                .difference(&dead_reqs)
                .map(|x| *x)
                .collect::<HashSet<TypeId>>()
                .union(&live_reqs)
                .map(|x| *x)
                .collect::<HashSet<TypeId>>();
            let dead_letter_bin = dead_reqs
                .difference(&new_root.ride_along_components)
                .filter_map(|x| world.components().get_id(*x))
                .collect::<Vec<ComponentId>>();

            let mut targets: Vec<Entity> = Vec::new();
            let Ok(kids) = world.entity(ui).get_components::<&Children>() else {
                panic!("failed to initialize children for root");
            };
            for kid in kids.iter() {
                let Ok(ride_along) = world.entity(kid).get_components::<&RideAlongComponent>()
                else {
                    continue;
                };
                let (in_dead_reqs, in_live_reqs) = (
                    dead_reqs.contains(&ride_along.0),
                    new_root.ride_along_components.contains(&ride_along.0),
                );

                if in_dead_reqs && !in_live_reqs {
                    targets.push(kid);
                }
            }

            let mut commands = world.commands();
            if new_root.ride_along_components.contains(&type_id) {
                commands
                    .entity(ui)
                    .with_child(view_only_component(c_name.to_owned(), type_id.clone()));
            } else {
                for c_id in dead_letter_bin {
                    commands
                        .entity(new_root.component_holder)
                        .remove_by_id(c_id);
                }

                commands.entity(new_root.component_holder).log_components();
            }
            commands
                .entity(ui)
                .entry::<EntityUiRoot>()
                .and_modify(move |mut w_root| {
                    w_root.desired_component_set.remove(&type_id);
                    w_root.ride_along_components = new_root.ride_along_components;
                });
            for t in targets {
                commands.entity(t).despawn();
            }
        }
    }
}

#[derive(Component, Clone)]
#[relationship_target(relationship = ComponentUiFor)]
pub struct ComponentUisFor(Vec<Entity>);

pub trait WorldRequiredComponentExtension {
    fn required_components(&self, component: TypeId) -> Vec<TypeId>;
}
impl WorldRequiredComponentExtension for World {
    fn required_components(&self, component: TypeId) -> Vec<TypeId> {
        self.components()
            .get_valid_id(component)
            .and_then(|c_id| self.components().get_info(c_id))
            .map(|c_info| {
                c_info
                    .required_components()
                    .iter_ids()
                    .filter_map(|c_id| self.components().get_info(c_id).and_then(|n| n.type_id()))
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[derive(EntityEvent)]
pub struct UpdateComponentFieldValue {
    #[event_target]
    ui_entity: Entity,
    new_value: Box<dyn PartialReflect>,
    component_ui_field_for: FieldAccessPath,
}
impl UpdateComponentFieldValue {
    pub fn validate(&self) -> Result<(), String> {
        self.new_value
            .get_represented_type_info()
            .ok_or("No represented type information".to_owned())
            .and_then(|n| {
                if n.type_id() == self.component_ui_field_for.value_type_id {
                    Ok(())
                } else {
                    Err("Type Id provided does not match Type Id stored".to_owned())
                }
            })
    }
    pub fn new(
        ui_entity: Entity,
        new_value: Box<dyn PartialReflect>,
        component_ui_field_for: FieldAccessPath,
    ) -> Self {
        Self {
            ui_entity,
            new_value,
            component_ui_field_for,
        }
    }
}
fn on_update_event_dynamic(
    source: On<UpdateComponentFieldValue>,
    world: DeferredWorld,
    mut commands: Commands,
) {
    let path = source.component_ui_field_for.path.clone();

    let world_entity = source.component_ui_field_for.owning_entity.clone();

    let component_ref = match world.get_reflect(
        source.component_ui_field_for.owning_entity,
        source.component_ui_field_for.component_type_id,
    ) {
        Ok(f) => f,
        Err(e) => {
            info!("{:?}", e);
            return;
        }
    };
    let mut shadow = component_ref.reflect_clone().unwrap();
    let old_val = match path.reflect_element_mut(shadow.as_partial_reflect_mut()) {
        Ok(f) => f,
        Err(e) => {
            info!("{:?}", e);
            return;
        }
    };
    let _thing = match old_val.try_apply(&*source.new_value) {
        Ok(f) => f,
        Err(e) => {
            info!("{:?}", e);
            return;
        }
    };

    commands.entity(world_entity).insert_reflect(shadow);

    commands.trigger(RefreshInputFields {
        component_ui_root: source.ui_entity,
    });
}

//
fn instantiate_or_die(
    reg: &AppTypeRegistry,
    type_id: TypeId,
    variant_index: Option<usize>,
) -> Result<Box<dyn PartialReflect>, String> {
    let registry = reg.read();
    let registration = registry.get(type_id).ok_or("No Registration for type")?;
    let type_info = registration.type_info();
    let name = type_info.type_path_table().short_path();

    // We should always use reflect default if it's available, unless we've been handed an override in the form of our variant index.
    if let Some(def) = registration.data::<ReflectDefault>()
        && variant_index.is_none()
    {
        return Ok(def.default());
    }

    match type_info.kind() {
        bevy::reflect::ReflectKind::Struct => {
            let struct_info = type_info
                .as_struct()
                .map_err(|_| "Inconceivable reflection cast error")?;
            let mut placeholder = DynamicStruct::default();
            placeholder.set_represented_type(Some(type_info));

            //let mut idx = 0;
            for &field_name in struct_info.field_names() {
                let field_type = struct_info
                    .field(field_name)
                    .map(|x| x.type_id())
                    .ok_or("Nah")?;
                let field_value = instantiate_or_die(reg, field_type, None)?;
                placeholder.insert_boxed(field_name, field_value);
            }
            Ok(Box::new(placeholder))
        }
        bevy::reflect::ReflectKind::TupleStruct => {
            let tuple_struct_info = type_info
                .as_tuple_struct()
                .map_err(|_| "Inconceivable reflection cast error")?;

            let mut placeholder = DynamicTupleStruct::default();
            placeholder.set_represented_type(Some(type_info));
            //let mut idx = 0;
            for unnamed_field in tuple_struct_info.iter() {
                let field_type = unnamed_field
                    .type_info()
                    .map(|x| x.type_id())
                    .ok_or("Nah")?;
                let field_value = instantiate_or_die(reg, field_type, None)?;
                placeholder.insert_boxed(field_value);
            }
            Ok(Box::new(placeholder))
        }
        bevy::reflect::ReflectKind::Tuple => {
            let tuple_info = type_info
                .as_tuple()
                .map_err(|_| "Inconceivable reflection cast error")?;

            let mut placeholder = DynamicTuple::default();
            placeholder.set_represented_type(Some(type_info));
            //let mut idx = 0;
            for unnamed_field in tuple_info.iter() {
                let field_type = unnamed_field
                    .type_info()
                    .map(|x| x.type_id())
                    .ok_or("Nah")?;
                let field_value = instantiate_or_die(reg, field_type, None)?;
                placeholder.insert_boxed(field_value);
            }
            Ok(Box::new(placeholder))
        }

        bevy::reflect::ReflectKind::Enum => {
            let enum_info = type_info
                .as_enum()
                .map_err(|_| "Inconceivable reflection cast error")?;

            let vindex = variant_index.unwrap_or(0);

            let mut placeholder = DynamicEnum::default();
            placeholder.set_represented_type(Some(type_info));

            let variant_info = enum_info
                .variant_at(vindex)
                .ok_or(format!("Invalid variant index {} for {name}", vindex))?;

            match variant_info.variant_type() {
                bevy::reflect::VariantType::Struct => {
                    let mut variant_placeholder = DynamicStruct::default();

                    let v_struct_info = variant_info
                        .as_struct_variant()
                        .map_err(|_| "Inconceivable reflection cast error")?;

                    //let mut idx = 0;
                    for &field_name in v_struct_info.field_names() {
                        let field_type = v_struct_info
                            .field(field_name)
                            .map(|x| x.type_id())
                            .ok_or("Nah")?;
                        let field_value = instantiate_or_die(reg, field_type, None)?;
                        variant_placeholder.insert_boxed(field_name, field_value);
                    }
                    placeholder.set_variant_with_index(
                        vindex,
                        variant_info.name(),
                        DynamicVariant::Struct(variant_placeholder),
                    );
                    Ok(Box::new(placeholder))
                }
                bevy::reflect::VariantType::Tuple => {
                    let mut variant_placeholder = DynamicTuple::default();

                    let v_tuple_info = variant_info
                        .as_tuple_variant()
                        .map_err(|_| "Inconceivable reflection cast error")?;

                    //let mut idx = 0;
                    for unnamed_field in v_tuple_info.iter() {
                        let field_type = unnamed_field
                            .type_info()
                            .map(|x| x.type_id())
                            .ok_or("Nah")?;
                        let field_value = instantiate_or_die(reg, field_type, None)?;
                        variant_placeholder.insert_boxed(field_value);
                    }
                    placeholder.set_variant_with_index(
                        vindex,
                        variant_info.name(),
                        DynamicVariant::Tuple(variant_placeholder),
                    );
                    Ok(Box::new(placeholder))
                }
                bevy::reflect::VariantType::Unit => {
                    let v_unit_info = variant_info
                        .as_unit_variant()
                        .map_err(|_| "Inconceviable reflection cast error")?;
                    placeholder.set_variant_with_index(
                        vindex,
                        v_unit_info.name(),
                        DynamicVariant::Unit,
                    );
                    Ok(Box::new(placeholder))
                }
            }
        }
        //If you haven't implemented default for some collection <T> it's either because you forgot, something scary is going on, or <T> doesn't implement it, in all three cases, you can go kick rocks.
        bevy::reflect::ReflectKind::List
        | bevy::reflect::ReflectKind::Array
        | bevy::reflect::ReflectKind::Map
        | bevy::reflect::ReflectKind::Set
        | bevy::reflect::ReflectKind::Opaque => {
            Err(format!("ReflectDefault not implemented for {name}"))
        }
    }
}

#[derive(Component, Clone)]
#[component(on_add = image_node_sans_handle_added)]
pub struct ImageNodeSansHandle {
    pub color: Color,
    pub path_to_image: String,
    pub texture_atlas: Option<TextureAtlas>,
    pub flip_x: bool,
    pub flip_y: bool,
    pub rect: Option<Rect>,
    pub image_mode: NodeImageMode,
}
impl From<ImageNode> for ImageNodeSansHandle {
    fn from(value: ImageNode) -> Self {
        let ImageNode {
            color,
            image,
            texture_atlas,
            flip_x,
            flip_y,
            rect,
            image_mode,
        } = value;
        ImageNodeSansHandle {
            color,
            path_to_image: image.path().unwrap().to_string(),
            texture_atlas,
            flip_x,
            flip_y,
            rect,
            image_mode,
        }
    }
}
impl Default for ImageNodeSansHandle {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            texture_atlas: None,
            path_to_image: "".to_owned(),
            flip_x: false,
            flip_y: false,
            rect: None,
            image_mode: NodeImageMode::Auto,
        }
    }
}
impl ImageNodeSansHandle {
    pub fn given_world(&self, world: &DeferredWorld) -> ImageNode {
        let assets = world.resource::<AssetServer>();
        let ImageNodeSansHandle {
            color,
            path_to_image,
            texture_atlas,
            flip_x,
            flip_y,
            rect,
            image_mode,
        } = self.clone();
        ImageNode {
            color,
            image: assets.load(path_to_image),
            texture_atlas,
            flip_x,
            flip_y,
            rect,
            image_mode,
        }
    }
    pub fn from_path(path: String) -> Self {
        Self {
            path_to_image: path,
            ..default()
        }
    }
}
fn image_node_sans_handle_added(mut world: DeferredWorld, context: HookContext) {
    let val = world
        .get::<ImageNodeSansHandle>(context.entity)
        .unwrap()
        .clone();
    let new = val.given_world(&world);
    let mut commands = world.commands();
    commands.entity(context.entity).insert(new);
    commands
        .entity(context.entity)
        .remove::<ImageNodeSansHandle>();
}

// Event traversal would force us to visit every entity in the middle -
// This still blasts a bunch of updates we don't need but at least it stays scoped
// to the component.
fn cascade_field_updates(
    src: On<RefreshInputFields>,
    world: DeferredWorld,
    mut commands: Commands,
) {
    let mut children = world
        .try_query::<&Children>()
        .expect("Query instantiation failed");
    let mut field_inputs = world
        .try_query::<(&ValueInputInput, &FieldAccessPath)>()
        .expect("Query instantiation failed");

    let mut queue = VecDeque::from([src.event_target()]);

    while !queue.is_empty() {
        let cursor = queue.pop_front().unwrap();
        if let Ok((input, fap)) = field_inputs.get(&world, cursor) {
            let path = fap.path.clone();

            let component_ref = match world.get_reflect(fap.owning_entity, fap.component_type_id) {
                Ok(f) => f,
                Err(e) => {
                    info!("{:?}", e);
                    return;
                }
            };
            let mut shadow = component_ref.reflect_clone().unwrap();
            let old_val = match path.reflect_element_mut(shadow.as_partial_reflect_mut()) {
                Ok(f) => f,
                Err(e) => {
                    info!("{:?}", e);
                    return;
                }
            };
            if !input.val.reflect_partial_eq(old_val).unwrap_or(false) {
                let shadow_val = old_val.reflect_clone().unwrap();
                commands
                    .entity(cursor)
                    .entry::<ValueInputInput>()
                    .and_modify(move |mut input_mut| {
                        input_mut.val = shadow_val;
                    });
            }
        };

        let Ok(kids) = children.get(&world, cursor).map(|x| x.iter()) else {
            continue;
        };
        for kid in kids {
            queue.push_back(kid);
        }
    }
}
