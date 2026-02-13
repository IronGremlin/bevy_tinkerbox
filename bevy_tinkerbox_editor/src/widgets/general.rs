use bevy::{
    ecs::{
        lifecycle::HookContext,
        relationship::{RelatedSpawner, Relationship},
        world::DeferredWorld,
    },
    feathers::{
        theme::{ThemeBackgroundColor, ThemeToken, UiTheme},
        tokens,
    },
    input::keyboard::KeyboardInput,
    input_focus::{FocusedInput, InputFocus},
    picking::hover::Hovered,
    prelude::*,
    ui_widgets::{ControlOrientation, CoreScrollbarThumb, Scrollbar, observe},
};

use bevy_ui_text_input::{
    TextInputBuffer, TextInputMode, TextInputNode, TextInputPrompt, TextInputStyle,
};

use crate::{
    theme::{self, local_text::FontSize, local_tokens},
    widgets::field_input::ValueInputOutput,
};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(watch_for_close);
    app.add_systems(Update, value_edit_dispatch);
}

fn scroll_field(overflow: Overflow) -> Node {
    Node {
        display: Display::Flex,
        flex_direction: FlexDirection::Column,
        padding: UiRect::all(px(4)),
        row_gap: px(2),
        overflow,
        ..default()
    }
}
fn scroll_bar(scroll_target: Entity, orientation: ControlOrientation) -> impl Bundle {
    use ControlOrientation::*;
    (
        match orientation {
            Horizontal => Node {
                min_height: px(8),
                grid_row: GridPlacement::start(2),
                grid_column: GridPlacement::start(1),
                ..default()
            },
            Vertical => Node {
                min_width: px(8),
                grid_row: GridPlacement::start(1),
                grid_column: GridPlacement::start(2),
                ..default()
            },
        },
        Scrollbar {
            orientation: orientation,
            target: scroll_target,
            min_thumb_length: 16.0,
        },
        ThemeBackgroundColor(tokens::SLIDER_BG),
        ZIndex(10),
        Children::spawn(Spawn((
            Node {
                position_type: PositionType::Absolute,
                border_radius: BorderRadius::all(px(4)),
                ..default()
            },
            Hovered::default(),
            ThemeBackgroundColor(tokens::SLIDER_BAR),
            //TODO - This should be a broader 'hint' concept
            // Refactor to use tokens
            HoverBackground {
                out: tokens::SLIDER_BAR,
                over: local_tokens::SLIDER_ACTIVE,
            },
            CoreScrollbarThumb,
            ZIndex(20),
        ))),
    )
}
fn scroll_box() -> impl Bundle {
    (
        Node {
            display: Display::Grid,
            min_width: vw(20),
            max_width: vw(35),
            max_height: vmin(90),
            grid_template_columns: vec![RepeatedGridTrack::flex(1, 1.), RepeatedGridTrack::auto(1)],
            grid_template_rows: vec![RepeatedGridTrack::flex(1, 1.), RepeatedGridTrack::auto(1)],
            row_gap: px(2),
            column_gap: px(2),
            ..default()
        },
        BackgroundColor::from(Color::NONE),
    )
}

pub fn vertical_scroll_area<F>(that_which_is_scrolled: SpawnWith<F>) -> impl Bundle
where
    F: FnOnce(&mut RelatedSpawner<ChildOf>) + Send + Sync + 'static,
{
    scroll_area(ScrollAxes::Y, that_which_is_scrolled)
}

pub enum ScrollAxes {
    X,
    Y,
    XY,
}

pub fn scroll_area<F>(scroll_axes: ScrollAxes, that_which_is_scrolled: SpawnWith<F>) -> impl Bundle
where
    F: FnOnce(&mut RelatedSpawner<ChildOf>) + Send + Sync + 'static,
{
    (
        // Frame element which contains the scroll area and scrollbars.
        scroll_box(),
        Children::spawn((SpawnWith(move |parent: &mut RelatedSpawner<ChildOf>| {
            // The actual scrolling area.
            // Note that we're using `SpawnWith` here because we need to get the entity id of the
            // scroll area in order to set the target of the scrollbars.
            let scroll_area_id = parent
                .spawn((
                    scroll_field(match scroll_axes {
                        ScrollAxes::X => Overflow {
                            x: OverflowAxis::Scroll,
                            y: OverflowAxis::Clip,
                        },
                        ScrollAxes::Y => Overflow {
                            x: OverflowAxis::Clip,
                            y: OverflowAxis::Scroll,
                        },
                        ScrollAxes::XY => Overflow::scroll(),
                    }),
                    ThemeBackgroundColor(local_tokens::PANE_BG),
                    ScrollPosition(Vec2::new(0.0, 0.0)),
                    Children::spawn(that_which_is_scrolled),
                ))
                .id();
            match scroll_axes {
                ScrollAxes::X => {
                    parent.spawn(scroll_bar(scroll_area_id, ControlOrientation::Horizontal));
                }
                ScrollAxes::Y => {
                    parent.spawn(scroll_bar(scroll_area_id, ControlOrientation::Vertical));
                }
                ScrollAxes::XY => {
                    parent.spawn(scroll_bar(scroll_area_id, ControlOrientation::Vertical));
                    parent.spawn(scroll_bar(scroll_area_id, ControlOrientation::Horizontal));
                }
            };
        }),)),
    )
}
#[derive(Component)]
pub struct CloseRoot;

#[derive(EntityEvent)]
#[entity_event(propagate)]
#[entity_event(auto_propagate)]
pub struct CloseEvent {
    entity: Entity,
}
impl CloseEvent {
    pub fn new(entity: Entity) -> Self {
        Self { entity }
    }
}

fn watch_for_close(mut src: On<CloseEvent>, stop: Query<Has<CloseRoot>>, mut commands: Commands) {
    if stop.get(src.event_target()).unwrap_or(false) {
        src.propagate(false);
        commands.entity(src.event_target()).despawn();
    }
}

#[derive(Component)]
#[component(on_insert = on_add_over_background)]
#[require(BackgroundColor)]
pub struct HoverBackground {
    pub over: ThemeToken,
    pub out: ThemeToken,
}
impl HoverBackground {
    pub fn item() -> HoverBackground {
        HoverBackground {
            over: theme::local_tokens::ITEM_ACTIVE,
            out: theme::local_tokens::ITEM_BG,
        }
    }
}
fn on_add_over_background(mut world: DeferredWorld, context: HookContext) {
    let theme = world.resource::<UiTheme>();
    let hbg_og = world
        .entity(context.entity)
        .components::<&HoverBackground>();
    let HoverBackground { over, out } = hbg_og;
    let color_over = theme.color(&over);
    let color_out = theme.color(&out);
    world
        .commands()
        .entity(context.entity)
        .observe(
            move |source: On<Pointer<Out>>,
                  mut q: Query<(&HoverBackground, &mut BackgroundColor)>| {
                let Ok((_hbg, mut bg)) = q.get_mut(source.event_target()) else {
                    return;
                };
                *bg = BackgroundColor::from(color_out);
            },
        )
        .observe(
            move |source: On<Pointer<Over>>,
                  mut q: Query<(&HoverBackground, &mut BackgroundColor)>| {
                let Ok((_hbg, mut bg)) = q.get_mut(source.event_target()) else {
                    return;
                };
                *bg = BackgroundColor::from(color_over);
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
    (Text::new(caption), FontSize::Normal.font())
}
pub fn as_bundle(bundle: impl Bundle) -> SpawnWith<impl FnOnce(&mut RelatedSpawner<ChildOf>)> {
    SpawnWith(|p: &mut RelatedSpawner<ChildOf>| {
        p.spawn(bundle);
    })
}

pub fn hide_filtered_components<RootMarker, ItemMarker>(
    filter_text_buffer: Query<&TextInputBuffer, With<RootMarker>>,
    mut component_node_list: Query<(&mut Node, &Name), With<ItemMarker>>,
) where
    RootMarker: Component,
    ItemMarker: Component,
{
    let Ok(text) = filter_text_buffer.single().map(|b| b.get_text()) else {
        return;
    };

    for (mut node, name) in component_node_list.iter_mut() {
        node.display = if name.as_str().contains(text.as_str()) {
            Display::Flex
        } else {
            Display::None
        }
    }
}

// Prototype form-control type concept:
// Make a component hook for FormControl that, on being added to an entity, navigates up the heirarchy to find 'UiRoot', then stops, and creates a relationship between the root and the original targeted entity, and inserts some form-control state management THING if one does not already exist at the root.
//
// Form Events can be dispatched from individual form fields, which will use FormControl traversal to go be observed at the root.
//
// This should ideally be generic over some form state component T. Implementors will be expected to handle their own logic and register their own global observers per T.
//
//
//
// This model unfortunately stuffs all state management and event handling into the one entity at the UI root, but I think that's probably OK?
//
// World shapes for now can just be simple shapes per bevy example, we only need to prove out basic interactions and that each sub-form component makes a different shape.
//
#[derive(Component)]
pub struct FormControlSubject;
#[derive(Component)]
#[component(on_add = form_element_marker_added)]
pub struct FormElementMarker;

fn form_element_marker_added(mut world: DeferredWorld, context: HookContext) {
    let mut find_join_point_state = world
        .try_query::<(Entity, Option<&FormControlSubject>, Option<&ChildOf>)>()
        .unwrap();
    let find_join_point = world.query(&mut find_join_point_state);

    let mut traversal_cursor = find_join_point.get(context.entity);
    let mut traversal_result: Result<Entity, String> =
        Err("Could not find ancestor with UiRoot".to_owned());
    while traversal_cursor.is_ok() {
        match traversal_cursor {
            Ok((ui_for, Some(_), _)) => {
                traversal_result = Ok(ui_for);
                break;
            }
            Ok((_, None, Some(parent))) => {
                traversal_cursor = find_join_point.get(parent.get());
            }
            _ => {
                // We're only here if we hit the last ancestor and never found our UI root OR
                // we tried to access a dead entity. We don't really care about why this happened exactly so we can just fall back to the default error on our result.
                break;
            }
        }
    }
    match traversal_result {
        Err(e) => {
            info!({ e })
        }
        Ok(root) => {
            let mut commands = world.commands();
            commands
                .entity(root)
                .add_one_related::<FormElement>(context.entity);
            commands
                .entity(context.entity)
                .remove::<FormElementMarker>();
        }
    }
}

#[derive(Component, Clone)]
#[relationship(relationship_target = FormControl)]
pub struct FormElement {
    form_control: Entity,
}

#[derive(Component, Clone)]
#[relationship_target(relationship = FormElement)]
pub struct FormControl {
    elements: Vec<Entity>,
}
#[derive(EntityEvent, Clone, PartialEq, Debug, Reflect, Component)]
#[entity_event(propagate = &'static FormElement, auto_propagate)]
pub struct FormDataChanged {
    pub entity: Entity,
}

fn value_edit_dispatch(
    q: Query<Entity, (With<FormElement>, Changed<ValueInputOutput>)>,
    mut commands: Commands,
) {
    for origin in q.iter() {
        commands.trigger(FormDataChanged { entity: origin });
    }
}
