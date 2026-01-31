use std::{any::TypeId, path::Path};

use ::bevy::prelude::*;
use bevy::{
    asset::io::file::FileAssetReader,
    ecs::{entity::EntityHashMap, reflect::ReflectCommandExt, world::DeferredWorld},
    platform::collections::HashSet,
};
use bevy_file_dialog::{EntityFileDialogExt, EntityScopedDialogEvent};

use crate::{
    EntityUiRoot,
    asset_extensions::{
        AssetServerSaveExtension, SerializationProxies,
        asset_tracking::{AssetLoadedEvent, load_and_watch},
    },
    ui_context_core::SelectedEntityUiRoot,
    widgets::add_entity_button::make_new_entity_ui,
};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(scene_save);
    app.add_observer(scene_load);
}

#[derive(Event)]
pub struct SaveScene(pub String);
#[derive(Event)]
pub struct LoadScene(pub String);

fn scene_save(src: On<SaveScene>, world: DeferredWorld) {
    let mut ui_roots = world
        .try_query::<&EntityUiRoot>()
        .expect("Failed to instantiate query");
    let type_registry = world.resource::<AppTypeRegistry>();
    let proxy_registry = world.resource::<SerializationProxies>();

    let mut tiny_world = World::new();
    tiny_world.insert_resource(type_registry.clone());
    let mut mapper = EntityHashMap::<Entity>::new();
    let mut aggregate: HashSet<TypeId> = HashSet::new();

    ui_roots.iter(&world).for_each(|ui_root| {
        let desired_serialization_components: HashSet<TypeId> = ui_root
            .desired_component_set
            .iter()
            .map(|t_id| {
                let serialization_proxy = proxy_registry.get_proxy(&t_id);
                *(serialization_proxy.unwrap_or(t_id))
            })
            .collect();
        info!(
            "Entity: {:?} with {:?} components",
            ui_root.component_holder,
            desired_serialization_components.len()
        );
        for x in desired_serialization_components.iter() {
            aggregate.insert(x.clone());
        }

        let smol_scene = DynamicSceneBuilder::from_world(&*world)
            .with_component_filter(SceneFilter::Allowlist(desired_serialization_components))
            .extract_entity(ui_root.component_holder)
            .build();
        let r =
            smol_scene.write_to_world_with(&mut tiny_world, &mut mapper, &type_registry.clone());
        if r.is_err() {
            info!("{:?}", r);
        }
    });
    let big_scene = DynamicSceneBuilder::from_world(&tiny_world)
        .with_component_filter(SceneFilter::Allowlist(aggregate))
        .deny_all_resources()
        .extract_entities(mapper.iter().map(|(_k, v)| *v))
        .build();

    for ent in big_scene.entities.iter() {
        info!(
            "Saving: Entity: {:?} with {:?} components",
            ent.entity,
            ent.components.len()
        );
    }
    AssetServer::save_dynamic_scene(Path::new(&src.0), type_registry, big_scene);
}

fn scene_bounce(
    src: On<AssetLoadedEvent<DynamicScene>>,
    scenes: Res<Assets<DynamicScene>>,
    proxies: Res<SerializationProxies>,
    find_anchor: Query<Entity, With<SelectedEntityUiRoot>>,
    mut commands: Commands,
) {
    let Ok(anchor) = find_anchor.single() else {
        return;
    };
    let dyn_scene = scenes
        .get(src.handle.id())
        .expect("Dynamic scene asset failed to exist");
    for dyn_entity in dyn_scene.entities.iter() {
        let world_target = commands.spawn_empty().id();
        let new_entity_ui = commands.spawn_empty().id();
        commands.entity(anchor).add_child(new_entity_ui);
        commands
            .entity(new_entity_ui)
            .insert(make_new_entity_ui(world_target));

        for box_component in dyn_entity.components.iter() {
            let og_tinfo = box_component
                .get_represented_type_info()
                .expect("Valid Type Info");
            let og_tid = og_tinfo.type_id().clone();
            let type_id = proxies.get_target(og_tinfo.type_path()).unwrap_or(&og_tid);

            commands.entity(world_target).insert_reflect(
                box_component
                    .reflect_clone()
                    .expect("Failed to clone component"),
            );
            commands.trigger(ComponentInstantiation {
                entity: new_entity_ui,
                type_id: type_id.clone(),
            });
        }
    }
    commands.entity(src.event_target()).despawn();
}

fn scene_load(src: On<LoadScene>, asset_server: Res<AssetServer>, mut commands: Commands) {
    let asset_bucket = commands.spawn_empty().id();
    commands.entity(asset_bucket).observe(scene_bounce);
    load_and_watch::<DynamicScene>(asset_bucket, &mut commands, &src.0, &*asset_server);
}

#[derive(EntityEvent, Clone)]
pub struct ComponentInstantiation {
    pub entity: Entity,
    pub type_id: TypeId,
}
pub fn save_scene_dialog(src: On<Pointer<Click>>, mut commands: Commands) {
    commands
        .entity(src.event_target())
        .with_dialog()
        .set_title("Save Scene")
        .set_directory("./sample_project_bin/assets/scenes")
        .add_filter("scene files", &["ron"])
        .save_file(Vec::new());
}
pub fn save_scene_with_path(event: On<EntityScopedDialogEvent>, mut commands: Commands) {
    info!("Saved! {:?}", event);
    use bevy_file_dialog::EntityScopedDialogResult::*;
    match event.clone().result {
        Save(file_save) => {
            let path = file_save
                .path
                .clone()
                .to_owned()
                .to_str()
                .unwrap()
                .to_owned();

            commands.trigger(SaveScene(path));
        }
        _ => {}
    };
}
pub fn load_scene_with_path(event: On<EntityScopedDialogEvent>, mut commands: Commands) {
    info!("Loaded! {:?}", event);
    use bevy_file_dialog::EntityScopedDialogResult::*;
    match event.clone().result {
        Pick(file_pick) => {
            let full_path = file_pick.path.clone().to_owned();
            //TODO - figure out how to do this without hard-coding this string.
            let root = FileAssetReader::get_base_path()
                .parent()
                .unwrap()
                .join("sample_project_bin")
                .join("assets");

            let fp_ = full_path.clone();
            let fp = fp_.to_string_lossy();
            let r_ = root.clone();
            let r = r_.to_string_lossy();

            let path = full_path
                .strip_prefix(root)
                .map_err(move |e| {
                    info!("Could not strip {r} path from {fp}",);
                    info!("{:?}", e.to_string());
                    e
                })
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned();
            commands.trigger(LoadScene(path));
        }
        _ => {}
    };
}

pub fn load_scene_dialog(src: On<Pointer<Click>>, mut commands: Commands) {
    commands
        .entity(src.event_target())
        .with_dialog()
        .set_title("Load Scene")
        .set_directory("./sample_project_bin/assets/scenes")
        .add_filter("scene files", &["ron"])
        .pick_file_path();
}
