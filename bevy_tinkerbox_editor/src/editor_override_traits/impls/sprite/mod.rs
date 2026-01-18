use ::bevy::prelude::*;
use bevy::{
    asset::RenderAssetUsages,
    camera::{RenderTarget, visibility::RenderLayers},
    render::render_resource::{TextureDimension, TextureFormat, TextureUsages},
    sprite::Anchor,
    ui_widgets::observe,
};

use crate::{
    AssortedIcons, LoadingStatus, UiCtxt,
    drag_snap::{DragSnapState2d, SnapDrag, SnapDragEnd, WorldSnap2dGrid},
    editor_override_traits::{EditorPerFieldUI, impls::sprite::texture_atlas_layout::GridArgs},
    widgets::{
        field_input::{ValueInputInput, ValueInputOutput, better_value_input_field},
        general::{
            FormControl, FormControlSubject, FormDataChanged, FormElement, FormElementMarker,
        },
    },
};

pub(super) fn plugin(app: &mut App) {
    app.init_gizmo_group::<TextureAtlasPreviewGizmos>();
    app.init_state::<TALWindowStatus>();
    app.add_observer(initialize_texture_atlas_layout_ui);

    app.add_systems(
        Update,
        (
            taw_update_cascade,
            texture_atlas_draggables,
            render_texture_atlas_cells,
        )
            .chain(),
    );
}
#[derive(Component)]
struct TextureAtlasPreviewNode;

#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
enum TALWindowStatus {
    Open,
    #[default]
    Closed,
}

#[derive(Component)]
struct SizeOfTexture(UVec2);

fn make_close_tal_ui(
    world_target: Entity,
    gargs_holder: Entity,
) -> impl Fn(
    On<Pointer<Click>>,
    Query<&GridArgs>,
    Query<&mut Sprite>,
    ResMut<Assets<TextureAtlasLayout>>,
    ResMut<NextState<TALWindowStatus>>,
) {
    move |_src: On<Pointer<Click>>,
          gargs: Query<&GridArgs>,
          mut sprites: Query<&mut Sprite>,
          mut tals: ResMut<Assets<TextureAtlasLayout>>,
          mut state: ResMut<NextState<TALWindowStatus>>| {
        let Ok(args) = gargs.get(gargs_holder) else {
            return;
        };
        let Ok(mut sprite) = sprites.get_mut(world_target) else {
            return;
        };
        let layout: TextureAtlasLayout = Into::into(*args);
        sprite.texture_atlas = Some(TextureAtlas {
            layout: tals.add(layout),
            index: 0,
        });
        state.set(TALWindowStatus::Closed);
    }
}

fn initialize_texture_atlas_layout_ui(
    src: On<OpenTextureAtlasUi>,
    mut commands: Commands,
    mut state: ResMut<NextState<TALWindowStatus>>,
    icons: Res<AssortedIcons>,
    mut images: ResMut<Assets<Image>>,
    mut gizmo_cfg: ResMut<GizmoConfigStore>,
) {
    state.set(TALWindowStatus::Open);
    gizmo_cfg.insert(
        GizmoConfig {
            depth_bias: -1.0,
            render_layers: RenderLayers::layer(1),
            ..default()
        },
        TextureAtlasPreviewGizmos,
    );
    let image_aspect_ratio = images.get(src.image.id()).unwrap().aspect_ratio();
    let size_of_image = images.get(src.image.id()).unwrap().size();
    let (w, h) = if image_aspect_ratio.is_landscape() {
        (
            1. - image_aspect_ratio.inverse().ratio(),
            image_aspect_ratio.inverse().ratio(),
        )
    } else {
        (image_aspect_ratio.ratio(), 1. - image_aspect_ratio.ratio())
    };
    let mut image = Image::new_uninit(
        default(),
        TextureDimension::D2,
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::all(),
    );
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;
    let camera_canvas = images.add(image);

    let initial_grid_args = src.args;

    commands.spawn((
        Sprite {
            image: src.image.clone(),
            ..default()
        },
        Anchor(Vec2::new(1., 1.) * -0.5),
        SizeOfTexture(size_of_image),
        Transform::from_xyz(0., 0., 1.),
        Pickable {
            should_block_lower: false,
            ..default()
        },
        RenderLayers::layer(1),
        DespawnOnExit(TALWindowStatus::Open),
    ));
    commands.spawn((
        Sprite {
            image: icons.checker_board.clone(),
            rect: Some(Rect::from_center_size(Vec2::ZERO, size_of_image.as_vec2())),
            ..default()
        },
        Anchor(Vec2::new(1., 1.) * -0.5),
        Transform::from_xyz(0., 0., 0.),
        RenderLayers::layer(1),
        DespawnOnExit(TALWindowStatus::Open),
    ));

    let cam = commands
        .spawn((
            Camera2d,
            Camera {
                order: 1,
                target: RenderTarget::Image(camera_canvas.clone().into()),
                ..default()
            },
            Transform::from_scale(Vec2::splat(0.25).extend(1.))
                .with_translation((0.5 * size_of_image.as_vec2()).extend(1.)),
            RenderLayers::layer(1),
            DespawnOnExit(TALWindowStatus::Open),
        ))
        .id();
    let gargs_holder = commands
        .spawn((
            Node {
                display: Display::Grid,
                min_width: vmax(50.),
                max_width: percent(100.),
                height: vmin(50.),
                border: UiRect::all(px(4.)),
                justify_content: JustifyContent::Center,
                ..default()
            },
            BorderColor::all(Color::from(Srgba::WHITE)),
            BackgroundColor::from(Srgba::BLACK),
            FormControlSubject,
            initial_grid_args,
            DespawnOnExit(TALWindowStatus::Open),
            observe(
                |src: On<FormDataChanged>,
                 form_fields: Query<(&ValueInputOutput, &GridArgFieldKey)>,
                 handles: Query<Entity, With<DragHandle>>,
                 mut wrap: Query<&mut texture_atlas_layout::GridArgs>,
                 mut commands: Commands| {
                    let _ = form_fields.get(src.original_event_target()).map(
                        |(output, grid_args_key)| {
                            wrap.get_mut(src.event_target()).map(|mut orig_gargs| {
                                // We need to do this dance to avoid looping forever on changes.
                                let mut gargs = orig_gargs.clone();
                                match *grid_args_key {
                                    GridArgFieldKey::CellSizeX => {
                                        gargs.cell_size.x.apply(&*output.result)
                                    }
                                    GridArgFieldKey::CellSizeY => {
                                        gargs.cell_size.y.apply(&*output.result)
                                    }
                                    GridArgFieldKey::NumColumns => {
                                        gargs.columns.apply(&*output.result)
                                    }
                                    GridArgFieldKey::NumRows => gargs.rows.apply(&*output.result),
                                    GridArgFieldKey::OffsetX => {
                                        gargs.offset.x.apply(&*output.result)
                                    }
                                    GridArgFieldKey::OffsetY => {
                                        gargs.offset.y.apply(&*output.result)
                                    }
                                    GridArgFieldKey::PaddingX => {
                                        gargs.padding.x.apply(&*output.result)
                                    }
                                    GridArgFieldKey::PaddingY => {
                                        gargs.padding.y.apply(&*output.result)
                                    }
                                };
                                if orig_gargs.set_if_neq(gargs) {
                                    //If we successfully mutate our gargs, clear our drag handles so we can respawn them.
                                    handles.iter().for_each(|h| commands.entity(h).despawn());
                                }
                            })
                        },
                    );
                },
            ),
        ))
        .id();
    commands
        .entity(gargs_holder)
        .with_child((
            Node {
                width: percent(100.),
                height: px(36.),
                display: Display::Grid,
                grid_template_columns: vec![
                    RepeatedGridTrack::percent(1, 85.),
                    RepeatedGridTrack::fr(1, 25.),
                ],
                ..default()
            },
            BackgroundColor::from(Srgba::BLUE),
            children![
                (
                    Node {
                        justify_self: JustifySelf::Center,
                        height: px(36.),
                        ..default()
                    },
                    children![(
                        Text::new("Texture Atlas Layout"),
                        TextFont::from_font_size(18.)
                    )]
                ),
                (
                    Node {
                        justify_self: JustifySelf::End,
                        width: px(22.),
                        height: px(22.),
                        border: UiRect::all(px(2.)),
                        ..default()
                    },
                    BackgroundColor::from(Srgba::RED),
                    BorderColor::all(Srgba::WHITE),
                    observe(make_close_tal_ui(src.event_target(), gargs_holder))
                )
            ],
        ))
        .with_child(texture_atlas_preview_ui_bundle(initial_grid_args))
        .with_child((
            Node {
                width: px(1000. * w),
                height: px(1000. * h),
                border: UiRect::all(px(3.)),
                ..default()
            },
            BorderColor::all(Color::from(Srgba::BLUE)),
            Pickable {
                should_block_lower: false,
                ..default()
            },
            ViewportNode { camera: cam },
            TextureAtlasPreviewNode,
            DespawnOnExit(TALWindowStatus::Open),
        ));
}
//God this sounds so much cooler than it is
fn taw_update_cascade(
    ws: Query<
        (&texture_atlas_layout::GridArgs, &FormControl),
        Changed<texture_atlas_layout::GridArgs>,
    >,
    mut text_inputs: Query<(&mut ValueInputInput, &GridArgFieldKey), With<FormElement>>,
) {
    for (gargs, control) in ws.iter() {
        for entity in control.iter() {
            let Ok((mut input, grid_arg_key)) = text_inputs.get_mut(entity) else {
                continue;
            };
            if input.is_changed() {
                continue;
            }
            match grid_arg_key {
                GridArgFieldKey::CellSizeX => input.val = Box::new(gargs.cell_size.x),
                GridArgFieldKey::CellSizeY => input.val = Box::new(gargs.cell_size.y),
                GridArgFieldKey::NumColumns => input.val = Box::new(gargs.columns),
                GridArgFieldKey::NumRows => input.val = Box::new(gargs.rows),
                GridArgFieldKey::OffsetX => input.val = Box::new(gargs.offset.x),
                GridArgFieldKey::OffsetY => input.val = Box::new(gargs.offset.y),
                GridArgFieldKey::PaddingX => input.val = Box::new(gargs.padding.x),
                GridArgFieldKey::PaddingY => input.val = Box::new(gargs.padding.y),
            }
        }
    }
}

#[derive(Component)]
struct CellPreviewAnchor(Vec2);
#[derive(Component)]
struct DragHandle;
fn texture_atlas_draggables(
    ws: Query<&texture_atlas_layout::GridArgs, Changed<texture_atlas_layout::GridArgs>>,
    vp_nodes: Query<&ViewportNode, With<TextureAtlasPreviewNode>>,
    q3: Query<&SizeOfTexture>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let Ok(SizeOfTexture(texture_size)) = q3.single() else {
        return;
    };
    for gargs in ws.iter() {
        let Ok(vp_node) = vp_nodes.single() else {
            info!(
                "Assumed singular TextureAtlasViewport, found: {:?}",
                vp_nodes.iter().len()
            );
            return;
        };
        let height = texture_size.as_vec2().y;
        let width = texture_size.as_vec2().x;
        let local_offset = gargs.offset.as_vec2() * Vec2::new(1., -1.);
        // These should represent the corners of the upper-left-most cell in
        // the texture atlas grid.
        let upper_left = local_offset + Vec2::Y * height;
        let lower_right: Vec2 =
            gargs.cell_size.as_vec2() * Vec2::new(1., -1.) + local_offset + Vec2::Y * height;
        commands.spawn((
            Mesh2d(meshes.add(Circle::new(3.))),
            MeshMaterial2d(materials.add(Color::from(Srgba::RED.with_alpha(0.65)))),
            Transform::from_translation(upper_left.extend(2.)),
            Pickable::default(),
            WorldSnap2dGrid {
                camera: vp_node.camera,
            },
            DragHandle,
            RenderLayers::layer(1),
            DespawnOnExit(TALWindowStatus::Open),
            observe(
                |src: On<Pointer<SnapDrag>>, mut my_sad_ass: Query<&mut Transform>| {
                    if let Ok(mut xform) = my_sad_ass.get_mut(src.event_target()) {
                        xform.translation = src.last_seen.extend(2.);
                    }
                },
            ),
            observe(
                move |src: On<Pointer<SnapDragEnd>>,
                      mut commands: Commands,
                      handles: Query<Entity, With<DragHandle>>,
                      wrapper: Query<Entity, With<texture_atlas_layout::GridArgs>>| {
                    for handle in handles.iter() {
                        commands.entity(handle).despawn();
                    }
                    let Ok(wrap) = wrapper.single() else {
                        return;
                    };

                    let corner = Vec2::Y * height;
                    let base_line = src.last_seen - corner;
                    commands
                        .entity(wrap)
                        .entry::<texture_atlas_layout::GridArgs>()
                        .and_modify(move |mut gargs| {
                            let mut g_args = gargs.clone();
                            let mut terminal_pos = base_line.clone();
                            // We do not support a negative offset.
                            // Also we have to invert y because sprite co-ords are from the top down,
                            // so downward drag = negative offset.
                            terminal_pos.y = terminal_pos.y * -1.;
                            // If we drag off the target area we just clamp that to our real region.
                            // If we're in lala land, reset to zed.
                            terminal_pos.y = terminal_pos.y.clamp(0., height);
                            terminal_pos.x = terminal_pos.x.clamp(0., width);
                            g_args.offset = terminal_pos.as_uvec2();
                            gargs.set_if_neq(g_args);
                        });
                },
            ),
        ));
        commands.spawn((
            Mesh2d(meshes.add(Circle::new(3.))),
            MeshMaterial2d(materials.add(Color::from(Srgba::BLUE.with_alpha(0.65)))),
            CellPreviewAnchor(upper_left),
            WorldSnap2dGrid {
                camera: vp_node.camera,
            },
            DragHandle,
            Transform::from_translation(lower_right.extend(2.)),
            RenderLayers::layer(1),
            DespawnOnExit(TALWindowStatus::Open),
            observe(
                |src: On<Pointer<SnapDrag>>, mut my_sad_ass: Query<&mut Transform>| {
                    if let Ok(mut xform) = my_sad_ass.get_mut(src.event_target()) {
                        xform.translation = src.last_seen.extend(2.);
                    }
                },
            ),
            observe(
                |src: On<Pointer<SnapDragEnd>>,
                 drag: Query<&CellPreviewAnchor>,
                 mut commands: Commands,
                 handles: Query<Entity, With<DragHandle>>,
                 wrapper: Query<Entity, With<texture_atlas_layout::GridArgs>>| {
                    let Ok(cpa) = drag.get(src.event_target()) else {
                        return;
                    };
                    let new_cell_size = (cpa.0 - src.last_seen).abs().as_uvec2();
                    for handle in handles.iter() {
                        commands.entity(handle).despawn();
                    }
                    let Ok(wrap) = wrapper.single() else {
                        return;
                    };
                    commands
                        .entity(wrap)
                        .entry::<texture_atlas_layout::GridArgs>()
                        .and_modify(move |mut gargs| {
                            let mut g_args = gargs.clone();
                            g_args.cell_size = new_cell_size;
                            gargs.set_if_neq(g_args);
                        });
                },
            ),
        ));
    }
}

#[derive(Default, Reflect, GizmoConfigGroup)]
struct TextureAtlasPreviewGizmos;

fn render_texture_atlas_cells(
    q: Query<&texture_atlas_layout::GridArgs>,
    q2: Query<(&DragSnapState2d, &CellPreviewAnchor)>,
    q3: Query<&SizeOfTexture>,
    mut gizmos: Gizmos<TextureAtlasPreviewGizmos>,
) {
    let Ok(SizeOfTexture(texture_size)) = q3.single() else {
        return;
    };
    for gargs in q.iter() {
        let layout = TextureAtlasLayout::from_grid(
            gargs.cell_size,
            gargs.columns,
            gargs.rows,
            if gargs.padding == UVec2::ZERO {
                None
            } else {
                Some(gargs.padding)
            },
            if gargs.offset == UVec2::ZERO {
                None
            } else {
                Some(gargs.offset)
            },
        );

        for urect in layout.textures.iter() {
            let rect = urect.as_rect();
            // Ok so -technically- pixel co-ordinates are y inverted.
            // As long as we're a grid this doesn't actually matter but if someone came along and
            // wanted to add arbitrarily sized region support this would break very confusingly.
            //
            let center = rect.center() * Vec2::new(1., -1.) + Vec2::Y * texture_size.as_vec2().y;

            gizmos.primitive_2d(
                &Rectangle::from_corners(rect.min, rect.max),
                center,
                Srgba::GREEN,
            );
        }
    }
    for (state, cpa) in q2.iter() {
        gizmos.primitive_2d(
            &Rectangle::from_corners(cpa.0, state.last_seen),
            cpa.0 + (cpa.0 - state.last_seen) * -0.5,
            Srgba::BLUE,
        );
    }
}

//TODO -
// define event for initializing texture atlas ui with world entity and initial texture atlas, if any.
// define event for closing texture atlas ui and writing final texture atlas to world entity
//
#[derive(EntityEvent)]
struct OpenTextureAtlasUi {
    entity: Entity,
    args: GridArgs,
    image: Handle<Image>,
}

impl EditorPerFieldUI for Sprite {
    fn construct_per_field_ui(&self, ctxt: &UiCtxt, commands: &mut Commands) {
        let (world_target, ui_anchor) = (ctxt.world_target(), ctxt.ui_anchor());

        let handle_click = move |src: On<Pointer<Click>>,
                                 sprites: Query<&Sprite>,
                                 layouts: Res<Assets<TextureAtlasLayout>>,
                                 mut commands: Commands| {
            let Ok(sprite) = sprites.get(world_target) else {
                return;
            };
            let gargs: GridArgs = sprite
                .texture_atlas
                .as_ref()
                .and_then(|texa| layouts.get(texa.layout.id()))
                .map(|layout| layout.clone().into())
                .unwrap_or_default();

            commands
                .entity(world_target)
                .trigger(|e| OpenTextureAtlasUi {
                    entity: e,
                    args: gargs,
                    image: sprite.image.clone(),
                });
        };

        match ctxt.path() {
            "texture_atlas" => {
                //TODO - make this a real UI element that doesn't suck
                commands.entity(ctxt.ui_anchor()).with_child((
                    Node::default(),
                    children![(
                        Node {
                            width: px(12.0),
                            height: px(12.0),
                            border: UiRect::all(px(1.)),
                            ..default()
                        },
                        BorderColor::all(Srgba::WHITE),
                        BackgroundColor::from(Srgba::BLACK),
                        observe(handle_click),
                    )],
                ));
            }
            _ => {
                ctxt.next(commands);
            }
        }
    }
}
mod layout {
    pub const BIG_COLUMN_WIDTH: f32 = 64.;
    pub const BIG_ROW_HEIGHT: f32 = 18.;
    pub const SUB_COL_WIDTH: f32 = 24.;
    pub const PRIMARY_FONT_SIZE: f32 = 9.;
}
use layout::*;
#[derive(Reflect, Clone, Copy, Eq, PartialEq, Component)]
enum GridArgFieldKey {
    CellSizeX,
    CellSizeY,
    NumColumns,
    NumRows,
    OffsetX,
    OffsetY,
    PaddingX,
    PaddingY,
}

fn texture_atlas_preview_ui_bundle(args: texture_atlas_layout::GridArgs) -> impl Bundle {
    use GridArgFieldKey::*;
    fn grid_cell_1x2(col: i16) -> impl Bundle {
        Node {
            display: Display::Grid,
            grid_column: GridPlacement::start(col),
            grid_row: GridPlacement::span(2),
            ..default()
        }
    }
    fn grid_cell_2x1(col: i16, row: i16) -> impl Bundle {
        Node {
            display: Display::Grid,
            grid_column: GridPlacement::start_end(col, col + 2),
            grid_row: GridPlacement::start(row),
            ..default()
        }
    }
    fn sub_label(text: impl Into<String>) -> impl Bundle {
        (Text::new(text), TextFont::from_font_size(PRIMARY_FONT_SIZE))
    }
    fn label(text: impl Into<String>) -> impl Bundle {
        (
            Node {
                display: Display::Grid,
                ..default()
            },
            children![(Text::new(text), TextFont::from_font_size(PRIMARY_FONT_SIZE))],
        )
    }
    fn input_for(gargs: texture_atlas_layout::GridArgs, key: GridArgFieldKey) -> impl Bundle {
        (
            Node {
                display: Display::Grid,
                border: UiRect::all(px(1.)),
                padding: UiRect::horizontal(px(2.)),
                ..default()
            },
            BorderColor::from(Srgba::WHITE),
            BackgroundColor(Srgba::hex("#46474d").unwrap_or(Srgba::WHITE).into()),
            children![(
                match key {
                    CellSizeX => better_value_input_field(gargs.cell_size.x),
                    CellSizeY => better_value_input_field(gargs.cell_size.y),
                    NumColumns => better_value_input_field(gargs.columns),
                    NumRows => better_value_input_field(gargs.rows),
                    OffsetX => better_value_input_field(gargs.offset.x),
                    OffsetY => better_value_input_field(gargs.offset.y),
                    PaddingX => better_value_input_field(gargs.padding.x),
                    PaddingY => better_value_input_field(gargs.padding.y),
                },
                key,
                FormElementMarker
            )],
        )
    }

    (
        Node {
            display: Display::Grid,
            grid_template_columns: vec![RepeatedGridTrack::px(9, BIG_COLUMN_WIDTH)],
            grid_template_rows: vec![RepeatedGridTrack::px(2, BIG_ROW_HEIGHT)],

            ..default()
        },
        DespawnOnExit(TALWindowStatus::Open),
        children![
            (grid_cell_1x2(1), children![sub_label("Cell Size:")]),
            (
                grid_cell_1x2(2),
                children![
                    (
                        Node {
                            display: Display::Grid,
                            grid_template_columns: vec![RepeatedGridTrack::px(2, SUB_COL_WIDTH)],
                            ..default()
                        },
                        children![(label("X:")), (input_for(args.clone(), CellSizeX))]
                    ),
                    (
                        Node {
                            display: Display::Grid,
                            grid_template_columns: vec![RepeatedGridTrack::px(2, SUB_COL_WIDTH)],
                            ..default()
                        },
                        children![(label("Y:")), (input_for(args.clone(), CellSizeY))]
                    ),
                ]
            ),
            (
                grid_cell_2x1(3, 1),
                children![(
                    Node {
                        display: Display::Grid,
                        grid_template_columns: vec![RepeatedGridTrack::px(2, SUB_COL_WIDTH * 2.)],
                        ..default()
                    },
                    children![(label("Rows:")), (input_for(args.clone(), NumRows))]
                ),]
            ),
            (
                grid_cell_2x1(3, 2),
                children![(
                    Node {
                        display: Display::Grid,
                        grid_template_columns: vec![RepeatedGridTrack::px(2, SUB_COL_WIDTH * 2.)],
                        ..default()
                    },
                    children![(label("Columns:")), (input_for(args.clone(), NumColumns))]
                ),]
            ),
            (grid_cell_1x2(5), children![sub_label("Offset:")]),
            (
                grid_cell_1x2(6),
                children![
                    (
                        Node {
                            display: Display::Grid,
                            grid_template_columns: vec![RepeatedGridTrack::px(2, SUB_COL_WIDTH)],
                            ..default()
                        },
                        children![(label("X:")), (input_for(args.clone(), OffsetX))]
                    ),
                    (
                        Node {
                            display: Display::Grid,
                            grid_template_columns: vec![RepeatedGridTrack::px(2, SUB_COL_WIDTH)],
                            ..default()
                        },
                        children![(label("Y:")), (input_for(args.clone(), OffsetY))]
                    ),
                ]
            ),
            (grid_cell_1x2(7), children![sub_label("Padding:")]),
            (
                grid_cell_1x2(8),
                children![
                    (
                        Node {
                            display: Display::Grid,
                            grid_template_columns: vec![RepeatedGridTrack::px(2, SUB_COL_WIDTH)],
                            ..default()
                        },
                        children![(label("X:")), (input_for(args.clone(), PaddingX))]
                    ),
                    (
                        Node {
                            display: Display::Grid,
                            grid_template_columns: vec![RepeatedGridTrack::px(2, SUB_COL_WIDTH)],
                            ..default()
                        },
                        children![(label("Y:")), (input_for(args.clone(), PaddingY))]
                    ),
                ]
            )
        ],
    )
}
pub mod texture_atlas_layout {
    use ::bevy::prelude::*;
    #[derive(Clone, Copy, Eq, PartialEq, Component, Debug)]
    pub struct GridArgs {
        pub cell_size: UVec2,
        pub rows: u32,
        pub columns: u32,
        pub padding: UVec2,
        pub offset: UVec2,
    }
    impl Default for GridArgs {
        fn default() -> Self {
            Self {
                cell_size: UVec2::new(1, 1),
                rows: 1,
                columns: 1,
                padding: Default::default(),
                offset: Default::default(),
            }
        }
    }
    impl From<TextureAtlasLayout> for GridArgs {
        fn from(value: TextureAtlasLayout) -> Self {
            if value.textures.is_empty() {
                return GridArgs::default();
            }
            let mut rows = 1;
            let mut columns = 1;
            let mut x_pad = None;
            let mut y_pad = None;
            let offset = value.textures.get(0).unwrap().min.clone();
            let cell_size = value.textures.get(0).unwrap().size();
            let mut x_max = cell_size.x;
            let mut col_max = cell_size.y;
            for rect in value.textures.iter() {
                // if our cells ever change size we're not assessing a grid.
                if cell_size != rect.size() {
                    return GridArgs::default();
                }
                if rect.max.y > col_max {
                    // We have dropped down a row (remember pixel co-ords are inverted, greater values are "down")

                    rows = rows + 1;

                    // The distance between the top of this rectangle and the bottom of the formerly tallest
                    // is our vertical padding.
                    let pady = rect.min.y - col_max;
                    // If we have no set padding, trust what we get. If our padding changes, we're not assessing a grid.
                    if let Some(pad_val) = y_pad {
                        if pady != pad_val {
                            return GridArgs::default();
                        }
                    } else {
                        y_pad = Some(pady);
                    }

                    col_max = rect.max.y;
                }

                if rect.max.x > x_max {
                    // We've moved right, this must be a new column.
                    columns = columns + 1;
                    // The distance between the right side of formerly furthest right cell and the left side of this cell is our horizontal padding.
                    let padx = rect.min.x - x_max;
                    if let Some(pad_val) = x_pad {
                        if padx != pad_val {
                            return GridArgs::default();
                        }
                    } else {
                        x_pad = Some(padx);
                    }
                    x_max = rect.max.x;
                }
            }
            let padding = match (x_pad, y_pad) {
                (None, None) => UVec2::ZERO,
                (None, Some(y)) => UVec2::new(0, y),
                (Some(x), None) => UVec2::new(x, 0),
                (Some(x), Some(y)) => UVec2::new(x, y),
            };
            return GridArgs {
                cell_size,
                rows,
                columns,
                padding,
                offset,
            };
        }
    }
    impl Into<TextureAtlasLayout> for GridArgs {
        fn into(self) -> TextureAtlasLayout {
            TextureAtlasLayout::from_grid(
                self.cell_size,
                self.columns,
                self.rows,
                if self.padding == UVec2::ZERO {
                    None
                } else {
                    Some(self.padding)
                },
                if self.offset == UVec2::ZERO {
                    None
                } else {
                    Some(self.offset)
                },
            )
        }
    }
    #[cfg(test)]
    mod test {
        use bevy::{
            image::TextureAtlasLayout,
            math::{URect, UVec2},
        };

        use crate::editor_override_traits::impls::sprite::texture_atlas_layout::GridArgs;

        #[test]
        fn round_trip() {
            let gargs = GridArgs {
                cell_size: UVec2::splat(32),
                rows: 2,
                columns: 6,
                padding: UVec2::splat(1),
                offset: UVec2::ZERO,
            };
            let layout: TextureAtlasLayout = gargs.into();
            let gargs2: GridArgs = layout.into();
            assert_eq!(gargs, gargs2);
        }
        #[test]
        fn round_trip_less_happy() {
            let gargs = GridArgs {
                cell_size: UVec2::splat(32),
                rows: 2,
                columns: 6,
                padding: UVec2::splat(1),
                offset: UVec2::splat(2),
            };
            let layout: TextureAtlasLayout = gargs.into();
            let gargs2: GridArgs = layout.into();
            assert_eq!(gargs, gargs2);
        }
        #[test]
        fn gigo() {
            let gargs = GridArgs {
                cell_size: UVec2::splat(32),
                rows: 2,
                columns: 6,
                padding: UVec2::splat(1),
                offset: UVec2::ZERO,
            };
            let mut layout: TextureAtlasLayout = gargs.into();
            layout.add_texture(URect::from_center_size(UVec2::ZERO, UVec2::splat(24)));
            let gargs2: GridArgs = layout.into();
            assert_eq!(GridArgs::default(), gargs2);
        }
    }
}
