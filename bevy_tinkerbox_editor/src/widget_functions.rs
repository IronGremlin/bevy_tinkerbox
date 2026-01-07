use bevy::{
    ecs::relationship::RelatedSpawner,
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    input::keyboard::KeyboardInput,
    input_focus::{FocusedInput, InputFocus},
    picking::hover::Hovered,
    prelude::*,
    ui_widgets::{ControlOrientation, CoreScrollbarThumb, Scrollbar, observe},
};

use bevy_ui_text_input::{
    TextInputBuffer, TextInputMode, TextInputNode, TextInputPrompt, TextInputStyle,
};

use crate::theme::colors;

pub fn scroll_area_demo<F>(that_which_is_scrolled: SpawnWith<F>) -> impl Bundle
where
    F: FnOnce(&mut RelatedSpawner<ChildOf>) + Send + Sync + 'static,
{
    (
        // Frame element which contains the scroll area and scrollbars.
        Node {
            display: Display::Grid,
            min_width: vw(20),
            min_height: vh(20),
            grid_template_columns: vec![RepeatedGridTrack::flex(1, 1.), RepeatedGridTrack::auto(1)],
            grid_template_rows: vec![RepeatedGridTrack::flex(1, 1.), RepeatedGridTrack::auto(1)],
            row_gap: px(2),
            column_gap: px(2),
            ..default()
        },
        Children::spawn((SpawnWith(move |parent: &mut RelatedSpawner<ChildOf>| {
            // The actual scrolling area.
            // Note that we're using `SpawnWith` here because we need to get the entity id of the
            // scroll area in order to set the target of the scrollbars.
            let scroll_area_id = parent
                .spawn((
                    Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(px(4)),
                        row_gap: px(2),
                        overflow: Overflow {
                            x: OverflowAxis::Clip,
                            y: OverflowAxis::Scroll,
                        },
                        ..default()
                    },
                    BackgroundColor(colors::gry_nut().into()),
                    ScrollPosition(Vec2::new(0.0, 0.0)),
                    Children::spawn(that_which_is_scrolled),
                ))
                .id();

            // Vertical scrollbar
            parent.spawn((
                Node {
                    min_width: px(8),
                    grid_row: GridPlacement::start(1),
                    grid_column: GridPlacement::start(2),
                    ..default()
                },
                Scrollbar {
                    orientation: ControlOrientation::Vertical,
                    target: scroll_area_id,
                    min_thumb_length: 16.0,
                },
                Children::spawn(Spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                    Hovered::default(),
                    BackgroundColor(colors::GRAY2.into()),
                    //TODO - This should be a broader 'hint' concept
                    // This currently works 'ok' to hint interactability but gets weird because the cursor often
                    // drifts off target during drag and therefor causes color to shift
                    // This
                    HoverBackground {
                        out: colors::GRAY2.into(),
                        over: colors::WHITE.into(),
                    },
                    BorderRadius::all(px(4)),
                    CoreScrollbarThumb,
                ))),
            ));
        }),)),
    )
}

#[derive(Component)]
#[component(on_insert = on_add_over_background)]
#[require(BackgroundColor)]
pub struct HoverBackground {
    pub over: Color,
    pub out: Color,
}
fn on_add_over_background(mut world: DeferredWorld, context: HookContext) {
    world
        .commands()
        .entity(context.entity)
        .observe(
            |source: On<Pointer<Out>>, mut q: Query<(&HoverBackground, &mut BackgroundColor)>| {
                let Ok((hbg, mut bg)) = q.get_mut(source.event_target()) else {
                    return;
                };
                *bg = BackgroundColor(hbg.out);
            },
        )
        .observe(
            |source: On<Pointer<Over>>, mut q: Query<(&HoverBackground, &mut BackgroundColor)>| {
                let Ok((hbg, mut bg)) = q.get_mut(source.event_target()) else {
                    return;
                };
                *bg = BackgroundColor(hbg.over);
            },
        );
}

pub fn centered(contents: impl Bundle) -> impl Bundle {
    ((
        Node {
            justify_content: JustifyContent::Center,
            ..default()
        },
        children![contents],
    ),)
}
pub fn filter_with_prompt<T>(prompt_text: impl Into<String>, text_filter_target: T) -> impl Bundle
where
    T: Bundle,
{
    (
        TextInputNode {
            mode: TextInputMode::SingleLine,
            max_chars: Some(20),
            clear_on_submit: true,
            ..default()
        },
        TextInputBuffer::default(),
        TextInputPrompt {
            text: prompt_text.into(),
            font: Some(TextFont::from_font_size(11.0)),
            ..default()
        },
        BackgroundColor(Srgba::hex("#46474d").unwrap_or(Srgba::WHITE).into()),
        TextFont::from_font_size(11.0),
        TextInputStyle { ..default() },
        Node {
            height: px(18),
            min_width: px(50),
            width: px(220),
            ..default()
        },
        take_focus_on_click(),
        text_filter_target,
    )
}
pub fn take_focus_on_click() -> impl Bundle {
    (
        observe(|event: On<Pointer<Click>>, mut focus: ResMut<InputFocus>| {
            focus.set(event.entity);
        }),
        observe(
            |event: On<FocusedInput<KeyboardInput>>, mut focus: ResMut<InputFocus>| {
                match event.input.logical_key {
                    bevy::input::keyboard::Key::Escape => {
                        focus.clear();
                    }
                    _ => {}
                };
            },
        ),
    )
}

pub fn text_row(caption: &str) -> (Text, TextFont) {
    (
        Text::new(caption),
        TextFont {
            font_size: 10.0,
            ..default()
        },
    )
}
