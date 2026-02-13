use bevy::{
    ecs::world::DeferredWorld,
    feathers::theme::{ThemeBackgroundColor, ThemeBorderColor},
    prelude::*,
    ui_widgets::observe,
};

use crate::{
    EditorUiScreenRoot, ImageNodeSansHandle,
    editor_override_traits::EditorFieldUI,
    theme::{self, local_text::FontSize, local_tokens},
    ui_context_core::{RefreshInputFields, UiCtxt},
    widgets::{
        add_entity_button::{NamedWorldTarget, NamedWorldTargetItem},
        general::{CloseEvent, CloseRoot, HoverBackground},
    },
};

#[derive(Component)]
struct OurTarget(pub Entity);

impl EditorFieldUI for Children {
    fn construct_field_ui(&self, ctxt: &UiCtxt, commands: &mut Commands) {
        let list_header_element = commands.spawn_empty().id();
        commands.entity(ctxt.ui_anchor()).insert((
            Node {
                display: Display::Grid,
                grid_auto_flow: GridAutoFlow::Row,
                grid_template_rows: vec![
                    RepeatedGridTrack::px(1, FontSize::Med.float() + 2.),
                    RepeatedGridTrack::max_content(1),
                ],
                border: UiRect::all(px(2.)),
                ..default()
            },
            ThemeBorderColor(local_tokens::PANE_BG),
        ));

        let list_item_layout = commands
            .spawn((
                Node {
                    display: Display::Grid,
                    grid_auto_rows: GridTrack::minmax(
                        MinTrackSizingFunction::Px(FontSize::Med.float()),
                        MaxTrackSizingFunction::MaxContent,
                    ),
                    grid_auto_flow: GridAutoFlow::Row,
                    //trash-can, up/down chev, list preview
                    grid_template_columns: vec![
                        RepeatedGridTrack::px(3, FontSize::Med.float() + 2.),
                        RepeatedGridTrack::min_content(1),
                    ],
                    ..default()
                },
                OurTarget(ctxt.world_target()),
                observe(children_on_refresh),
                observe(in_which_we_spawn_our_entity_selector),
            ))
            .id();

        commands.entity(list_header_element).insert(children_header(
            list_item_layout,
            ctxt.world_target(),
            "0",
            "Children",
        ));
        commands
            .entity(ctxt.ui_anchor())
            .add_children(&[list_header_element, list_item_layout]);
        commands.trigger(RefreshInputFields {
            component_ui_root: list_item_layout,
        });
    }
}

fn children_on_refresh(
    src: On<RefreshInputFields>,
    our_target: Query<&OurTarget>,
    children: Query<&Children>,
    named_world_targets: Query<NamedWorldTarget>,
    mut commands: Commands,
) {
    commands.entity(src.event_target()).despawn_children();
    info!("Children refreshed");
    let shadow_list = our_target
        .get(src.event_target())
        .and_then(|OurTarget(x)| children.get(*x))
        .map(|x| x.iter().filter_map(|n| named_world_targets.get(n).ok()))
        .expect("failed to retrieve child entities");

    for (i, n) in shadow_list.enumerate() {
        let idx: i16 = i.try_into().unwrap();
        let trash = commands
            .spawn((
                Node {
                    display: Display::Grid,
                    width: px(FontSize::Med.float()),
                    height: px(FontSize::Med.float()),
                    grid_row: GridPlacement::start(idx + 1),
                    grid_column: GridPlacement::start(1),
                    ..default()
                },
                BackgroundColor::from(Srgba::RED),
            ))
            .id();
        let up_chev = commands
            .spawn((
                Node {
                    display: Display::Grid,
                    width: px(FontSize::Med.float()),
                    height: px(FontSize::Med.float()),
                    grid_row: GridPlacement::start(idx + 1),
                    grid_column: GridPlacement::start(2),
                    ..default()
                },
                BackgroundColor::from(Srgba::BLUE),
            ))
            .id();
        let down_chev = commands
            .spawn((
                Node {
                    display: Display::Grid,
                    width: px(FontSize::Med.float()),
                    height: px(FontSize::Med.float()),
                    grid_row: GridPlacement::start(idx + 1),
                    grid_column: GridPlacement::start(3),
                    ..default()
                },
                BackgroundColor::from(Srgba::GREEN),
            ))
            .id();
        let ui_anchor = commands
            .spawn(entity_display_item(&n, idx))
            .id();
        commands
            .entity(src.event_target())
            .add_children(&[trash, up_chev, down_chev, ui_anchor]);
    }
}
fn children_header(
    ui_layout: Entity,
    world_target: Entity,
    field_name: impl Into<String>,
    type_name: impl Into<String>,
) -> impl Bundle {
    (
        Node {
            display: Display::Grid,
            grid_auto_flow: GridAutoFlow::Column,
            grid_template_columns: vec![
                RepeatedGridTrack::min_content(1),
                RepeatedGridTrack::flex(1, 1.),
                RepeatedGridTrack::min_content(1),
                RepeatedGridTrack::min_content(1),
            ],
            min_width: Val::Percent(10.0),
            ..default()
        },
        children![
            (
                Node::default(),
                children![(Text::new(field_name), FontSize::Normal.font(),)]
            ),
            Node {
                min_width: percent(10.),
                ..default()
            },
            (
                Node {
                    display: Display::Grid,
                    justify_self: JustifySelf::End,
                    max_height: px(30.),
                    ..default()
                },
                children![(
                    Text::new(type_name),
                    FontSize::Normal.font(),
                    ThemeBackgroundColor(local_tokens::ITEM_ACTIVE)
                ),]
            ),
            (
                Name::new("Search For Entry Button"),
                Node {
                    width: px(14.),
                    height: px(14.),
                    ..default()
                },
                ImageNodeSansHandle::from_path("lucide/search-white.png".to_owned()),
                observe(
                    move |_: On<Pointer<Click>>, _world: DeferredWorld, mut commands: Commands| {
                        commands.trigger(EntitySelectionRequest {
                            entity: ui_layout,
                            subject: world_target,
                        });
                    }
                )
            ),
            (
                Name::new("Add List Entry Button"),
                Node {
                    width: px(14.),
                    height: px(14.),
                    ..default()
                },
                ImageNodeSansHandle::from_path("lucide/package-plus-white.png".to_owned()),
                observe(
                    move |_: On<Pointer<Click>>, _world: DeferredWorld, mut commands: Commands| {
                        //TODO - implement a real entity generation function
                        commands.trigger(RefreshInputFields {
                            component_ui_root: ui_layout,
                        });
                    }
                )
            ),
        ],
    )
}

#[derive(EntityEvent)]
/// An event which fires when the ui requests that the user select an entity.
/// This event is typically accompanied by an [EntityListSelection]
pub struct EntitySelectionRequest {
    /// The entity that owns this request
    /// EG, where we are listening for the answer.
    pub entity: Entity,
    /// The subject of the request - eg, the target of a relation.
    pub subject: Entity,
}

/// An event which fires when an entity is selected from a pick-list.
#[derive(EntityEvent)]
#[entity_event(propagate)]
#[entity_event(auto_propagate)]
pub struct EntityListSelection {
    /// The target of the event
    pub entity: Entity,
    /// The entity represented by the pick-list item.
    pub subject: Entity,
}
fn watch_entity_list_selection(clicked: Entity) -> impl Bundle {
    observe(move |src: On<Pointer<Click>>, mut commands: Commands| {
        commands.trigger(EntityListSelection {
            entity: src.event_target(),
            subject: clicked,
        });
        commands.trigger(CloseEvent::new(src.event_target()));
    })
}
fn entity_display_item(named_world_target: &NamedWorldTargetItem, idx: i16) -> impl Bundle {
    (
        Node {
            display: Display::Grid,
            grid_row: GridPlacement::start(idx + 1),
            grid_column: GridPlacement::start(4),
            ..default()
        },
        ThemeBackgroundColor(theme::local_tokens::ITEM_BG),
        //TODO - implement some kind of anchor link here
        HoverBackground::item(),
        children![(
            Text::new(named_world_target.name_plate()),
            FontSize::Normal.font()
        )],
    )
}
fn entity_search_item(named_world_target: &NamedWorldTargetItem) -> impl Bundle {
    (
        Node {
            display: Display::Grid,
            ..default()
        },
        ThemeBackgroundColor(theme::local_tokens::ITEM_BG),
        HoverBackground::item(),
        watch_entity_list_selection(named_world_target.id()),
        children![(
            Text::new(named_world_target.name_plate()),
            FontSize::Normal.font()
        )],
    )
}
fn in_which_we_spawn_our_entity_selector(
    src: On<EntitySelectionRequest>,
    global_root: Single<Entity, With<EditorUiScreenRoot>>,
    world_targets: Query<NamedWorldTarget>,
    mut commands: Commands,
) {
    // TODO - add 'closed' observer here
    let popup = commands
        .spawn((
            Node {
                display: Display::Grid,
                position_type: PositionType::Absolute,
                left: percent(50),
                top: percent(50),
                min_width: px(250),
                grid_template_rows: vec![
                    RepeatedGridTrack::px(1, FontSize::Med.float() + 2.),
                    RepeatedGridTrack::vmin(1, 35.),
                ],
                ..default()
            },
            entity_selection_handler(src.event().subject, src.event_target()),
            CloseRoot,
        ))
        .id();
    commands.entity(popup).with_child((
        Node {
            display: Display::Grid,
            grid_row: GridPlacement::start(1),
            grid_template_columns: vec![
                RepeatedGridTrack::percent(1, 90.),
                RepeatedGridTrack::percent(1, 10.),
            ],
            ..default()
        },
        children![
            (
                Node { ..default() },
                children![(Text::new("Entity Selector"), FontSize::Med.font())]
            ),
            //TODO - add a real close icon here
            (
                Node { ..default() },
                observe(|src: On<Pointer<Click>>, mut commands: Commands| {
                    commands.trigger(CloseEvent::new(src.event_target()));
                }),
                BackgroundColor::from(Srgba::RED)
            )
        ],
    ));
    //some scrolling layout container
    // TODO - actually make this scroll dude come on now
    let item_layout = commands
        .spawn(
            Node {
                display: Display::Grid,
                grid_row: GridPlacement::start(2),
                grid_auto_flow: GridAutoFlow::Row,
                grid_auto_rows: vec![GridTrack::px(FontSize::Normal.float() + 2.)],
                ..default()
            },
        )
        .id();
    commands.entity(popup).add_child(item_layout);
    commands.entity(*global_root).add_child(popup);
    for named_world_target in world_targets.iter() {
        commands
            .entity(item_layout)
            .with_child(entity_search_item(&named_world_target));
    }
}
fn entity_selection_handler(target_parent: Entity, ui_parent: Entity) -> impl Bundle {
    observe(
        move |src: On<EntityListSelection>, mut commands: Commands| {
            commands
                .entity(target_parent)
                .add_child(src.event().subject);
            commands.trigger(RefreshInputFields {
                component_ui_root: ui_parent,
            });
        },
    )
}
