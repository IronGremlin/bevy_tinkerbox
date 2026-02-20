use bevy::{
    ecs::{entity::EntityHashSet, query::QueryData, relationship::RelationshipSourceCollection},
    feathers::theme::{ThemeBackgroundColor, ThemeBorderColor},
    platform::collections::HashSet,
    prelude::*,
    ui_widgets::observe,
};

use crate::{
    ComponentUiFor, ComponentUisFor, EntityUiRoot,
    theme::{local_text::FontSize, local_tokens},
    ui_context_core::{SelectedEntityUiRoot, WorldTarget},
    widgets::{
        component_browser::{
            ComponentBrowserOpenRequest, ComponentBrowserWidgetRoot, component_browser_widget,
        },
        icons::IconImage,
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(handle_despawn_request);
    app.add_systems(Update, name_plate_update);
}

pub(crate) fn add_entity_button() -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Row,
            column_gap: px(16.),
            ..default()
        },
        ThemeBackgroundColor(local_tokens::PANE_BG),
        observe(add_entity_on_click),
        children![
            Text::new("Add Entity"),
            FontSize::Big.font(),
            (
                Name::new("Add Entity Button"),
                Node {
                    width: px(32.),
                    height: px(32.),
                    ..default()
                },
                IconImage::from_path("lucide/package-plus-white.png".to_owned())
            ),
        ],
    )
}

pub fn make_new_entity_ui(entity: Entity) -> impl Bundle {
    (
        Node {
            display: Display::Grid,
            border: UiRect::all(px(2.)),
            padding: UiRect::all(px(3.)),
            ..default()
        },
        ThemeBackgroundColor(local_tokens::PANE_BG),
        ThemeBorderColor(local_tokens::PANE_BORDER),
        EntityUiRoot {
            world_target: entity,
            desired_component_set: HashSet::new(),
            ride_along_components: HashSet::new(),
        },
        observe(add_component_button_on_click),
        children![(
            Node {
                flex_direction: FlexDirection::Row,
                column_gap: px(12.),
                ..default()
            },
            children![
                (
                    Text::new(format!("Entity({:?})", entity)),
                    EntityNamePlate,
                    FontSize::Big.font()
                ),
                (
                    Node {
                        width: px(24.),
                        height: px(24.),
                        ..default()
                    },
                    IconImage::from_path("lucide/list-plus-white.png".to_owned()),
                    observe(|src: On<Pointer<Click>>, mut commands: Commands| {
                        //eat clicks from children
                        if src.event_target() != src.original_event_target() {
                            return;
                        }
                        commands.trigger(ComponentBrowserOpenRequest::new(src.event_target()));
                    }),
                ),
                (
                    Node {
                        width: px(24.),
                        height: px(24.),
                        ..default()
                    },
                    IconImage {
                        color: Color::from(Srgba::RED),
                        ..IconImage::from_path("lucide/trash-2-white.png".to_owned())
                    },
                    observe(move |src: On<Pointer<Click>>, mut commands: Commands| {
                        //eat clicks from children
                        if src.event_target() != src.original_event_target() {
                            return;
                        }
                        commands.trigger(RemoveEntity { entity });
                    }),
                ),
            ]
        )],
    )
}

pub(crate) fn add_entity_on_click(
    _: On<Pointer<Click>>,
    mut commands: Commands,
    find_anchor: Query<Entity, With<SelectedEntityUiRoot>>,
) {
    let Ok(anchor) = find_anchor.single() else {
        return;
    };
    let world_target = commands.spawn_empty().id();
    let new_entity_ui = commands.spawn_empty().id();
    commands.entity(anchor).add_child(new_entity_ui);
    commands
        .entity(new_entity_ui)
        .insert(make_new_entity_ui(world_target));
}

pub(crate) fn add_component_button_on_click(
    source: On<ComponentBrowserOpenRequest>,
    mut commands: Commands,
    reg: Res<AppTypeRegistry>,
    find_component_ui_anchor: Query<Entity, With<EntityUiRoot>>,
    find_browser_widget_anchor: Query<Entity, With<ComponentBrowserWidgetRoot>>,
) {
    // We're only interested in events from our child button
    if source.event_target() == source.original_event_target() {
        return;
    }
    let Ok(c_anchor) = find_component_ui_anchor.get(source.event_target()) else {
        return;
    };
    let Ok(b_anchor) = find_browser_widget_anchor.single() else {
        return;
    };
    commands
        .entity(b_anchor)
        .despawn_children()
        .with_child(component_browser_widget(&reg, c_anchor));
}

#[derive(EntityEvent)]
pub struct RemoveEntity {
    entity: Entity,
}

fn handle_despawn_request(
    src: On<RemoveEntity>,
    uis: Query<&ComponentUisFor>,
    sans_node: Query<Entity, (With<ComponentUiFor>, Without<Node>)>,
    mut commands: Commands,
) {
    let mut dead_letter_queue = EntityHashSet::new();
    let dead = src.event_target();
    dead_letter_queue.add(dead);

    //Assumption:
    // All uis for components without nodes represent the root of some worldspace widget collection.
    // All uis for components that do contain nodes will be parented by the Entity who owns our EntityUiRoot,
    // and since we will be a WorldTarget, when we despawn, we will drag them all to hell with us.
    let _ = uis.get(dead).map(|x| {
        x.0.iter()
            .filter_map(|ent| sans_node.get(ent).ok())
            .for_each(|entity| {
                dead_letter_queue.add(entity);
            })
    });

    for n in dead_letter_queue {
        commands.entity(n).despawn();
    }
}
#[derive(Component)]
struct EntityNamePlate;

//TODO - We should handle name removal too.
fn name_plate_update(
    mut name_plates: Query<&mut Text, With<EntityNamePlate>>,
    names_we_care_about: Query<NamedWorldTarget, Changed<Name>>,
    kids: Query<&Children>,
) {
    for named_world_target in names_we_care_about.iter() {
        for desc in kids.iter_descendants(named_world_target.ui_root()) {
            if let Ok(mut plate) = name_plates.get_mut(desc) {
                plate.0 = named_world_target.name_plate()
            }
        }
    }
}

#[derive(QueryData)]
pub struct NamedWorldTarget {
    entity: Entity,
    name: Option<&'static Name>,
    world_target: &'static WorldTarget,
}
impl<'w, 's> NamedWorldTargetItem<'w, 's> {
    pub fn id(&self) -> Entity {
        self.entity
    }
    pub fn name_plate(&self) -> String {
        self.name
            .map(|n| format!("{} : Entity({:?})", n, self.entity))
            .unwrap_or(format!("Entity({:?})", self.entity))
    }
    pub fn ui_root(&self) -> Entity {
        self.world_target.ui_root()
    }
}
