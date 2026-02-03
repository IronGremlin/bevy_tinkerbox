use bevy::ecs::world::DeferredWorld;
use bevy::feathers::theme::{ThemeBackgroundColor, ThemeToken};
use bevy::prelude::*;
use bevy::ui_widgets::observe;

use std::any::TypeId;

use bevy::ecs::relationship::RelatedSpawner;

use bevy::ecs::spawn::SpawnWith;

use crate::theme::{self, local_tokens};
use crate::widgets::general::{
    HoverBackground, centered, filter_with_prompt, hide_filtered_components, text_row,
    vertical_scroll_area,
};
pub(super) fn plugin(app: &mut App) {
    app.add_observer(close_component_browser_on_select);
    app.add_systems(
        Update,
        (hide_filtered_components::<SceneEditorComponentFilter, ComponentSubject>,),
    );
}
pub(crate) fn component_browser_widget(
    reg: &AppTypeRegistry,
    component_ui_anchor: Entity,
) -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(3),
            border_radius: BorderRadius::all(px(3)),
            ..default()
        },
        ThemeBackgroundColor(local_tokens::PANE_BG),
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
            Spawn(vertical_scroll_area(spawn_component_entries(
                reg.clone(),
                component_ui_anchor,
            ))),
        )),
    )
}

pub(crate) fn spawn_component_entries(
    registrations: AppTypeRegistry,
    component_ui_anchor: Entity,
) -> SpawnWith<impl FnOnce(&mut RelatedSpawner<ChildOf>)> {
    SpawnWith(move |parent: &mut RelatedSpawner<ChildOf>| {
        let lock = registrations.read();
        for entry in lock.iter() {
            if entry.data::<ReflectComponent>().is_none() {
                continue;
            }
            // Indicate if we can actually construct a dynamic component to edit:
            let (resting_color, highlight_color, warning): (ThemeToken, ThemeToken, &'static str) =
                if entry.data::<ReflectDefault>().is_none() {
                    (
                        theme::local_tokens::WARNING_BG,
                        theme::local_tokens::WARNING_PRIMARY,
                        "no ReflectDefault impl",
                    )
                } else {
                    (
                        theme::local_tokens::ITEM_BG,
                        theme::local_tokens::ITEM_ACTIVE,
                        "",
                    )
                };
            let name = format!(
                "{} | {}",
                entry.type_info().type_path_table().short_path(),
                warning
            );
            let h_background = HoverBackground {
                out: resting_color,
                over: highlight_color,
            };

            parent.spawn(component_entry(
                name,
                component_ui_anchor,
                entry.type_id(),
                h_background,
            ));
        }
    })
}

pub(crate) fn component_entry(
    name: String,
    component_ui_anchor: Entity,
    type_id: TypeId,
    h_background: HoverBackground,
) -> impl Bundle {
    (
        Node {
            column_gap: px(2),
            ..default()
        },
        Name::new(name.clone()),
        ComponentSubject {
            type_id: type_id,
            ui_anchor: component_ui_anchor,
        },
        Outline::new(Val::Px(2.), Val::ZERO, Color::NONE),
        ThemeBackgroundColor(h_background.out.clone()),
        h_background,
        observe(component_entry_on_click),
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
    )
}

pub(crate) fn component_entry_on_click(
    source: On<Pointer<Click>>,
    world: DeferredWorld,
    mut commands: Commands,
) {
    if let Some(c_subject) = world
        .entity(source.event_target())
        .get::<ComponentSubject>()
    {
        commands.trigger(ComponentSelection {
            entity: c_subject.ui_anchor,
            base: c_subject.type_id,
        });
    }
}

#[derive(Component)]
pub(crate) struct ComponentSubject {
    pub(crate) type_id: TypeId,
    pub(crate) ui_anchor: Entity,
}

#[derive(Component)]
pub struct SceneEditorComponentFilter;

#[derive(Component, Clone, EntityEvent)]
pub struct ComponentSelection {
    pub entity: Entity,
    pub base: TypeId,
}
#[derive(Component)]
pub struct ComponentBrowserWidgetRoot;
#[derive(EntityEvent)]
#[entity_event(propagate)]
#[entity_event(auto_propagate)]
pub struct ComponentBrowserOpenRequest {
    entity: Entity,
}
impl ComponentBrowserOpenRequest {
    pub fn new(entity: Entity) -> Self {
        Self { entity }
    }
}

fn close_component_browser_on_select(
    _: On<ComponentSelection>,
    mut commands: Commands,
    find_component_browser_anchor: Query<Entity, With<ComponentBrowserWidgetRoot>>,
) {
    let Ok(anchor) = find_component_browser_anchor.single() else {
        return;
    };
    commands.entity(anchor).despawn_children();
}
