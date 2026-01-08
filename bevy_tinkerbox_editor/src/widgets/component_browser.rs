use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy::ui_widgets::observe;

use std::any::TypeId;

use bevy::ecs::relationship::RelatedSpawner;

use bevy::ecs::spawn::SpawnWith;

use crate::WorldRequiredComponentExtension;
use crate::theme::colors;
use crate::widgets::general::{
    HoverBackground, centered, filter_with_prompt, hide_filtered_components, scroll_area_demo,
    text_row,
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
            Spawn(scroll_area_demo(spawn_component_entries(
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
            let h_background = HoverBackground {
                out: resting_color,
                over: resting_color.lighter(0.025),
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
        BackgroundColor(h_background.out),
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
        let r = world.resource::<AppTypeRegistry>();
        let registry = r.read();
        let name = registry
            .get(c_subject.type_id)
            .map(|x| x.type_info().type_path_table().short_path())
            .unwrap_or("");
        let mut required: Vec<(TypeId, &'static str)> = Vec::new();
        for val in world.required_components(c_subject.type_id) {
            let v_name = registry
                .get(val)
                .map(|e| e.type_info().type_path_table().short_path())
                .unwrap_or("");
            required.push((val.clone(), v_name.into()))
        }
        commands.trigger(ComponentSelection {
            entity: c_subject.ui_anchor,
            base: (c_subject.type_id, name),
            required,
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
    pub base: (TypeId, &'static str),
    pub required: Vec<(TypeId, &'static str)>,
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
