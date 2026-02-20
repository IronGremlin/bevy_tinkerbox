use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};

#[derive(Component, Clone)]
#[component(on_add = image_node_sans_handle_added)]
pub struct IconImage {
    pub color: Color,
    pub path_to_image: String,
    pub texture_atlas: Option<TextureAtlas>,
    pub flip_x: bool,
    pub flip_y: bool,
    pub rect: Option<Rect>,
    pub image_mode: NodeImageMode,
}
impl From<ImageNode> for IconImage {
    fn from(value: ImageNode) -> Self {
        let ImageNode {
            color,
            image,
            texture_atlas,
            flip_x,
            flip_y,
            rect,
            image_mode,
        } = value;
        IconImage {
            color,
            path_to_image: image.path().unwrap().to_string(),
            texture_atlas,
            flip_x,
            flip_y,
            rect,
            image_mode,
        }
    }
}
impl Default for IconImage {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            texture_atlas: None,
            path_to_image: "".to_owned(),
            flip_x: false,
            flip_y: false,
            rect: None,
            image_mode: NodeImageMode::Auto,
        }
    }
}
impl IconImage {
    pub fn given_world(&self, world: &DeferredWorld) -> ImageNode {
        let assets = world.resource::<AssetServer>();
        let IconImage {
            color,
            path_to_image,
            texture_atlas,
            flip_x,
            flip_y,
            rect,
            image_mode,
        } = self.clone();
        ImageNode {
            color,
            image: assets.load(path_to_image),
            texture_atlas,
            flip_x,
            flip_y,
            rect,
            image_mode,
        }
    }
    pub fn from_path(path: String) -> Self {
        Self {
            path_to_image: path,
            ..default()
        }
    }
}
fn image_node_sans_handle_added(mut world: DeferredWorld, context: HookContext) {
    let val = world.get::<IconImage>(context.entity).unwrap().clone();
    let new = val.given_world(&world);
    let mut commands = world.commands();
    commands.entity(context.entity).insert(new);
    commands.entity(context.entity).remove::<IconImage>();
}
