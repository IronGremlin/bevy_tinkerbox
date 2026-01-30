use bevy::platform::collections::HashMap;
use bevy::prelude::*;

use bevy::scene::serde::SceneSerializer;
use bevy::tasks::IoTaskPool;
use ron;

use std::any::TypeId;
use std::path::Path;
use std::{fs::File, io::Write};

use crate::asset_extensions::sprite::SpriteShadow;
pub mod asset_tracking;
pub mod editor_assets;
pub mod sprite;
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<SerializationProxies>();
    app.add_plugins((asset_tracking::plugin, sprite::plugin));
}

pub trait AssetServerSaveExtension {
    fn save_dynamic_scene(path: &Path, type_registry: &AppTypeRegistry, asset: DynamicScene);
}
impl AssetServerSaveExtension for AssetServer {
    fn save_dynamic_scene(path: &Path, type_registry: &AppTypeRegistry, asset: DynamicScene) {
        let path = path.to_owned();
        let guard = type_registry.read();
        let scene_serializer = SceneSerializer::new(&asset, &guard);

        let serialized_scene = ron::ser::to_string(&scene_serializer).unwrap();
        IoTaskPool::get()
            .spawn(async move {
                File::create(path)
                    .and_then(|mut file| file.write(serialized_scene.as_bytes()))
                    .expect("Error while writing scene to file");
            })
            .detach();
    }
}

#[derive(Resource)]
pub struct SerializationProxies {
    proxies_by_target: HashMap<TypeId, TypeId>,
    targets_by_proxy: HashMap<String, TypeId>,
}
impl SerializationProxies {
    pub fn get_proxy(&self, k: &TypeId) -> Option<&TypeId> {
        self.proxies_by_target.get(k)
    }
    pub fn get_target(&self, k: &str) -> Option<&TypeId> {
        self.targets_by_proxy.get(k)
    }
    pub fn register<Target, Proxy>(&mut self)
    where
        Target: ?Sized + 'static,
        Proxy: TypePath + ?Sized + 'static,
    {
        self.proxies_by_target
            .insert(TypeId::of::<Target>(), TypeId::of::<Proxy>());
        self.targets_by_proxy
            .insert(Proxy::type_path().to_owned(), TypeId::of::<Target>());
    }
}
impl Default for SerializationProxies {
    fn default() -> Self {
        let mut new_self = Self {
            proxies_by_target: HashMap::new(),
            targets_by_proxy: HashMap::new(),
        };
        new_self.register::<Sprite, SpriteShadow>();
        new_self
    }
}
