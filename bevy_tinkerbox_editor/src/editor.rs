//
use std::{any::TypeId, marker::Send};

use bevy::{
    asset::ron::{self, Deserializer},
    ecs::{
        lifecycle::HookContext,
        reflect::ReflectCommandExt,
        relationship::{RelatedSpawner, Relationship},
        spawn::SpawnWith,
        world::DeferredWorld,
    },
    input::keyboard::KeyboardInput,
    input_focus::{
        FocusedInput, InputFocus,
        tab_navigation::{TabGroup, TabNavigationPlugin},
    },
    platform::collections::HashSet,
    prelude::*,
    reflect::{
        DynamicEnum, DynamicStruct, DynamicTuple, DynamicTupleStruct, DynamicVariant, Enum,
        EnumInfo, ParsedPath, ReflectKind, TypeInfo, TypeRegistration, TypeRegistry, VariantInfo,
        VariantType,
        serde::{ReflectDeserializer, TypedReflectSerializer},
    },
    ui_widgets::{ScrollbarPlugin, observe},
};
use bevy_ui_text_input::{
    TextInputBuffer, TextInputMode, TextInputNode, TextInputPlugin, TextInputStyle,
};
use cosmic_text::Edit;
use serde::de::DeserializeSeed;

use crate::{
    theme::colors,
    widget_functions::{HoverBackground, centered, filter_with_prompt, scroll_area_demo, text_row},
};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((TextInputPlugin, TabNavigationPlugin, ScrollbarPlugin));
    app.init_resource::<AppTypeRegistry>();

    app.add_systems(
        PreStartup,
        (manually_registering_trait_data_for_fun_and_profit,).chain(),
    );
    app.add_observer(root);
    app.add_observer(component_ui_initializer);
    app.add_observer(on_update_event_dynamic);
    app.add_systems(
        Update,
        (hide_filtered_components, transform_editor_presentation),
    );
}

pub fn spawn_editor(
    mut commands: Commands,
    reg: Res<AppTypeRegistry>,
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
        UiTargetCamera(q.single().unwrap()),
        TabGroup::default(),
        //DespawnOnExit(ActiveEditor),
        Children::spawn((Spawn((
            Node {
                flex_direction: FlexDirection::Row,
                ..default()
            },
            Children::spawn((
                Spawn(scroll_area_demo(SpawnWith(
                    |p: &mut RelatedSpawner<'_, ChildOf>| {
                        p.spawn((
                            Node {
                                flex_direction: FlexDirection::Column,
                                ..default()
                            },
                            Children::spawn(Spawn((
                                Node {
                                    display: Display::Grid,
                                    ..default()
                                },
                                SelectedEntityUiRoot,
                            ))),
                        ));
                    },
                ))),
                Spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: px(3),
                        ..default()
                    },
                    Children::spawn((
                        Spawn((
                            Node {
                                flex_direction: FlexDirection::Column,
                                row_gap: px(3),
                                ..default()
                            },
                            BackgroundColor(colors::gry_nut().into()),
                            BorderRadius::all(px(3)),
                            Children::spawn((
                                Spawn(centered((
                                    Text::new("Add Components"),
                                    TextFont {
                                        font_size: 11.0,
                                        ..default()
                                    },
                                ))),
                                Spawn((
                                    Node {
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    },
                                    Children::spawn(Spawn(filter_with_prompt(
                                        "Filter",
                                        SceneEditorComponentFilter,
                                    ))),
                                )),
                                Spawn(scroll_area_demo(spawn_component_entries(reg.clone()))),
                            )),
                        )),
                        SpawnWith(|parent: &mut RelatedSpawner<'_, ChildOf>| {
                            let me = parent
                                .spawn((
                                    Node {
                                        width: px(250),
                                        align_content: AlignContent::Center,
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    },
                                    children![],
                                ))
                                .id();

                            parent.spawn(Observer::new(
                                move |event: On<ComponentPreSelection>,
                                      mut commands: Commands,
                                      world: DeferredWorld| {
                                    let reg = world.resource::<AppTypeRegistry>();
                                    let preview =
                                        component_preview(me, event.type_of, &reg, &world);
                                    match preview {
                                        Some(selection) => {
                                            let kid = commands
                                                .spawn(component_selection_widget(selection))
                                                .id();
                                            commands.entity(me).despawn_children().add_child(kid);
                                        }
                                        _ => {
                                            commands.entity(me).despawn_children();
                                        }
                                    };
                                },
                            ));
                        }),
                    )),
                )),
            )),
        )),)),
    ));
}

fn hide_filtered_components(
    filter_text_buffer: Query<&TextInputBuffer, With<SceneEditorComponentFilter>>,
    mut component_node_list: Query<(&mut Node, &Name), With<ComponentSubject>>,
) {
    let Ok(text) = filter_text_buffer.single().map(|b| b.get_text()) else {
        return;
    };

    for (mut node, name) in component_node_list.iter_mut() {
        node.display = if name.as_str().contains(text.as_str()) {
            Display::Flex
        } else {
            Display::None
        }
    }
}

fn spawn_component_entries(
    registrations: AppTypeRegistry,
) -> SpawnWith<impl FnOnce(&mut RelatedSpawner<ChildOf>)> {
    SpawnWith(move |parent: &mut RelatedSpawner<ChildOf>| {
        let lock = registrations.read();
        for entry in lock.iter() {
            if entry.data::<ReflectComponent>().is_none() {
                continue;
            }
            // Indicate if we can actually construct a dynamic component to edit:
            let (resting_color, warning): (Color, &'static str) =
                if entry.data::<ReflectDefault>().is_none() {
                    (
                        Srgba::new(0.2, 0.15, 0.15, 1.0).into(),
                        "no ReflectDefault impl",
                    )
                } else {
                    (colors::gry_nut().into(), "")
                };
            let name = format!(
                "{} | {}",
                entry.type_info().type_path_table().short_path(),
                warning
            );

            parent.spawn((
                Node {
                    column_gap: px(2),
                    ..default()
                },
                Name::new(name.clone()),
                ComponentSubject(entry.type_id()),
                Outline::new(Val::Px(2.), Val::ZERO, Color::NONE),
                BackgroundColor(resting_color),
                HoverBackground {
                    out: resting_color,
                    over: resting_color.lighter(0.025),
                },
                observe(
                    |source: On<Pointer<Click>>, world: DeferredWorld, mut commands: Commands| {
                        if let Some(t_id) = world
                            .entity(source.event_target())
                            .get::<ComponentSubject>()
                        {
                            commands.trigger(ComponentPreSelection { type_of: t_id.0 });
                        }
                    },
                ),
                children![
                    (
                        Node {
                            width: px(12),
                            ..default()
                        },
                        BackgroundColor(Color::WHITE)
                    ),
                    centered(text_row(name.as_str())),
                    SceneEditorComponentFilter,
                ],
            ));
        }
    })
}

#[derive(Component)]
pub struct ComponentSubject(pub TypeId);
#[derive(Component)]
pub struct SceneEditorComponentFilter;

#[derive(Event)]
struct ComponentPreSelection {
    type_of: TypeId,
}

#[derive(Component, Clone, EntityEvent)]
pub struct ComponentSelection {
    pub entity: Entity,
    pub base: (TypeId, &'static str),
    pub required: Vec<(TypeId, &'static str)>,
}

fn component_preview(
    entity: Entity,
    id: TypeId,
    reg: &AppTypeRegistry,
    world: &DeferredWorld,
) -> Option<ComponentSelection> {
    let registry = reg.read();
    let Some(type_registration) = registry.get(id) else {
        return None;
    };
    let name = type_registration.type_info().type_path_table().short_path();
    let mut required: Vec<(TypeId, &'static str)> = Vec::new();
    for val in world.required_components(id) {
        let v_name = registry
            .get(val)
            .map(|e| e.type_info().type_path_table().short_path())
            .unwrap_or("");
        required.push((val.clone(), v_name.into()))
    }

    Some(ComponentSelection {
        entity,
        base: (id, name.into()),
        required: required,
    })
}
fn component_selection_widget(selection: ComponentSelection) -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Column,
            border: UiRect::all(px(10)),
            ..default()
        },
        //BackgroundColor(Srgba::BLUE.into()),
        BorderColor::all(Srgba::RED),
        selection.clone(),
        children![
            (
                centered(text_row(selection.base.1)),
                selection.clone(),
                observe(
                    |src: On<Pointer<Click>>, world: DeferredWorld, mut commands: Commands| {
                        if let Some(in_a_bottle) =
                            world.entity(src.event_target()).get::<ComponentSelection>()
                        {
                            commands.trigger(in_a_bottle.clone());
                        }
                    }
                )
            ),
            (
                Node {
                    display: Display::Grid,
                    grid_template_columns: vec![
                        RepeatedGridTrack::min_content(1),
                        RepeatedGridTrack::px(1, 12.)
                    ],
                    column_gap: px(2),

                    ..default()
                },
                Children::spawn((
                    Spawn((
                        Node {
                            grid_column: GridPlacement::span(2),
                            grid_row: GridPlacement::start(1),
                            ..default()
                        },
                        text_row(format!("Required: {:?}", selection.required.len()).as_str()),
                    )),
                    SpawnWith(move |p: &mut RelatedSpawner<'_, ChildOf>| {
                        let mut inc = 2;
                        for (_, kid_name) in selection.required.iter() {
                            p.spawn((
                                Node {
                                    display: Display::Grid,
                                    grid_column: GridPlacement::start(1),
                                    grid_row: GridPlacement::start(inc),
                                    ..default()
                                },
                                text_row(kid_name),
                            ));
                            p.spawn((
                                Node {
                                    display: Display::Grid,
                                    grid_column: GridPlacement::start(2),
                                    grid_row: GridPlacement::start(inc),
                                    ..default()
                                },
                                text_row("✅"),
                            ));
                            inc = inc + 1;
                        }
                    })
                ))
            )
        ],
    )
}

//TODO - This should eventually represent some concept of displayed and actively edited serialization targets
// vs. implicit 'required' ride-along components
#[derive(Component, Clone)]
pub struct EntityUiRoot {
    pub component_holder: Entity,
    pub desired_component_set: HashSet<TypeId>,
}

#[derive(Component, Clone)]
#[relationship(relationship_target = ComponentUisFor)]
pub struct ComponentUiFor {
    #[relationship]
    pub target: Entity,
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

pub(crate) fn root(
    source: On<ComponentSelection>,
    mut dworld: DeferredWorld,
    mut commands: Commands,
) {
    let mut q = dworld
        .try_query_filtered::<Entity, With<SelectedEntityUiRoot>>()
        .unwrap();
    let window_root = dworld.query(&mut q).single().unwrap();

    let reg = dworld.resource::<AppTypeRegistry>();

    // We're potentially going to make a bunch of edits here so lets get a clone of our component to work with.
    //
    let mut component_ui_state = match dworld.entity(window_root).get::<EntityUiRoot>() {
        Some(x) => x.clone(),
        None => EntityUiRoot {
            component_holder: commands.spawn_empty().id(),
            desired_component_set: HashSet::new(),
        },
    };

    let type_id = source.base.0;
    let the_one_in_the_world = component_ui_state.component_holder;

    if !component_ui_state.desired_component_set.contains(&type_id) {
        if let Ok(init_component) = instantiate_or_die(&*reg, type_id, None) {
            commands
                .entity(the_one_in_the_world)
                .insert_reflect(init_component);
        } else {
            error!("Couldn't initialize selected component");
            return;
        }
    }

    let mut type_ids: Vec<TypeId> = Vec::new();
    type_ids.push(type_id);
    let reqs = dworld.required_components(type_id);
    let r = reg.read();

    for component_type_id in vec![type_ids, reqs].iter().flatten() {
        let registry = reg.read();
        let Some(_root_type_info) = registry.get(*component_type_id) else {
            info!("Failed to find registration for ID {:?}", component_type_id);
            continue;
        };
        if !component_ui_state
            .desired_component_set
            .insert(*component_type_id)
        {
            info!(
                "Component ID {:?} already exists on entity",
                component_type_id
            );
            continue;
        }
        let registration = r.get(*component_type_id).unwrap();

        // info!("Type info! {:?}", root_type_info.type_info());
        commands.trigger(UiRequestedFor {
            component_ui_root: window_root,
            world_target: the_one_in_the_world,
            component_type_registration: registration.clone(),
        });
    }
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
            ComponentUiFor {
                target: source.world_target,
            },
        ))
        .id();
    commands
        .entity(source.component_ui_root)
        .add_child(component_card);
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
    if maybe_header_trait.is_some() {
        let my_dyn = maybe_header_trait
            .unwrap()
            .get(component.as_partial_reflect().try_as_reflect().unwrap())
            .unwrap();
        my_dyn.construct_header_ui(component_card, &mut commands);
    }

    (ComponentUiContext {
        world_target: source.world_target,
        component_type_registration: &source.component_type_registration,
        the_component: component.as_partial_reflect(),
        reg: world.resource::<AppTypeRegistry>(),
    })
    .step(
        ComponentUiStepContext {
            local_ui_focus: component_card,
            local_type_info: source.component_type_registration.type_info(),
            local_path: "".to_string(),
            local_name: "".to_string(),
        },
        &mut commands,
    );
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

                if let Some(editor_trait) = maybe_field_level_trait {
                    let bucket = commands.spawn_empty().id();
                    commands.entity(row).add_child(bucket);
                    match step_context.local_path.reflect_element(self.the_component) {
                        Ok(v) => {
                            let my_dyn = editor_trait.get(v.try_as_reflect().unwrap()).unwrap();
                            my_dyn.construct_field_ui(bucket, commands);
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
                } else {
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
                        Err(t) => commands.entity(row).with_child(bullshit_error(t)),
                    };
                }

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
            Text::new(name),
            TextFont {
                font_size: 16.,
                ..Default::default()
            }
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

fn bullshit_error(content: impl Into<String>) -> impl Bundle {
    (
        Node {
            max_width: percent(30.0),
            overflow: Overflow {
                x: OverflowAxis::Clip,
                y: OverflowAxis::Clip,
            },
            ..default()
        },
        //BackgroundColor(Color::Srgba(Srgba::RED)),
        Text::new(content),
        TextFont::from_font_size(9.),
        TextColor(Srgba::RED.into()),
    )
}

pub fn value_input_field<T: Component>(starting_value: String, marker: T) -> impl Bundle {
    let mut buffer = TextInputBuffer::default();
    buffer.editor.insert_string(starting_value.as_str(), None);
    (
        TextInputNode {
            mode: TextInputMode::SingleLine,
            clear_on_submit: false,
            unfocus_on_submit: false,
            ..default()
        },
        buffer,
        BackgroundColor(Srgba::hex("#46474d").unwrap_or(Srgba::WHITE).into()),
        TextFont::from_font_size(11.0),
        TextInputStyle { ..default() },
        Node {
            height: px(18),
            width: px(60),
            ..default()
        },
        take_focus_on_click(),
        handle_keyboard_interaction_for_input_field(),
        marker,
    )
}

fn take_focus_on_click() -> impl Bundle {
    observe(|event: On<Pointer<Click>>, mut focus: ResMut<InputFocus>| {
        focus.set(event.entity);
    })
}
fn handle_keyboard_interaction_for_input_field() -> impl Bundle {
    observe(
        |event: On<FocusedInput<KeyboardInput>>,
         mut focus: ResMut<InputFocus>,
         reg: Res<AppTypeRegistry>,
         q: Query<(&FieldAccessPath, &TextInputBuffer)>,
         mut commands: Commands| {
            match event.input.logical_key {
                bevy::input::keyboard::Key::Escape => {
                    focus.clear();
                }
                _ => {}
            };

            if let Ok((cap, tb)) = q.get(event.focused_entity) {
                let real_registry = reg.read();
                let mut registry = TypeRegistry::new();
                let value_registration = real_registry.get(cap.value_type_id).unwrap();
                registry.add_registration(value_registration.clone());

                let text = tb.get_text();

                let prop_path = cap.path.to_string();
                let val_type_name = value_registration
                    .type_info()
                    .type_path_table()
                    .short_path();
                let component_type_name = real_registry
                    .get(cap.component_type_id)
                    .unwrap()
                    .type_info()
                    .type_path_table()
                    .short_path();
                let deserializer_hint = format!(
                    "{{\"{}\":{}}}",
                    value_registration.type_info().type_path(),
                    text
                );
                let reflect_deserializer = ReflectDeserializer::new(&registry);
                let Ok(mut deserializer) = Deserializer::from_str(deserializer_hint.as_str())
                else {
                    info!("failed to construct deserializer");
                    info!(
                        "When attempting to apply edit @{prop_path} targeting value {val_type_name} for type {component_type_name}"
                    );
                    return;
                };
                let Ok(output) = reflect_deserializer
                        .deserialize(&mut deserializer)
                        .map_err(move |e| {
                            info!("failed to construct output - {:?}", e.to_string());
                            info!("When attempting to apply edit @{prop_path} targeting value {val_type_name} for type {component_type_name}");
                        }) else {
                            return;
                        };

                commands.trigger(DynamicComponentUiUpdateEvent {
                    ui_entity: event.focused_entity,
                    new_value: output,
                    component_ui_field_for: cap.clone(),
                });
            }
        },
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
    fn validate(&self) -> Result<(), String> {
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

    //Err("lol. lmao.".to_string())
}

fn manually_registering_trait_data_for_fun_and_profit(reg: ResMut<AppTypeRegistry>) {
    let mut registry = reg.write();
    registry.register_type_data::<bool, ReflectEditorFieldUI>();
    registry.register_type_data::<Transform, ReflectEditorPerFieldUI>();
    registry.register_type_data::<Transform, ReflectEditorHeaderUI>();
}
// fn register_hooks(mut world: World) {
//     world.register_component_hooks::<ImageNode_SansHandle>();
// }
//TODO:
//
// Ok remember, the big value add of all that work you just did is that now you can define header/struct
// UI creation in terms of ComponentUIContext. Your users do not want all that shit though.
// They want a function that lets them put boxes in the right spot on the UI.
//
// Your goal here is basically (component_ui_field_for: &ComponentUIFieldFor, ui_anchor: Entity) -> impl Bundle
// We want to hand them the shit they need to make UI that knows how to talk to the world and a spot to live in the containers, past that, we can be doneski.
//
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

#[reflect_trait]
pub trait EditorHeaderUI {
    fn construct_header_ui(&self, ui_anchor: Entity, commands: &mut Commands);
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
#[reflect_trait]
pub trait EditorPerFieldUI {
    fn construct_per_field_ui(&self, ctxt: &UiCtxt, commands: &mut Commands);
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

#[reflect_trait]
pub trait EditorFieldUI {
    fn construct_field_ui(&self, entity: Entity, commands: &mut Commands);
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

                commands.trigger(DynamicComponentUiUpdateEvent {
                    ui_entity: source.entity,
                    new_value: Box::new(!state.clone()),
                    component_ui_field_for: my_cap.clone(),
                });
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
// Prototype form-control type concept:
// Make a component hook for FormControl that, on being added to an entity, navigates up the heirarchy to find 'UiRoot', then stops, and creates a relationship between the root and the original targeted entity, and inserts some form-control state management THING if one does not already exist at the root.
//
// Form Events can be dispatched from individual form fields, which will use FormControl traversal to go be observed at the root.
//
// This should ideally be generic over some form state component T. Implementors will be expected to handle their own logic and register their own global observers per T.
//
//
//
// This model unfortunately stuffs all state management and event handling into the one entity at the UI root, but I think that's probably OK?
//
// World shapes for now can just be simple shapes per bevy example, we only need to prove out basic interactions and that each sub-form component makes a different shape.
//
#[derive(Component)]
#[component(on_add = form_element_marker_added)]
pub struct FormElementMarker;

fn form_element_marker_added(mut world: DeferredWorld, context: HookContext) {
    let mut find_join_point_state = world
        .try_query::<(Entity, Option<&ComponentUiFor>, Option<&ChildOf>)>()
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
        Ok(root) => {
            let mut commands = world.commands();
            commands
                .entity(root)
                .add_one_related::<FormElement>(context.entity);
            commands
                .entity(context.entity)
                .remove::<FormElementMarker>();
        }
    }
}

#[derive(Component, Clone)]
#[relationship(relationship_target = FormControl)]
pub struct FormElement {
    form_control: Entity,
}

#[derive(Component, Clone)]
#[relationship_target(relationship = FormElement)]
pub struct FormControl {
    elements: Vec<Entity>,
}
#[derive(EntityEvent, Clone, PartialEq, Debug, Reflect, Component)]
#[entity_event(propagate = &'static FormElement, auto_propagate)]
pub struct FormEvent<E: Clone + Reflect> {
    entity: Entity,
    event: E,
}

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
