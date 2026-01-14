use crate::{
    AssortedIcons, ComponentUiFor, ComponentUisFor, DynamicComponentUiUpdateEvent, EntityUiRoot,
    FieldAccessPath, FieldUiRequestedFor, ImageNodeSansHandle, LoadingStatus, MainEditorCamera,
    UiCtxt,
    editor_override_traits::{
        EditorFieldUI, EditorHeaderUI, EditorPerFieldUI, ReflectEditorFieldUI,
        ReflectEditorHeaderUI, ReflectEditorPerFieldUI,
    },
};
use bevy::{
    asset::{RenderAssetUsages, io::file::FileAssetReader},
    camera::{RenderTarget, visibility::RenderLayers},
    ecs::world::DeferredWorld,
    image::ImageLoader,
    math::VectorSpace,
    prelude::*,
    render::render_resource::{TextureDimension, TextureFormat, TextureUsages},
    ui_widgets::observe,
    window::PrimaryWindow,
};
use bevy_file_dialog::{EntityFileDialogExt, EntityScopedDialogEvent};
pub mod transform;
pub(super) fn plugin(app: &mut App) {
    app.add_plugins(MeshPickingPlugin);
    app.add_plugins(transform::plugin);
    app.add_systems(
        PreStartup,
        manually_registering_trait_data_for_fun_and_profit,
    );
    app.add_systems(OnEnter(LoadingStatus::Complete), initialize_test);
    app.add_systems(
        Update,
        (rescale_texture_atlas_preview_camera, render_wrapper),
    );
}

fn manually_registering_trait_data_for_fun_and_profit(reg: ResMut<AppTypeRegistry>) {
    let mut registry = reg.write();
    registry.register_type_data::<bool, ReflectEditorFieldUI>();
    registry.register_type_data::<Handle<Image>, ReflectEditorFieldUI>();
    registry.register_type_data::<Transform, ReflectEditorPerFieldUI>();
    registry.register_type_data::<Transform, ReflectEditorHeaderUI>();
}

impl EditorFieldUI for bool {
    fn construct_field_ui(&self, entity: Entity, commands: &mut Commands) {
        let click_watcher = observe(
            |source: On<Pointer<Click>>, world: DeferredWorld, mut commands: Commands| {
                let my_cap = world
                    .entity(source.entity)
                    .get_components::<&FieldAccessPath>()
                    .unwrap();

                let component = world
                    .get_reflect(my_cap.owning_entity, my_cap.component_type_id)
                    .unwrap();
                let state = my_cap.path.element::<bool>(component).unwrap();

                commands.trigger(DynamicComponentUiUpdateEvent::new(
                    source.entity,
                    Box::new(!state.clone()),
                    my_cap.clone(),
                ));
            },
        );

        let world_watcher = observe(
            |source: On<FieldUiRequestedFor>, world: DeferredWorld, mut commands: Commands| {
                let my_cap = world
                    .entity(source.component_ui_root)
                    .get_components::<&FieldAccessPath>()
                    .unwrap();
                let component = world
                    .get_reflect(my_cap.owning_entity, my_cap.component_type_id)
                    .unwrap();
                let state = my_cap.path.element::<bool>(component).unwrap();
                commands.entity(source.component_ui_root).insert(if *state {
                    BackgroundColor::from(Srgba::GREEN)
                } else {
                    BackgroundColor::from(Srgba::BLACK)
                });
            },
        );
        commands.entity(entity).insert((
            Node {
                width: px(12.0),
                height: px(12.0),
                border: UiRect::all(px(1.)),
                ..default()
            },
            BorderColor::all(Srgba::WHITE),
            if *self {
                BackgroundColor::from(Srgba::GREEN)
            } else {
                BackgroundColor::from(Srgba::BLACK)
            },
            click_watcher,
            world_watcher,
        ));
    }
}
impl EditorPerFieldUI for Sprite {
    fn construct_per_field_ui(&self, ctxt: &UiCtxt, commands: &mut Commands) {
        match ctxt.path() {
            "image" => {}
            "texture_atlas" => {}
            "color" => {}
            "custom_size" => {}
            "rect" => {}
            "image_mode" => {}
            _ => {}
        };
        ctxt.next(commands);
    }
}

impl EditorFieldUI for Handle<Image> {
    fn construct_field_ui(&self, entity: Entity, commands: &mut Commands) {
        commands.entity(entity).insert((
            Node {
                display: Display::Grid,
                grid_auto_flow: GridAutoFlow::Column,
                column_gap: px(3.),
                width: percent(100.),
                ..default()
            },
            observe(
                |event: On<EntityScopedDialogEvent>,
                 world: DeferredWorld,
                 mut s_commands: Commands| {
                    let my_cap = world
                        .entity(event.event_target())
                        .components::<&FieldAccessPath>()
                        .clone();
                    let assets = world.resource::<AssetServer>();
                    use bevy_file_dialog::EntityScopedDialogResult::*;
                    match event.clone().result {
                        Pick(file_pick) => {
                            let full_path = file_pick.path.clone().to_owned();
                            //TODO - figure out how to do this without hard-coding this string.
                            let root = FileAssetReader::get_base_path().join("assets");
                            let path = full_path.strip_prefix(root).unwrap();
                            let handle: Handle<Image> = assets.load(path.to_owned());
                            s_commands.trigger(DynamicComponentUiUpdateEvent::new(
                                event.event_target(),
                                Box::new(handle),
                                my_cap.clone(),
                            ));
                        }
                        _ => {}
                    };
                },
            ),
            observe(
                |src: On<FieldUiRequestedFor>, world: DeferredWorld, mut s_commands: Commands| {
                    let my_cap = world
                        .entity(src.component_ui_root)
                        .get_components::<&FieldAccessPath>()
                        .unwrap();
                    let component = world
                        .get_reflect(my_cap.owning_entity, my_cap.component_type_id)
                        .unwrap();
                    let image = my_cap.path.element::<Handle<Image>>(component).unwrap();
                    let path = image
                        .path()
                        .map(|x| x.to_string())
                        .unwrap_or("<<unkown>>".to_owned());
                    let text_node = world
                        .entity(src.event_target())
                        .components::<&Children>()
                        .get(1)
                        .unwrap();
                    let text_entity = world
                        .entity(*text_node)
                        .components::<&Children>()
                        .get(0)
                        .unwrap();
                    s_commands
                        .entity(*text_entity)
                        .entry::<Text>()
                        .and_modify(|mut txt| {
                            txt.0 = path;
                        });
                },
            ),
            children![
                (Node {
                    width: px(5.0),
                    ..default()
                },),
                (
                    Node::default(),
                    BackgroundColor(Color::from(Srgba::gray(0.3))),
                    children![(
                        Text::new(
                            self.path()
                                .map(|n| n.to_string())
                                .unwrap_or("<<unknown>>".to_owned())
                        ),
                        TextLayout::new_with_justify(Justify::Center),
                        TextFont::from_font_size(8.)
                    )]
                ),
                (
                    Node {
                        width: px(12.0),
                        height: px(12.0),
                        border: UiRect::all(px(1.)),
                        ..default()
                    },
                    ImageNodeSansHandle::from_path("lucide/folder-white.png".to_owned()),
                    BorderColor::all(Srgba::WHITE),
                    observe(move |_: On<Pointer<Click>>, mut s_commands: Commands| {
                        s_commands
                            .entity(entity)
                            .with_dialog()
                            .add_filter("image", ImageLoader::SUPPORTED_FILE_EXTENSIONS)
                            .set_title("Select Image")
                            .pick_file_path();
                    }),
                )
            ],
        ));
    }
}
#[derive(Component)]
struct TextureAtlasPreviewNode;
#[derive(Component)]
struct TextureAtlasWrapper(TextureAtlasLayout);
fn initialize_test(
    mut commands: Commands,
    icons: Res<AssortedIcons>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let aspect_ratio_for_duck = images.get(icons.ducky.id()).unwrap().aspect_ratio();
    let size_of_duck = images.get(icons.ducky.id()).unwrap().size();
    let (w, h) = if aspect_ratio_for_duck.is_landscape() {
        (
            1. - aspect_ratio_for_duck.inverse().ratio(),
            aspect_ratio_for_duck.inverse().ratio(),
        )
    } else {
        (
            aspect_ratio_for_duck.ratio(),
            1. - aspect_ratio_for_duck.ratio(),
        )
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
    // let sprite_mat_mesh = meshes.add(Rectangle::from_size(size_of_duck.as_vec2()));
    // let sprite_mat_material = materials.add(ColorMaterial::from(icons.checker_board.clone()));

    commands.spawn((
        Sprite {
            image: icons.ducky.clone(),
            ..default()
        },
        Transform::from_xyz(0., 0., 1.),
        Pickable {
            should_block_lower: false,
            ..default()
        },
        RenderLayers::layer(1),
    ));
    commands.spawn((
        Sprite {
            image: icons.checker_board.clone(),
            rect: Some(Rect::from_center_size(Vec2::ZERO, size_of_duck.as_vec2())),
            ..default()
        },
        Transform::from_xyz(0., 0., 0.),
        RenderLayers::layer(1),
    ));

    let cam = commands
        .spawn((
            Camera2d,
            Camera {
                order: 1,
                target: RenderTarget::Image(camera_canvas.clone().into()),
                ..default()
            },
            Transform::from_scale(Vec2::splat(0.2).extend(1.)),
            RenderLayers::layer(1),
        ))
        .id();
    commands.spawn((
        Node {
            width: px(1000. * w),
            height: px(1000. * h),
            align_self: AlignSelf::Start,
            border: UiRect::all(px(3.)),
            ..default()
        },
        BorderColor::all(Color::from(Srgba::BLUE)),
        Pickable {
            should_block_lower: false,
            ..default()
        },
        ViewportNode { camera: cam },
        TextureAtlasWrapper(TextureAtlasLayout::from_grid(
            UVec2::splat(32),
            6,
            2,
            Some(UVec2::splat(1)),
            None,
        )),
        TextureAtlasPreviewNode,
    ));
}
fn rescale_texture_atlas_preview_camera(
    eq: Query<(&ViewportNode), With<TextureAtlasPreviewNode>>,
    mut q: Query<&mut Transform, (Without<MainEditorCamera>, With<Camera>)>,
) {
    eq.iter().for_each(|vp_node| {
        if let Ok(mut xform) = q.get_mut(vp_node.camera) {
            //xform.scale = Vec2::splat(0.2).extend(1.);
        }
    });
}

fn render_wrapper(
    mut commands: Commands,
    q: Query<&TextureAtlasWrapper, Changed<TextureAtlasWrapper>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for w in q.iter() {
        let sz = w.0.size.as_vec2();
        let offset = -0.5 * sz;
        for urect in w.0.textures.iter() {
            let rect = urect.as_rect();
            let center = rect.center();
            let [top, bottom, left, right] = [
                meshes.add(Rectangle::from_corners(
                    Vec2::new(0., 0.),
                    Vec2::new(rect.width(), 1.),
                )),
                meshes.add(Rectangle::from_corners(
                    Vec2::new(0., 0.),
                    Vec2::new(rect.width(), -1.),
                )),
                meshes.add(Rectangle::from_corners(
                    Vec2::new(0., 0.),
                    Vec2::new(1., rect.height()),
                )),
                meshes.add(Rectangle::from_corners(
                    Vec2::new(0., 0.),
                    Vec2::new(-1., rect.height()),
                )),
            ];
            let [top_offset, bottom_offset, left_offset, right_offset] = [
                Vec2::new(center.x, rect.max.y) + offset + (Vec2::NEG_Y * 0.5),
                Vec2::new(center.x, rect.min.y) + offset + (Vec2::Y * 0.5),
                Vec2::new(rect.min.x, center.y) + offset + (Vec2::X * 0.5),
                Vec2::new(rect.max.x, center.y) + offset + (Vec2::NEG_X * 0.5),
            ];
            let color = materials.add(Color::from(Srgba::GREEN.with_alpha(0.4)));

            commands.spawn((
                Mesh2d(top),
                MeshMaterial2d(color.clone()),
                RenderLayers::layer(1),
                Transform::from_translation((top_offset).extend(2.)),
            ));
            commands.spawn((
                Mesh2d(bottom),
                MeshMaterial2d(color.clone()),
                RenderLayers::layer(1),
                Transform::from_translation((bottom_offset).extend(2.)),
            ));
            commands.spawn((
                Mesh2d(left),
                MeshMaterial2d(color.clone()),
                RenderLayers::layer(1),
                Transform::from_translation((left_offset).extend(2.)),
            ));
            commands.spawn((
                Mesh2d(right),
                MeshMaterial2d(color.clone()),
                RenderLayers::layer(1),
                Transform::from_translation((right_offset).extend(2.)),
            ));
        }
    }
}
