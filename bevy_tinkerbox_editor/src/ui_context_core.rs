use std::any::TypeId;

use ::bevy::prelude::*;
use bevy::{
    ecs::{reflect::ReflectCommandExt, relationship::RelatedSpawner, world::DeferredWorld},
    feathers::{
        controls::radio,
        theme::{ThemeBorderColor, ThemedText},
    },
    platform::collections::HashSet,
    reflect::{
        DynamicEnum, Enum, EnumInfo, OpaqueInfo, ParsedPath, ReflectKind, TypeInfo,
        TypeRegistration, VariantType,
    },
    ui::Checked,
    ui_widgets::{RadioButton, RadioGroup, ValueChange, observe},
};

use crate::{
    ComponentUiFor, ImageNodeSansHandle, UpdateComponentFieldValue,
    WorldRequiredComponentExtension,
    editor_override_traits::{
        ReflectEditorFieldUI, ReflectEditorHeaderUI, ReflectEditorPerFieldUI,
    },
    instantiate_or_die,
    theme::local_tokens,
    view_only_component,
    widgets::{
        component_browser::ComponentSelection,
        field_input::{dynamic_value_input_field, input_field_error},
        general::{CloseEvent, CloseRoot},
        scene_actions::ComponentInstantiation,
        view_only_component::RideAlongComponent,
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(root);
    app.add_observer(component_ui_initializer);
    app.add_observer(scene_component_ui_instantiator);
}

pub(crate) fn root(source: On<ComponentSelection>, dworld: DeferredWorld, mut commands: Commands) {
    let window_root = source.event_target();

    let reg = dworld.resource::<AppTypeRegistry>();

    let mut component_ui_state = match dworld.entity(window_root).get::<EntityUiRoot>() {
        Some(x) => x.clone(),
        None => panic!("Couldn't find valid EntityUiRoot"),
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
        let Ok(kids) = dworld.entity(window_root).get_components::<&Children>() else {
            panic!("failed to initialize children for root");
        };
        for kid in kids.iter() {
            let Ok(ride_along) = dworld.entity(kid).get_components::<&RideAlongComponent>() else {
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

// TODO -
// This is basically the same function as 'root', but we
// need to do some work to go track down the ui-root inside the scroll container,
// and then also we shouldn't attempt to instantiate new components because we've already instantiated these
// world targets.
// There are ways to tie these together into one operation that isn't a white-hot pile of dogshit,
// but this works for now so we're leaving it be.
fn scene_component_ui_instantiator(
    src: On<ComponentInstantiation>,
    world: DeferredWorld,
    mut commands: Commands,
) {
    let mut window_root = src.event_target();
    let type_id = src.type_id;
    let reg = world.resource::<AppTypeRegistry>();
    let mut all_my_kids = world.try_query::<&Children>().unwrap();
    let Some(mut component_ui_state) = all_my_kids
        .query(&world)
        .iter_descendants(window_root.clone())
        .find_map(|ent| {
            if let Some(ui_state) = world.entity(ent).get::<EntityUiRoot>() {
                window_root = ent;
                return Some(ui_state.clone());
            } else {
                None
            }
        })
    else {
        panic!("bummer");
    };
    component_ui_state.desired_component_set.insert(type_id);
    let the_one_in_the_world = component_ui_state.component_holder;
    let reqs = world.required_components(type_id);
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
pub(crate) struct RefreshInputFields {
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
            ThemeBorderColor(local_tokens::PANE_BORDER),
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
    let Ok(kids) = world
        .entity(source.component_ui_root)
        .get_components::<&Children>()
    else {
        //TODO - handle this
        return;
    };
    for kid in kids.iter() {
        if let Ok(_ride_along) = world.entity(kid).get_components::<&RideAlongComponent>() {
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
    pub fn ui_anchor(&self) -> Entity {
        self.step_context.local_ui_focus
    }
    pub fn path(&self) -> &str {
        self.step_context.local_path.as_str()
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
impl<'a> ComponentUiStepContext<'a> {
    fn local_type_name(&self) -> &'static str {
        self.local_type_info.type_path_table().short_path()
    }
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
                    commands.trigger(RefreshInputFields {
                        component_ui_root: bucket,
                    });
                    return;
                }
                Err(_) => todo!(),
            }
        }

        match step_context.local_type_info {
            TypeInfo::Struct(struct_info) => {
                self.handle_struct(
                    step_context,
                    struct_info,
                    maybe_struct_level_trait,
                    commands,
                );
            }
            TypeInfo::TupleStruct(ts_info) => {
                self.handle_tuple_struct(step_context, ts_info, maybe_struct_level_trait, commands);
            }
            TypeInfo::Enum(enum_info) => {
                self.handle_enum(step_context, enum_info, commands);
            }
            TypeInfo::Opaque(o_info) => {
                self.handle_opaque(step_context, o_info, commands);
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

    fn handle_struct(
        &self,
        step_context: ComponentUiStepContext,
        struct_info: &bevy::reflect::StructInfo,
        maybe_struct_level_trait: Option<&ReflectEditorPerFieldUI>,
        commands: &mut Commands,
    ) {
        let type_name = step_context.local_type_name();
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

    fn handle_tuple_struct(
        &self,
        step_context: ComponentUiStepContext,
        ts_info: &bevy::reflect::TupleStructInfo,
        maybe_struct_level_trait: Option<&ReflectEditorPerFieldUI>,
        commands: &mut Commands,
    ) {
        let type_name = step_context.local_type_name();
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

            let property_path = if step_context.local_path != "SelectedEntityUiRoot" {
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

    fn handle_enum(
        &self,
        step_context: ComponentUiStepContext<'_>,
        enum_info: &EnumInfo,
        commands: &mut Commands,
    ) {
        let type_name = step_context.local_type_name();
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
            .spawn(radio_button_group(
                num_variants.try_into().unwrap(),
                enum_info,
                &cap,
                &enum_metadata,
                enum_variant_index,
            ))
            .id();

        commands
            .entity(step_context.local_ui_focus)
            .add_child(radio_group_container)
            .add_child(structured_variant_layout_container);

        handle_enum_variant(
            &self,
            enum_metadata,
            enum_variant_index,
            structured_variant_layout_container,
            commands,
        );
    }
    fn handle_opaque(
        &self,
        step_context: ComponentUiStepContext<'_>,
        o_info: &OpaqueInfo,
        commands: &mut Commands,
    ) {
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
                ParsedPath::parse(step_context.local_path.as_str()).map_err(|e| e.to_string())
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
                    .map(|we| {
                        let val = we
                            .try_as_reflect()
                            .expect("Partial reflect should impl reflect why can this fail");
                        let boxed = val
                            .reflect_clone()
                            .expect("field input types must be clonable");
                        dynamic_value_input_field(boxed)
                    })
                    .map(|x| (x, cap))
            }) {
            Ok(f) => commands.entity(row).with_child(f),
            Err(t) => commands.entity(row).with_child(input_field_error(t)),
        };

        commands.entity(row).with_child((
            Node { ..default() },
            Text::new(o_info.type_path_table().short_path()),
            TextFont::from_font_size(10.0),
        ));
        commands.trigger(RefreshInputFields {
            component_ui_root: step_context.local_ui_focus,
        });
    }
}

fn radio_button_group(
    num_variants: u16,
    enum_info: &EnumInfo,
    cap: &FieldAccessPath,
    enum_metadata: &EnumMetadata,
    current_idx: usize,
) -> impl Bundle {
    let info = enum_info.clone();
    (
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
        },
        RadioGroup,
        cap.clone(),
        enum_metadata.clone(),
        observe(
            |value_change: On<ValueChange<Entity>>,
             kids: Query<&Children>,
             q_radio: Query<Entity, With<RadioButton>>,
             mut commands: Commands| {
                for radio in q_radio.iter_many(kids.get(value_change.event_target()).unwrap()) {
                    if radio == value_change.value {
                        commands.entity(radio).insert(Checked);
                    } else {
                        commands.entity(radio).remove::<Checked>();
                    }
                }
            },
        ),
        observe(enum_radio_observer),
        observe(enum_subelement_observer),
        Children::spawn(SpawnWith(move |parent: &mut RelatedSpawner<ChildOf>| {
            info.iter().enumerate().for_each(|(i, variant_info)| {
                if i == current_idx {
                    parent.spawn((
                        radio(Checked, Spawn((Text::new(variant_info.name()), ThemedText))),
                        SelectionIndex(i),
                    ));
                } else {
                    parent.spawn((
                        radio((), Spawn((Text::new(variant_info.name()), ThemedText))),
                        SelectionIndex(i),
                    ));
                }
            });
        })),
    )
}

//Here we try our best to handle presentation and world mutation separately.
// This lets us delegate the somewhat nasty and convoluted dynamic ref mutation to one spot,
// but also let's us re-use as much of this as possible and provide kind of a nice model for
// how the event contract is supposed to work.
fn enum_radio_observer(
    event: On<ValueChange<Entity>>,
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
    let SelectionIndex(s_index) = match q2.get(event.value) {
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
                let triggered_event = UpdateComponentFieldValue {
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
    source: On<RefreshInputFields>,
    mut commands: Commands,
    world: DeferredWorld,
) {
    let cap = match world
        .entity(source.event_target())
        .get_components::<&FieldAccessPath>()
    {
        Ok(f) => f,
        Err(_) => {
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
        Ok(f) => f,
        Err(_) => {
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

#[derive(Component, Clone)]
pub struct EntityUiRoot {
    pub component_holder: Entity,
    pub desired_component_set: HashSet<TypeId>,
    pub ride_along_components: HashSet<TypeId>,
}
