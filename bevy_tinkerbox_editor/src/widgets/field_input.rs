use bevy::asset::ron::Deserializer;

use bevy::reflect::serde::ReflectDeserializer;

use bevy::reflect::TypeRegistry;

use bevy::input::keyboard::KeyboardInput;

use bevy::input_focus::FocusedInput;

use bevy::input_focus::InputFocus;

use bevy_ui_text_input::TextInputStyle;

use bevy_ui_text_input::TextInputMode;

use bevy_ui_text_input::TextInputNode;

use bevy::{prelude::*, ui_widgets::observe};
use bevy_ui_text_input::TextInputBuffer;
use cosmic_text::Edit;
use serde::de::DeserializeSeed;

use crate::{DynamicComponentUiUpdateEvent, FieldAccessPath};

pub fn input_field_error(content: impl Into<String>) -> impl Bundle {
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

pub fn take_focus_on_click() -> impl Bundle {
    observe(|event: On<Pointer<Click>>, mut focus: ResMut<InputFocus>| {
        focus.set(event.entity);
    })
}

pub fn handle_keyboard_interaction_for_input_field() -> impl Bundle {
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

                commands.trigger(DynamicComponentUiUpdateEvent::new(
                    event.focused_entity,
                    output,
                    cap.clone(),
                ));
            }
        },
    )
}
