use std::{any::TypeId, marker::Send};

use bevy::{
    asset::ron::{self},
    ecs::{
        component::ComponentId, lifecycle::HookContext, reflect::ReflectCommandExt,
        relationship::Relationship, world::DeferredWorld,
    },
    image::{ImageFilterMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor},
    input_focus::tab_navigation::{TabGroup, TabNavigationPlugin},
    platform::collections::HashSet,
    prelude::*,
    reflect::{
        DynamicEnum, DynamicStruct, DynamicTuple, DynamicTupleStruct, DynamicVariant, Enum,
        EnumInfo, ParsedPath, ReflectKind, TypeInfo, TypeRegistration, VariantInfo, VariantType,
        serde::TypedReflectSerializer,
    },
    ui_widgets::{ScrollbarPlugin, observe},
};
use bevy_file_dialog::FileDialogPlugin;
use bevy_ui_text_input::TextInputPlugin;

use crate::{
    asset_tracking::ResourceHandles,
    editor_override_traits::*,
    widgets::{
        add_entity_button::add_entity_button,
        component_browser::{ComponentBrowserWidgetRoot, ComponentSelection},
        field_input::*,
        general::*,
        view_only_component::{RideAlongComponent, view_only_component},
    },
};

mod asset_tracking;
pub mod drag_snap;
mod editor_override_traits;
mod theme;
pub mod widgets;

pub struct ComponentEditorPlugin;
impl Plugin for ComponentEditorPlugin {
    fn build(&self, app: &mut App) {
        //TODO - it feels really weird to be initializing the text input plugin here -
        // but it's also not clear which sub module should own it.
        app.add_plugins((
            TextInputPlugin,
            TabNavigationPlugin,
            ScrollbarPlugin,
            FileDialogPlugin::default(),
        ));
        app.add_plugins((
            asset_tracking::plugin,
            drag_snap::plugin,
            editor_override_traits::plugin,
            widgets::plugin,
        ));
        app.init_state::<LoadingStatus>();
        app.init_resource::<AppTypeRegistry>();
        app.init_resource::<AssortedIcons>();
        app.add_systems(Startup, editor_initialization);
        app.add_observer(root);
        app.add_observer(component_ui_initializer);
        app.add_observer(on_update_event_dynamic);

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

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct AssortedIcons {
    #[dependency]
    pub trash: Handle<Image>,
    #[dependency]
    pub package_plus: Handle<Image>,
    #[dependency]
    pub list_plus: Handle<Image>,
    //TODO - we do not actually need a ducky here.
    #[dependency]
    pub ducky: Handle<Image>,
    #[dependency]
    pub move_icon: Handle<Image>,
    #[dependency]
    pub rotate_ccw: Handle<Image>,
    #[dependency]
    pub maximize_2: Handle<Image>,
    #[dependency]
    pub eye: Handle<Image>,
    #[dependency]
    pub move_3d: Handle<Image>,
    #[dependency]
    pub folder: Handle<Image>,
    #[dependency]
    pub square_pen: Handle<Image>,
    #[dependency]
    pub checker_board: Handle<Image>,
}
impl FromWorld for AssortedIcons {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            trash: assets.load("lucide/trash-2-white.png"),
            package_plus: assets.load("lucide/package-plus-white.png"),
            list_plus: assets.load("lucide/list-plus-white.png"),
            ducky: assets.load_with_settings(
                "images/ducky.png",
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
            move_icon: assets.load("lucide/move-white.png"),
            rotate_ccw: assets.load("lucide/rotate-ccw-white.png"),
            maximize_2: assets.load("lucide/maximize-2-white.png"),
            eye: assets.load("lucide/eye-white.png"),
            move_3d: assets.load("lucide/move-3d-white.png"),
            folder: assets.load("lucide/folder-white.png"),
            square_pen: assets.load("lucide/square-pen-white.png"),
            checker_board: assets.load_with_settings(
                "images/Kenny/Checkerboard/checkerboard.png",
                |settings: &mut ImageLoaderSettings| {
                    settings.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                        address_mode_u: bevy::image::ImageAddressMode::Repeat,
                        address_mode_v: bevy::image::ImageAddressMode::Repeat,
                        mag_filter: ImageFilterMode::Nearest,
                        min_filter: ImageFilterMode::Nearest,
                        mipmap_filter: ImageFilterMode::Nearest,
                        ..default()
                    })
                },
            ),
        }
    }
}

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
        children![(
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
        )],
    ));
}

//TODO - This should eventually represent some concept of displayed and actively edited serialization targets
// vs. implicit 'required' ride-along components
#[derive(Component, Clone)]
pub struct EntityUiRoot {
    pub component_holder: Entity,
    pub desired_component_set: HashSet<TypeId>,
    pub ride_along_components: HashSet<TypeId>,
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
            let Some(kids) = world.entity(ui).get_components::<&Children>() else {
                panic!("failed to initialize children for root");
            };
            for kid in kids.iter() {
                let Some(ride_along) = world.entity(kid).get_components::<&RideAlongComponent>()
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

pub(crate) fn root(source: On<ComponentSelection>, dworld: DeferredWorld, mut commands: Commands) {
    let window_root = source.event_target();

    let reg = dworld.resource::<AppTypeRegistry>();

    let mut component_ui_state = match dworld.entity(window_root).get::<EntityUiRoot>() {
        Some(x) => x.clone(),
        None => panic!("Uninitialized world entity during component UI creation"),
    };

    let type_id = source.base;
    let the_one_in_the_world = component_ui_state.component_holder;

    if component_ui_state.desired_component_set.insert(type_id) {
        if let Ok(init_component) = instantiate_or_die(&*reg, type_id, None) {
            commands
                .entity(the_one_in_the_world)
                .insert_reflect(init_component);
        } else {
            error!("Couldn't initialize selected component");
            return;
        }
    } else {
        return;
    }
    if component_ui_state.ride_along_components.remove(&type_id) {
        let Some(kids) = dworld.entity(window_root).get_components::<&Children>() else {
            panic!("failed to initialize children for root");
        };
        for kid in kids.iter() {
            let Some(ride_along) = dworld.entity(kid).get_components::<&RideAlongComponent>()
            else {
                continue;
            };

            if ride_along.0 == type_id {
                commands.entity(kid).despawn();
            }
        }
    }

    let reqs = dworld.required_components(type_id);
    let r = reg.read();

    for component_type_id in reqs {
        let registry = reg.read();
        let Some(root_type_info) = registry.get(component_type_id) else {
            info!("Failed to find registration for ID {:?}", component_type_id);
            continue;
        };
        let (was_in_ridealong_list, was_in_active_list) = (
            !component_ui_state
                .ride_along_components
                .insert(component_type_id),
            component_ui_state
                .desired_component_set
                .contains(&component_type_id),
        );
        let name = root_type_info.type_info().type_path_table().short_path();
        if was_in_active_list || was_in_ridealong_list {
            info!(
                "Component {:?} already exists on entity :[ was_active: {:?}, was_ride_along: {:?}]",
                name, was_in_active_list, was_in_ridealong_list
            );
            continue;
        }
        commands
            .entity(window_root)
            .with_child(view_only_component(name.to_owned(), component_type_id));
    }
    let registration = r.get(type_id).unwrap();

    commands.trigger(UiRequestedFor {
        component_ui_root: window_root,
        world_target: the_one_in_the_world,
        component_type_registration: registration.clone(),
    });
    commands.entity(window_root).insert(component_ui_state);
}

//TODO - At the very least we should work on some of the nomenclature here.
//
#[derive(EntityEvent)]
pub(crate) struct UiRequestedFor {
    #[event_target]
    pub component_ui_root: Entity,
    pub world_target: Entity,
    pub component_type_registration: TypeRegistration,
}

#[derive(EntityEvent)]
pub(crate) struct FieldUiRequestedFor {
    #[event_target]
    pub component_ui_root: Entity,
}

pub(crate) fn component_ui_initializer(
    source: On<UiRequestedFor>,
    world: DeferredWorld,

    mut commands: Commands,
) {
    let registry = world.resource::<AppTypeRegistry>().read();
    let registration = registry
        .get(source.component_type_registration.type_info().type_id())
        .unwrap();

    let maybe_header_trait = registration.data::<ReflectEditorHeaderUI>();

    let component_card = commands
        .spawn((
            Node {
                display: Display::Grid,
                border: UiRect::all(px(1.)),
                margin: UiRect::vertical(px(3.)),
                ..default()
            },
            BorderColor::all(Srgba::BLACK),
            CloseRoot,
            ComponentIdentifer(source.component_type_registration.clone()),
        ))
        .id();

    commands
        .entity(source.world_target)
        .add_one_related::<ComponentUiFor>(component_card);

    // Assumption -
    // The sibling immediately before the first RAC will be the end of the live components.
    // If no RACs exist, this will just put us at the end of the list, which is fine.
    //

    let mut kidx: usize = 0;
    let Some(kids) = world
        .entity(source.component_ui_root)
        .get_components::<&Children>()
    else {
        //TODO - handle this
        return;
    };
    for kid in kids.iter() {
        if let Some(_ride_along) = world.entity(kid).get_components::<&RideAlongComponent>() {
            break;
        } else {
            kidx = kidx + 1;
        }
    }

    commands
        .entity(source.component_ui_root)
        .insert_child(kidx, component_card);

    commands.entity(component_card).with_child((component_title(
        source
            .component_type_registration
            .type_info()
            .type_path_table()
            .short_path(),
    ),));

    let component = match world.get_reflect(
        source.world_target,
        source.component_type_registration.type_id(),
    ) {
        Ok(x) => x,
        Err(e) => {
            info!("{:?}", e);
            return;
        }
    };
    let component_ctxt = ComponentUiContext {
        world_target: source.world_target,
        component_type_registration: &source.component_type_registration,
        the_component: component.as_partial_reflect(),
        reg: world.resource::<AppTypeRegistry>(),
    };
    let step_context = ComponentUiStepContext {
        local_ui_focus: component_card,
        local_type_info: source.component_type_registration.type_info(),
        local_path: "".to_string(),
        local_name: "".to_string(),
    };
    let new_ui_ctx = UiCtxt::new(&component_ctxt, &step_context);
    if maybe_header_trait.is_some() {
        let my_dyn = maybe_header_trait
            .unwrap()
            .get(component.as_partial_reflect().try_as_reflect().unwrap())
            .unwrap();
        my_dyn.construct_header_ui(&new_ui_ctx, &mut commands);
    }

    new_ui_ctx.next(&mut commands);
}
pub struct UiCtxt<'a, 'b, 'w> {
    root_context: &'a ComponentUiContext<'a, 'b, 'w>,
    step_context: &'a ComponentUiStepContext<'a>,
}
impl<'a, 'b, 'w> UiCtxt<'a, 'b, 'w> {
    pub fn world_target(&self) -> Entity {
        self.root_context.world_target
    }
    pub fn component_instance(&self) -> &dyn PartialReflect {
        self.root_context.the_component
    }
    pub fn ui_anchor(&self) -> Entity {
        self.step_context.local_ui_focus
    }
    pub fn path(&self) -> &str {
        self.step_context.local_path.as_str()
    }
    pub fn type_name(&self) -> &str {
        self.step_context.local_name.as_str()
    }
    pub fn field_access_path(&self) -> FieldAccessPath {
        FieldAccessPath {
            path: ParsedPath::parse(self.step_context.local_path.as_str()).unwrap(),
            value_type_id: self.step_context.local_type_info.type_id(),
            component_type_id: self.root_context.component_type_registration.type_id(),
            owning_entity: self.root_context.world_target,
        }
    }
    pub fn next(&self, commands: &mut Commands) {
        self.root_context.step(self.step_context.clone(), commands);
    }
    pub fn new(
        root_context: &'a ComponentUiContext<'a, 'b, 'w>,
        step_context: &'a ComponentUiStepContext<'a>,
    ) -> Self {
        Self {
            root_context,
            step_context,
        }
    }
}
#[derive(Clone)]
pub struct ComponentUiStepContext<'a> {
    pub local_ui_focus: Entity,
    pub local_type_info: &'a TypeInfo,
    pub local_path: String,
    pub local_name: String,
}
#[derive(Clone)]
pub struct ComponentUiContext<'a, 'b, 'w> {
    pub world_target: Entity,
    pub component_type_registration: &'a TypeRegistration,
    pub the_component: &'b dyn PartialReflect,
    pub reg: &'w AppTypeRegistry,
}
impl<'a, 'b, 'w> ComponentUiContext<'a, 'b, 'w> {
    pub fn step(&self, step_context: ComponentUiStepContext, commands: &mut Commands) {
        let type_name = step_context.local_type_info.type_path_table().short_path();
        let registry = self.reg.read();
        let registration = registry
            .get(step_context.local_type_info.type_id())
            .unwrap();

        let maybe_field_level_trait = registration.data::<ReflectEditorFieldUI>();
        let maybe_struct_level_trait = registration.data::<ReflectEditorPerFieldUI>();

        if let Some(editor_trait) = maybe_field_level_trait {
            //TODO - Clean this up a bit.
            // probably push this off into leaf-level functions for these match arms, treat this as
            // another matched case - it essentially is, since we're overriding step sequence.
            let mut next_step = step_context.clone();
            let row = commands
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        padding: UiRect::all(px(1.)),
                        column_gap: px(2.),
                        ..default()
                    },
                    children![(
                        Name::new("Field Label"),
                        Text::new(step_context.local_name),
                        TextFont::from_font_size(10.0),
                    )],
                ))
                .id();
            commands.entity(step_context.local_ui_focus).add_child(row);
            let bucket = commands.spawn_empty().id();
            commands.entity(row).add_child(bucket);
            match step_context.local_path.reflect_element(self.the_component) {
                Ok(v) => {
                    let my_dyn = editor_trait.get(v.try_as_reflect().unwrap()).unwrap();

                    next_step.local_ui_focus = bucket;
                    my_dyn.construct_field_ui(&UiCtxt::new(&self, &next_step), commands);
                    commands.entity(bucket).insert(FieldAccessPath {
                        path: ParsedPath::parse(step_context.local_path.as_str()).unwrap(),
                        value_type_id: step_context.local_type_info.type_id(),
                        component_type_id: self.component_type_registration.type_id(),
                        owning_entity: self.world_target,
                    });
                    commands.trigger(FieldUiRequestedFor {
                        component_ui_root: bucket,
                    });
                    return;
                }
                Err(_) => todo!(),
            }
        }

        match step_context.local_type_info {
            TypeInfo::Struct(struct_info) => {
                if step_context.local_name != "" {
                    commands
                        .entity(step_context.local_ui_focus)
                        .with_child(field_name_with_type(step_context.local_name, type_name));
                }

                let field_layout = commands.spawn(field_layout()).id();

                commands
                    .entity(step_context.local_ui_focus)
                    .add_child(field_layout);

                for &field_name in struct_info.field_names() {
                    let field_type_info = struct_info
                        .field(field_name)
                        .and_then(|t| t.type_info())
                        .unwrap();

                    let property_path = if step_context.local_path != "" {
                        format!("{}.{field_name}", step_context.local_path)
                    } else {
                        field_name.to_owned()
                    };

                    let next_step = ComponentUiStepContext {
                        local_ui_focus: field_layout,
                        local_type_info: field_type_info,
                        local_path: property_path,
                        local_name: field_name.to_string(),
                    };
                    if maybe_struct_level_trait.is_some() {
                        let my_dyn = maybe_struct_level_trait
                            .unwrap()
                            .get(self.the_component.try_as_reflect().unwrap())
                            .unwrap();
                        my_dyn.construct_per_field_ui(&UiCtxt::new(&self, &next_step), commands);
                    } else {
                        self.step(next_step.clone(), commands);
                    }
                }
            }
            TypeInfo::TupleStruct(ts_info) => {
                if step_context.local_name != "" {
                    commands
                        .entity(step_context.local_ui_focus)
                        .with_child(field_name_with_type(step_context.local_name, type_name));
                }

                let field_layout = commands.spawn(field_layout()).id();

                commands
                    .entity(step_context.local_ui_focus)
                    .add_child(field_layout);

                for unnamed in ts_info.iter() {
                    let field_name = unnamed.index().to_string();
                    let field_type_info = ts_info
                        .field_at(unnamed.index())
                        .and_then(|t| t.type_info())
                        .unwrap();

                    let property_path = if step_context.local_path != "" {
                        format!("{}.{field_name}", step_context.local_path)
                    } else {
                        field_name.to_owned()
                    };
                    let next_step = ComponentUiStepContext {
                        local_ui_focus: field_layout,
                        local_type_info: field_type_info,
                        local_path: property_path,
                        local_name: field_name,
                    };
                    if maybe_struct_level_trait.is_some() {
                        let my_dyn = maybe_struct_level_trait
                            .unwrap()
                            .get(self.the_component.try_as_reflect().unwrap())
                            .unwrap();
                        my_dyn.construct_per_field_ui(&UiCtxt::new(&self, &next_step), commands);
                    } else {
                        self.step(next_step.clone(), commands);
                    }
                }
            }
            TypeInfo::Enum(enum_info) => {
                let num_variants: usize = enum_info.iter().len();

                commands
                    .entity(step_context.local_ui_focus)
                    .with_child(field_name_with_type(step_context.local_name, type_name));

                //TODO - don't barf here.
                let path = ParsedPath::parse(step_context.local_path.as_str()).unwrap();

                //This feels kind of goofy - it feels like there should be some way to cast this as a dynamic enum.

                let enum_variant_index = path
                    .reflect_element(self.the_component)
                    .map_err(|_| 0)
                    .and_then(|we| {
                        let mut dummy = DynamicEnum::default();
                        dummy.apply(&*we);
                        Ok(dummy.variant_index())
                    })
                    .map_err(|_| 0)
                    .unwrap_or(0);

                let structured_variant_layout_container = commands
                    .spawn(Node {
                        display: Display::None,
                        ..default()
                    })
                    .id();
                let enum_metadata = EnumMetadata::new(
                    enum_info.clone(),
                    enum_variant_index,
                    step_context.local_path.to_owned(),
                    structured_variant_layout_container,
                );
                let cap = FieldAccessPath {
                    path,
                    value_type_id: enum_info.type_id(),
                    component_type_id: self.component_type_registration.type_id(),
                    owning_entity: self.world_target.clone(),
                };

                let radio_group_container = commands
                    .spawn((
                        Name::new(type_name),
                        //Ok look I know this is unsafe but seriously if you have more than u16::MAX_SIZE
                        // variants in your enum I think you need to reconsider some shit.
                        radio_group_container(num_variants.try_into().unwrap()),
                        enum_metadata.clone(),
                        cap.clone(),
                        BorderColor::all(Srgba::GREEN),
                        observe(enum_radio_observer),
                        observe(enum_subelement_observer),
                    ))
                    .id();

                commands
                    .entity(step_context.local_ui_focus)
                    .add_child(radio_group_container)
                    .add_child(structured_variant_layout_container);

                let mut idx: usize = 0;
                for variant in enum_info.iter() {
                    commands
                        .entity(radio_group_container)
                        .with_child(radio_group_button(enum_variant_index, idx, variant));

                    idx = idx + 1;
                }
                handle_enum_variant(
                    &self,
                    enum_metadata,
                    enum_variant_index,
                    structured_variant_layout_container,
                    commands,
                );
            }
            TypeInfo::Opaque(o_info) => {
                let row = commands
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            padding: UiRect::all(px(1.)),
                            column_gap: px(2.),
                            ..default()
                        },
                        children![(
                            Name::new("Field Label"),
                            Text::new(step_context.local_name),
                            TextFont::from_font_size(10.0),
                        )],
                    ))
                    .id();
                commands.entity(step_context.local_ui_focus).add_child(row);

                //TODO - We should restructure this to be less spah-get

                match field_is_parseable(&step_context.local_type_info.type_id(), &*self.reg)
                    .and_then(|_| {
                        ParsedPath::parse(step_context.local_path.as_str())
                            .map_err(|e| e.to_string())
                    })
                    .and_then(|path| {
                        Ok(FieldAccessPath {
                            owning_entity: self.world_target.clone(),
                            component_type_id: self.component_type_registration.type_id(),
                            path: path,
                            value_type_id: step_context.local_type_info.type_id(),
                        })
                    })
                    .and_then(|cap| {
                        cap.path
                            .reflect_element(&*self.the_component)
                            .map_err(|e| e.to_string())
                            .and_then(|we| {
                                let read_lock = self.reg.read();
                                let serializer = TypedReflectSerializer::new(&*we, &*read_lock);
                                ron::to_string(&serializer).map_err(|_| "".to_string())
                            })
                            .map(|txt| value_input_field(txt, cap))
                    }) {
                    Ok(f) => commands.entity(row).with_child(f),
                    Err(t) => commands.entity(row).with_child(input_field_error(t)),
                };

                commands.entity(row).with_child((
                    Node { ..default() },
                    Text::new(o_info.type_path_table().short_path()),
                    TextFont::from_font_size(10.0),
                ));
                commands.trigger(FieldUiRequestedFor {
                    component_ui_root: step_context.local_ui_focus,
                });
            }
            //these two are esentially the same case
            TypeInfo::Set(_info) => {
                info!("View entity [Set]: {:?}", step_context.local_ui_focus);
            }
            TypeInfo::List(_info) => {
                info!("View entity [Li]: {:?}", step_context.local_ui_focus);
            }
            //these two are esentially the same case
            TypeInfo::Tuple(_info) => {
                info!("View entity [Tu]: {:?}", step_context.local_ui_focus);
            }
            TypeInfo::Array(_info) => {
                info!("View entity [Arr]: {:?}", step_context.local_ui_focus);
            }

            //This seems rare enough as a case that maybe we can leave it til later?
            // Not looking forward to designing the UI for this.
            TypeInfo::Map(_info) => {
                info!("View entity [Mp]: {:?}", step_context.local_ui_focus);
            }
            //We explicitly allow this because we want this default to be resistant to changes to this type/this code.
            #[allow(unreachable_patterns)]
            _ => {
                info!("View entity [Wtf]: {:?}", step_context.local_ui_focus);
            }
        };
    }
}

fn radio_group_container(num_variants: u16) -> impl Bundle {
    Node {
        display: Display::Grid,
        justify_self: JustifySelf::Center,
        grid_template_columns: vec![RepeatedGridTrack::fr(num_variants.min(4), 0.5)],

        column_gap: px(0.),
        min_width: Val::Percent(10.),
        max_width: Val::Percent(100.),
        border: UiRect::all(px(2.)),
        margin: UiRect::all(px(1.)),
        ..default()
    }
}

fn radio_group_button(
    selected_index: usize,
    current_index: usize,
    variant_info: &VariantInfo,
) -> impl Bundle {
    (
        Node {
            border: UiRect::all(px(1.)),
            margin: UiRect::all(px(0.)),
            padding: UiRect::vertical(px(1.)),
            display: Display::Grid,
            grid_column: GridPlacement::auto(),
            min_width: percent(10.),
            width: percent(100.),
            justify_content: JustifyContent::Center,
            justify_self: JustifySelf::Center,
            ..default()
        },
        (if selected_index == current_index {
            BackgroundColor(Color::from(Srgba::BLUE))
        } else {
            BackgroundColor(Color::NONE)
        }),
        SelectionIndex(current_index),
        BorderColor::all(Srgba::RED),
        observe(trigger_radio_selection),
        children![(Text::new(variant_info.name()), TextFont::from_font_size(8.),)],
    )
}

//TODO - We should find a nicer way to handle styling internally to this radio selection shit.
// Probably take a look at feathers etc.
fn trigger_radio_selection(
    source: On<Pointer<Click>>,
    mut q: Query<(&mut BackgroundColor, &ChildOf), With<Node>>,
    children: Query<&Children>,
    mut commands: Commands,
) {
    if let Ok((mut bgc, dad)) = q.get_mut(source.event_target()) {
        *bgc = BackgroundColor::from(Srgba::BLUE);
        for bro in children.get(dad.0).unwrap() {
            if *bro != source.event_target()
                && let Ok((mut no, _)) = q.get_mut(*bro)
            {
                *no = BackgroundColor::from(Color::NONE);
            }
        }
    }
    commands.trigger(RadioGroupSelection {
        entity: source.event_target(),
    })
}
//Here we try our best to handle presentation and world mutation separately.
// This lets us delegate the somewhat nasty and convoluted dynamic ref mutation to one spot,
// but also let's us re-use as much of this as possible and provide kind of a nice model for
// how the event contract is supposed to work.
fn enum_radio_observer(
    event: On<RadioGroupSelection>,
    appreg: Res<AppTypeRegistry>,
    q: Query<(&EnumMetadata, &FieldAccessPath), Without<SelectionIndex>>,
    q2: Query<&SelectionIndex>,
    mut commands: Commands,
) {
    let (metadata, cap) = match q.get(event.event_target()) {
        Ok(f) => f,
        Err(e) => {
            info!("{:?}", e);
            return;
        }
    };
    let SelectionIndex(s_index) = match q2.get(event.original_event_target()) {
        Ok(f) => f,
        Err(e) => {
            info!("{:?}", e);
            return;
        }
    };

    if metadata.current_vidx != *s_index {
        let mut newdata = metadata.clone();
        newdata.current_vidx = *s_index;

        commands
            .entity(event.event_target())
            .insert(newdata.clone());
        commands
            .entity(metadata.associated_layout)
            .despawn_children();

        //TODO - this makes extra work if we're shifting to a unit enum, we should probably short-circuit it.

        let _ = instantiate_or_die(&*appreg, newdata.info.type_id(), Some(*s_index))
            .map_err(String::from)
            .and_then(|n| {
                let triggered_event = DynamicComponentUiUpdateEvent {
                    ui_entity: event.event_target(),
                    new_value: n,
                    component_ui_field_for: cap.clone(),
                };
                triggered_event.validate()?;
                Ok(commands.trigger(triggered_event))
            });
    }
}
fn enum_subelement_observer(
    source: On<FieldUiRequestedFor>,
    mut commands: Commands,
    world: DeferredWorld,
) {
    let cap = match world
        .entity(source.event_target())
        .get_components::<&FieldAccessPath>()
    {
        Some(f) => f,
        None => {
            info!("Failed to locate ComponentFieldUiFor");
            return;
        }
    };
    let the_component = match world.get_reflect(cap.owning_entity, cap.component_type_id) {
        Ok(x) => x.as_partial_reflect(),
        Err(e) => {
            info!("{:?}", e);
            return;
        }
    };
    let newdata = match world
        .entity(source.event_target())
        .get_components::<&EnumMetadata>()
    {
        Some(f) => f,
        None => {
            info!("Failed to locate EnumMetadata");
            return;
        }
    };

    let selection = newdata.current_vidx;
    let reg = world.resource::<AppTypeRegistry>();
    let registration = match reg.read().get(cap.component_type_id) {
        Some(f) => f.clone(),
        None => {
            info!("Failed to find type registration");
            return;
        }
    };

    handle_enum_variant(
        &ComponentUiContext {
            world_target: cap.owning_entity,
            component_type_registration: &registration,
            the_component: the_component,
            reg: reg,
        },
        newdata.clone(),
        selection,
        newdata.associated_layout,
        &mut commands,
    );
}

fn component_title(name: impl Into<String>) -> impl Bundle {
    (
        Node::default(),
        children![
            (
                Text::new(name),
                TextFont {
                    font_size: 16.,
                    ..Default::default()
                }
            ),
            (
                Name::new("Remove Component"),
                Node {
                    width: px(12.),
                    height: px(12.),
                    ..default()
                },
                ImageNodeSansHandle {
                    path_to_image: "lucide/trash-2-white.png".to_owned(),
                    color: Color::from(Srgba::RED),
                    ..default()
                },
                observe(|src: On<Pointer<Click>>, mut commands: Commands| {
                    commands.trigger(CloseEvent::new(src.event_target()));
                })
            )
        ],
    )
}
fn field_layout() -> impl Bundle {
    (Node {
        display: Display::Grid,
        border: UiRect::all(px(1.)),
        padding: UiRect::vertical(px(2.)),
        row_gap: px(2.0),
        ..default()
    },)
}

fn handle_enum_variant(
    ui_context: &ComponentUiContext,
    newdata: EnumMetadata,
    s_index: usize,
    structured_variant_layout_container: Entity,
    commands: &mut Commands,
) {
    let v_info = newdata.info.variant_at(s_index).unwrap();
    match v_info.variant_type() {
        VariantType::Struct => {
            commands
                .entity(structured_variant_layout_container)
                .insert(Node {
                    min_width: percent(10.),
                    ..default()
                })
                .insert(BackgroundColor::from(Srgba::BLACK));
            let v_struct_info = v_info.as_struct_variant().unwrap();
            commands
                .entity(structured_variant_layout_container)
                .with_child(field_name_with_type(v_info.name(), ""));

            let field_layout = commands
                .spawn((Node {
                    display: Display::Grid,
                    row_gap: px(2.0),
                    ..default()
                },))
                .id();
            commands
                .entity(structured_variant_layout_container)
                .add_child(field_layout);
            for &field_name in v_struct_info.field_names() {
                let field_type_info = v_struct_info
                    .field(field_name)
                    .and_then(|t| t.type_info())
                    .unwrap();
                ui_context.step(
                    ComponentUiStepContext {
                        local_ui_focus: field_layout,
                        local_type_info: field_type_info,
                        local_path: format!("{}.{field_name}", newdata.path_string),
                        local_name: field_name.to_string(),
                    },
                    commands,
                );
            }
        }
        VariantType::Tuple => {
            let v_tuple_info = v_info.as_tuple_variant().unwrap();
            commands
                .entity(structured_variant_layout_container)
                .insert(Node {
                    min_width: percent(10.),
                    ..default()
                });
            commands
                .entity(structured_variant_layout_container)
                .with_child(field_name_with_type(v_info.name(), ""));

            let field_layout = commands
                .spawn((Node {
                    display: Display::Grid,
                    row_gap: px(2.0),
                    ..default()
                },))
                .id();
            commands
                .entity(structured_variant_layout_container)
                .add_child(field_layout);
            for unnamed in v_tuple_info.iter() {
                let field_name = unnamed.index().to_string();
                let field_type_info = v_tuple_info
                    .field_at(unnamed.index())
                    .and_then(|t| t.type_info())
                    .unwrap();
                ui_context.step(
                    ComponentUiStepContext {
                        local_ui_focus: field_layout,
                        local_type_info: field_type_info,
                        local_path: format!("{}.{field_name}", newdata.path_string),
                        local_name: field_name.to_string(),
                    },
                    commands,
                );
            }
        }
        VariantType::Unit => {
            commands
                .entity(structured_variant_layout_container)
                .insert(Node {
                    display: Display::None,
                    ..default()
                });
        }
    };
}

fn field_name_with_type(
    field_name: impl Into<String>,
    type_name: impl Into<String>,
) -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Row,
            min_width: Val::Percent(10.0),
            ..default()
        },
        children![
            (
                Node::default(),
                children![(Text::new(field_name), TextFont::from_font_size(9.),)]
            ),
            Node {
                min_width: percent(10.),
                ..default()
            },
            (
                Node::default(),
                children![(
                    Text::new(type_name),
                    TextFont::from_font_size(10.),
                    BackgroundColor::from(Srgba::BLUE),
                ),]
            ),
        ],
    )
}

fn field_is_parseable(value_id: &TypeId, reg: &AppTypeRegistry) -> Result<(), String> {
    let registry = reg.read();
    let Some(registration) = registry.get(*value_id) else {
        return Err("Unregistered Type".to_string());
    };
    if registration.data::<ReflectDeserialize>().is_none() {
        return Err("No Deserializer for Type".to_string());
    }
    if registration.type_info().kind() != ReflectKind::Opaque {
        return Err("NonScalar values unsupported at this time".to_string());
    }

    Ok(())
}

#[derive(Component, Clone)]
pub struct EnumMetadata {
    pub info: EnumInfo,
    pub current_vidx: usize,
    pub path_string: String,
    pub associated_layout: Entity,
    //pub cap: ComponentFieldUiFor,
}
impl EnumMetadata {
    fn new(
        info: EnumInfo,
        current_vidx: usize,
        path_string: String,
        associated_layout: Entity,
        //cap: ComponentFieldUiFor,
    ) -> Self {
        EnumMetadata {
            info,
            current_vidx,
            path_string,
            associated_layout,
            //cap,
        }
    }
}

#[derive(Component)]
struct SelectionIndex(pub usize);

#[derive(Component, Clone)]
pub struct FieldAccessPath {
    pub path: ParsedPath,
    pub value_type_id: TypeId,
    pub component_type_id: TypeId,
    pub owning_entity: Entity,
}
#[derive(Component)]
pub struct SelectedEntityUiRoot;
#[derive(Component)]
pub struct ComponentIdentifer(pub TypeRegistration);
#[derive(EntityEvent)]
#[entity_event(propagate, auto_propagate)]
pub struct RadioGroupSelection {
    entity: Entity,
}

#[derive(EntityEvent)]
pub struct DynamicComponentUiUpdateEvent {
    #[event_target]
    ui_entity: Entity,
    new_value: Box<dyn PartialReflect>,
    component_ui_field_for: FieldAccessPath,
}
impl DynamicComponentUiUpdateEvent {
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
    source: On<DynamicComponentUiUpdateEvent>,
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

    commands.trigger(FieldUiRequestedFor {
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
