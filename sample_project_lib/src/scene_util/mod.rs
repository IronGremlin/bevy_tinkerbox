#![allow(dead_code)]
use bevy::ecs::entity::EntityHashSet;
use bevy::ecs::entity_disabling::Disabled;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Warned>();
    app.add_systems(Update, required_transform_monitor);
}
/// A flag indicating an expectation of author-validated transform data.
///
/// This is used when serializing scene data for some templated entity.
/// When included, an error (in production) will be logged, or the application will panic (in dev) unless a command is issued during that frame to supply fresh transform data.
///
/// This helps ensure that templated entities are supplied with a valid initial transform in the frame in which they are spawned.
///
#[derive(Component, Reflect, Default, Clone)]
#[require(Transform = RequiresTransform::placeholder(), Disabled)]
pub struct RequiresTransform {
    data: Option<Transform>,
}
impl RequiresTransform {
    // This is a likely unnecessary degree of safety, but in the event we've done something stupid, this defaults our transforms below any scene backgrounds etc,
    // which should hopefully hide our shame.
    fn placeholder() -> Transform {
        Transform::from_xyz(0.0, 0.0, -10000.0)
    }
    /// The function to call to supply validated transform data.
    ///
    /// The expectation is that a user will accomplish this using [EntityCommands::entry].
    /// Example:
    /// ```
    /// # use bevy::prelude::*;
    /// # fn some_test_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    /// commands
    ///    .spawn(SceneRoot(asset_server.load("scenes/some_scene.ron")))
    ///    .entry::<RequiresTransform>()
    ///    .and_modify(|mut rt| {
    ///        rt.initialize(Transform::from_xyz(0., 0., 1.));
    ///    });
    /// # }
    /// ```
    pub fn initialize(&mut self, transform: Transform) {
        self.data = Some(transform);
    }
}
#[derive(Resource, Default)]
struct Warned(EntityHashSet);

fn required_transform_monitor(
    #[cfg(feature = "dev")] mut warnings_issued: ResMut<Warned>,
    might_need_transform: Query<(Entity, &Transform, &RequiresTransform), With<Disabled>>,
    mut commands: Commands,
) {
    for (entity, t, req_t) in might_need_transform.iter() {
        if req_t.data.is_some() {
            let xform = req_t.data.unwrap();
            commands.entity(entity).remove::<RequiresTransform>();
            commands.entity(entity).remove::<Disabled>();

            commands.entity(entity).insert(xform);
            #[cfg(feature = "dev")]
            warnings_issued.0.remove(&entity);
        } else {
            if !warnings_issued.0.contains(&entity) {
                #[cfg(not(feature = "dev"))]
                {
                    warn!(
                        "Entity: {:?} has not signalled valid Transform data: {:?}; remains disabled.",
                        entity, t
                    );
                    warnings_issued.0.insert(entity);
                }
                #[cfg(feature = "dev")]
                panic!(
                    "Entity: {:?} has not signalled valid Transform data: {:?}; remains disabled.",
                    entity, t
                );
            }
        }
    }
}
