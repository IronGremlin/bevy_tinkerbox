use bevy::ecs::reflect::ReflectCommandExt;
use bevy::ecs::world::DeferredWorld;
use ron;
use ron::Deserializer;

use bevy::reflect::serde::ReflectDeserializer;

use bevy::reflect::TypeRegistry;
use bevy::reflect::serde::TypedReflectSerializer;

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

use crate::theme::local_text::FontSize;
use crate::ui_context_core::FieldAccessPath;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (update_input_text, generic_field_output_change_monitor).chain(),
    );
    app.add_observer(generic_field_output_watcher);
}

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
        Text::new(content),
        FontSize::Normal.font(),
        TextColor(Srgba::RED.into()),
    )
}

pub fn take_focus_on_click() -> impl Bundle {
    observe(|event: On<Pointer<Click>>, mut focus: ResMut<InputFocus>| {
        focus.set(event.entity);
    })
}

#[derive(Component)]
pub struct ValueInputOutput {
    pub result: Box<dyn Reflect>,
}
impl ValueInputOutput {
    pub fn from_output(output: impl Reflect) -> Self {
        Self {
            result: Box::new(output),
        }
    }
}

#[derive(Component)]
pub struct ValueInputInput {
    pub val: Box<dyn Reflect>,
}
impl ValueInputInput {
    pub fn from_input(input: impl Reflect) -> Self {
        Self {
            val: Box::new(input),
        }
    }
}

pub fn dynamic_handle_keyboard_interaction_for_input_field() -> impl Bundle {
    observe(
        |event: On<FocusedInput<KeyboardInput>>,
         mut focus: ResMut<InputFocus>,
         reg: Res<AppTypeRegistry>,
         mut q: Query<(&mut ValueInputOutput, &TextInputBuffer)>| {
            match event.input.logical_key {
                bevy::input::keyboard::Key::Escape => {
                    focus.clear();
                }
                _ => {}
            };

            if let Ok((mut value_output, tb)) = q.get_mut(event.focused_entity) {
                let real_registry = reg.read();
                let mut registry = TypeRegistry::new();
                let type_id = value_output
                    .result
                    .get_represented_type_info()
                    .unwrap()
                    .type_id();

                let value_registration = real_registry.get(type_id).unwrap();
                registry.add_registration(value_registration.clone());

                let text = tb.get_text();

                let val_type_name = value_registration
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
                    info!("When attempting to apply edit targeting value {val_type_name}");
                    return;
                };
                let Ok(output) =
                    reflect_deserializer
                        .deserialize(&mut deserializer)
                        .map_err(move |e| {
                            info!("failed to construct output - {:?}", e.to_string());
                            info!("When attempting to apply edit targeting value {val_type_name}");
                        })
                else {
                    return;
                };

                value_output.result.apply(&*output);
            }
        },
    )
}
fn value_input_field_no_opinions() -> impl Bundle {
    (
        TextInputNode {
            mode: TextInputMode::SingleLine,
            clear_on_submit: false,
            unfocus_on_submit: false,
            ..default()
        },
        TextInputBuffer::default(),
        BackgroundColor(Srgba::hex("#46474d").unwrap_or(Srgba::WHITE).into()),
        FontSize::Normal.font(),
        TextInputStyle { ..default() },
        Node {
            display: Display::Grid,
            max_height: percent(100),
            max_width: percent(100),
            ..default()
        },
        take_focus_on_click(),
    )
}
//known
pub fn concrete_value_input_field<T: Reflect + Clone>(item: T) -> impl Bundle {
    (
        value_input_field_no_opinions(),
        ValueInputInput::from_input(item.clone()),
        ValueInputOutput::from_output(item),
        dynamic_handle_keyboard_interaction_for_input_field(),
    )
}
//dynamic
pub fn dynamic_value_input_field(item: Box<dyn Reflect>) -> impl Bundle {
    (
        value_input_field_no_opinions(),
        ValueInputInput {
            val: item.reflect_clone().unwrap(),
        },
        ValueInputOutput { result: item },
        dynamic_handle_keyboard_interaction_for_input_field(),
    )
}

fn update_input_text(
    mut buffers: Query<(&mut TextInputBuffer, &ValueInputInput), Changed<ValueInputInput>>,
    reg: Res<AppTypeRegistry>,
) {
    for (mut buffer, input) in buffers.iter_mut() {
        let registry = reg.read();
        let serializer = TypedReflectSerializer::new(&*input.val, &*registry);
        let text = ron::to_string(&serializer).unwrap_or("".to_owned());

        if buffer.get_text() != "" {
            *buffer = TextInputBuffer::default();
        }

        buffer.editor.insert_string(text.as_str(), None);
    }
}
#[derive(EntityEvent)]
struct ChangedOutput {
    entity: Entity,
    output: Box<dyn Reflect>,
    fap: FieldAccessPath,
}
fn generic_field_output_change_monitor(
    modified_fields: Query<
        (Entity, &ValueInputOutput, &FieldAccessPath),
        Changed<ValueInputOutput>,
    >,

    mut commands: Commands,
) {
    for (entity, output, fap) in modified_fields.iter() {
        commands.trigger(ChangedOutput {
            entity,
            output: output.result.reflect_clone().unwrap(),
            fap: fap.clone(),
        });
    }
}

fn generic_field_output_watcher(
    src: On<ChangedOutput>,
    world: DeferredWorld,
    mut commands: Commands,
) {
    let path = src.fap.path.clone();
    let world_entity = src.fap.owning_entity;
    let component_ref = match world.get_reflect(world_entity, src.fap.component_type_id) {
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
    //Attempt comparison so we can avoid triggering change detection if the value is the same.
    if !src.output.reflect_partial_eq(old_val).unwrap_or(false) {
        let _thing = match old_val.try_apply(&*src.output) {
            Ok(f) => f,
            Err(e) => {
                info!("{:?}", e);
                return;
            }
        };
        commands.entity(world_entity).insert_reflect(shadow);
    }
}
