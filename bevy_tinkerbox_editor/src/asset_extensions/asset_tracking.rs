//! A high-level way to load collections of asset handles as resources.
//!
//! Outright robbed from bevy_cli's new_2d template implementation
use std::collections::VecDeque;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<ResourceHandles>();
    app.add_systems(PreUpdate, (asset_watcher, load_resource_assets));
}

#[allow(unused)]
pub trait LoadResource {
    /// This will load the [`Resource`] as an [`Asset`]. When all of its asset dependencies
    /// have been loaded, it will be inserted as a resource. This ensures that the resource only
    /// exists when the assets are ready.
    fn load_resource<T: Resource + Asset + Clone + FromWorld>(&mut self) -> &mut Self;
}

impl LoadResource for App {
    fn load_resource<T: Resource + Asset + Clone + FromWorld>(&mut self) -> &mut Self {
        self.init_asset::<T>();
        let world = self.world_mut();
        let value = T::from_world(world);
        let assets = world.resource::<AssetServer>();
        let handle = assets.add(value);
        let mut handles = world.resource_mut::<ResourceHandles>();
        handles
            .waiting
            .push_back((handle.untyped(), |world, handle| {
                let assets = world.resource::<Assets<T>>();
                if let Some(value) = assets.get(handle.id().typed::<T>()) {
                    world.insert_resource(value.clone());
                }
            }));
        self
    }
}

// Note, this design is limited to one asset pending per entity.
//
// Even if we were to queue the untyped assets, this design fundamentally
// exposes a race condition between multiple asset loads of the same type per entity,
// and it also may leak observers.
//
// So we -assume- that the calling context uses disposable entities to watch and wait for these
// handles, and stores further information about what we're loading these assets for in those one-shot containers.

#[derive(Component)]
struct PendingAsset(pub UntypedHandle);

#[derive(EntityEvent, Clone)]
#[allow(unused)]
struct TypeErasedAssetLoadedEvent {
    entity: Entity,
    handle: UntypedHandle,
}

#[derive(EntityEvent, Clone)]
pub struct AssetLoadedEvent<A: Asset> {
    pub entity: Entity,
    pub handle: Handle<A>,
}

fn asset_watcher(
    asset_server: Res<AssetServer>,
    pending_assets: Query<(Entity, &PendingAsset)>,
    mut commands: Commands,
) {
    pending_assets.iter().for_each(|(entity, handle)| {
        if asset_server.is_loaded_with_dependencies(handle.0.id()) {
            commands.trigger(TypeErasedAssetLoadedEvent {
                entity,
                handle: handle.0.clone(),
            });
        }
    });
}

pub fn load_and_watch<A: Asset>(
    entity: Entity,
    commands: &mut Commands,
    asset_path: &str,
    asset_server: &AssetServer,
) {
    let handle = asset_server.load::<A>(asset_path.to_owned()).untyped();

    commands.entity(entity).observe(
        |src: On<TypeErasedAssetLoadedEvent>,
         q: Query<&PendingAsset>,
         mut sub_commands: Commands| {
            let Ok(untyped_handle) = q.get(src.event_target()) else {
                return;
            };
            let Ok(typed_handle) = untyped_handle.0.clone().try_typed::<A>() else {
                return;
            };
            sub_commands.trigger(AssetLoadedEvent::<A> {
                entity: src.event_target(),
                handle: typed_handle,
            });
            sub_commands
                .entity(src.event_target())
                .remove::<PendingAsset>();
        },
    );
    commands.entity(entity).insert(PendingAsset(handle));
}

/// A function that inserts a loaded resource.
type InsertLoadedResource = fn(&mut World, &UntypedHandle);

#[derive(Resource, Default)]
pub struct ResourceHandles {
    // Use a queue for waiting assets so they can be cycled through and moved to
    // `finished` one at a time.
    waiting: VecDeque<(UntypedHandle, InsertLoadedResource)>,
    finished: Vec<UntypedHandle>,
}

impl ResourceHandles {
    /// Returns true if all requested [`Asset`]s have finished loading and are available as [`Resource`]s.
    pub fn is_all_done(&self) -> bool {
        self.waiting.is_empty()
    }
}

fn load_resource_assets(world: &mut World) {
    world.resource_scope(|world, mut resource_handles: Mut<ResourceHandles>| {
        world.resource_scope(|world, assets: Mut<AssetServer>| {
            for _ in 0..resource_handles.waiting.len() {
                let (handle, insert_fn) = resource_handles.waiting.pop_front().unwrap();
                if assets.is_loaded_with_dependencies(&handle) {
                    insert_fn(world, &handle);
                    resource_handles.finished.push(handle);
                } else {
                    resource_handles.waiting.push_back((handle, insert_fn));
                }
            }
        });
    });
}
