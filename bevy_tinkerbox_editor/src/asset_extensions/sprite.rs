/// Utilities to better allow scene (de)serialization for scenes containing [Sprite]s.
use bevy::{asset::AssetPath, ecs::world::DeferredWorld, prelude::*};
use serde::Serialize;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<SpriteShadow>();
    app.add_observer(sprite_add_shadow);
    app.add_observer(shadow_add_sprite);
    app.add_systems(Update, sprite_stalker);
}

/// A serialization proxy for [Sprite].
///
/// Due to the dependence on multiple asset types, [Sprite] does not gracefully (de)serialize without some additional legwork.
/// We resolve that here by defining a more serialization friendly representation for some fields. Work remains to generalize this concept to
/// other asset types.
///
#[derive(Clone, Component, Reflect, Serialize)]
#[reflect(Component, Serialize, Default)]
pub struct SpriteShadow {
    /// The path to the image for this sprite.
    pub image: AssetPath<'static>,
    /// We bypass the asset store for our texture atlas. This unfortunately prevents serialized sprites from
    /// sharing TextureAtlas handles.
    pub texture_atlas: Option<(usize, TextureAtlasLayout)>,
    /// As in [Sprite]
    pub color: Color,
    /// As in [Sprite]
    pub flip_x: bool,
    /// As in [Sprite]
    pub flip_y: bool,
    /// As in [Sprite]
    pub custom_size: Option<Vec2>,
    /// As in [Sprite]
    pub rect: Option<Rect>,
}
impl SpriteShadow {
    fn from_sprite(sprite: &Sprite, texture_atlas_layouts: &Assets<TextureAtlasLayout>) -> Self {
        let path = match sprite.image.path() {
            Some(x) => x.clone_owned(),
            None => AssetPath::from(""),
        };
        SpriteShadow {
            image: path,
            texture_atlas: sprite.texture_atlas.clone().map(|tal| {
                let layout = texture_atlas_layouts.get(tal.layout.id()).unwrap();
                (tal.index, layout.clone())
            }),
            color: sprite.color,
            flip_x: sprite.flip_x,
            flip_y: sprite.flip_y,
            rect: sprite.rect.clone(),
            custom_size: sprite.custom_size,
        }
    }
    fn into_sprite(
        &self,
        texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
        asset_server: &AssetServer,
    ) -> Sprite {
        let texture_atlas = self.texture_atlas.as_ref().map(|(i, tal)| {
            let layout = texture_atlas_layouts.add(tal.clone());
            TextureAtlas { layout, index: *i }
        });
        let image = asset_server.load(self.image.clone());
        Sprite {
            image: image,
            texture_atlas: texture_atlas,
            color: self.color,
            flip_x: self.flip_x,
            flip_y: self.flip_y,
            custom_size: self.custom_size.clone(),
            rect: self.rect.clone(),
            image_mode: Sprite::default().image_mode,
        }
    }
}
impl Default for SpriteShadow {
    fn default() -> Self {
        Self {
            image: Default::default(),
            texture_atlas: Default::default(),
            color: Default::default(),
            flip_x: Default::default(),
            flip_y: Default::default(),
            custom_size: Default::default(),
            rect: Default::default(),
        }
    }
}

fn sprite_add_shadow(src: On<Add, Sprite>, world: DeferredWorld, mut commands: Commands) {
    let entity = src.entity;
    let sprite = world
        .get::<Sprite>(entity.clone())
        .expect("sprite doesn't exist on sprite add")
        .clone();
    let texture_atlas_layouts = world.resource::<Assets<TextureAtlasLayout>>();
    let shadow = SpriteShadow::from_sprite(&sprite, &*texture_atlas_layouts);
    commands.entity(entity).insert(shadow);
}
fn shadow_add_sprite(src: On<Add, SpriteShadow>, mut world: DeferredWorld, mut commands: Commands) {
    let entity = src.entity;
    let sprite_shadow = world
        .get::<SpriteShadow>(entity.clone())
        .expect("sprite shadow doesn't exist on sprite shadow add")
        .clone();
    let asset_server = world.resource::<AssetServer>().clone();
    let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();
    let sprite = sprite_shadow.into_sprite(&mut *texture_atlas_layouts, &asset_server);
    commands.entity(entity).insert(sprite);
}
fn sprite_stalker(
    mut watched_sprites: Query<(&Sprite, &mut SpriteShadow), Changed<Sprite>>,
    texture_atlas_layouts: Res<Assets<TextureAtlasLayout>>,
) {
    for (sprite, mut shadow) in watched_sprites.iter_mut() {
        shadow.image = match sprite.image.path() {
            Some(x) => x.clone_owned(),
            None => AssetPath::from(""),
        };
        shadow.texture_atlas = sprite.texture_atlas.clone().map(|tal| {
            let layout = texture_atlas_layouts.get(tal.layout.id()).unwrap();
            (tal.index, layout.clone())
        });
        shadow.color = sprite.color;
        shadow.flip_x = sprite.flip_x;
        shadow.flip_y = sprite.flip_y;
        shadow.custom_size = sprite.custom_size.clone();
        shadow.rect = sprite.rect.clone();
    }
}
