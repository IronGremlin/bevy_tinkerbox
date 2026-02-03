use bevy::{
    feathers::theme::ThemeBackgroundColor, platform::collections::HashSet, prelude::*,
    ui_widgets::observe,
};

use crate::{
    EntityUiRoot, ImageNodeSansHandle,
    theme::local_tokens,
    ui_context_core::SelectedEntityUiRoot,
    widgets::{
        component_browser::{
            ComponentBrowserOpenRequest, ComponentBrowserWidgetRoot, component_browser_widget,
        },
        general::{as_bundle, vertical_scroll_area},
    },
};

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
            (
                Name::new("Add Entity Button"),
                Node {
                    width: px(32.),
                    height: px(32.),
                    ..default()
                },
                ImageNodeSansHandle::from_path("lucide/package-plus-white.png".to_owned())
            ),
        ],
    )
}

pub fn make_new_entity_ui(entity: Entity) -> impl Bundle {
    vertical_scroll_area(as_bundle((
        Node {
            display: Display::Grid,
            ..default()
        },
        ThemeBackgroundColor(local_tokens::PANE_BG),
        EntityUiRoot {
            component_holder: entity,
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
                Text::new(format!("Entity({:?})", entity)),
                (
                    Name::new("Entity"),
                    Node {
                        width: px(24.),
                        height: px(24.),
                        ..default()
                    },
                    ImageNodeSansHandle::from_path("lucide/list-plus-white.png".to_owned()),
                    observe(|src: On<Pointer<Click>>, mut commands: Commands| {
                        //eat clicks from children
                        if src.event_target() != src.original_event_target() {
                            return;
                        }
                        commands.trigger(ComponentBrowserOpenRequest::new(src.event_target()));
                    }),
                ),
            ]
        )],
    )))
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
