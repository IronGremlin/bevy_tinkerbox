use std::any::TypeId;

use bevy::{prelude::*, ui_widgets::observe};

use crate::{widgets::{component_browser::ComponentSelection, icons::IconImage}, EntityUiRoot};

#[derive(Component)]
pub struct RideAlongComponent(pub TypeId);

pub fn view_only_component(name: String, type_id: TypeId) -> impl Bundle {
    (
        Node { ..default() },
        RideAlongComponent(type_id),
        children![
            (Text::new(name), TextFont::from_font_size(11.),),
            (
                Name::new("Promote Ridealong"),
                Node {
                    width: px(12.),
                    height: px(12.),
                    ..default()
                },
                IconImage::from_path("lucide/square-pen-white.png".to_owned()),
                observe(
                    move |src: On<Pointer<Click>>,
                          find_anchor: Query<(Entity, Option<&ChildOf>, Option<&EntityUiRoot>)>,
                          mut commands: Commands| {
                        let mut cursor: Result<
                            (Entity, Option<&ChildOf>, Option<&EntityUiRoot>),
                            String,
                        > = find_anchor
                            .get(src.event_target())
                            .map_err(|_| "Could not find entity root".to_owned());

                        while cursor.is_ok() {
                            let Ok((entity, m_parent, m_root)) = cursor else {
                                break;
                            };
                            if let Some(_) = m_root {
                                commands.trigger(ComponentSelection {
                                    entity,
                                    base: type_id,
                                });
                                break;
                            }
                            if let Some(parent) = m_parent {
                                cursor = find_anchor
                                    .get(parent.0)
                                    .map_err(|_| "Could not find entity root".to_owned());
                                continue;
                            }
                            break;
                        }
                        if cursor.is_err() {
                            error!("Could not find entity root");
                        }
                    }
                ),
            ),
        ],
    )
}
