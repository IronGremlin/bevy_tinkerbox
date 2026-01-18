use ::bevy::prelude::*;

#[derive(Component)]
pub struct DragSnapState2d {
    pub origin: Vec2,
    pub last_seen: Vec2,
}

#[derive(Component, Clone)]
pub struct WorldSnap2dGrid {
    pub camera: Entity,
}

#[derive(Debug, Reflect, Clone)]
pub struct SnapDrag {
    pub origin: Vec2,
    pub last_seen: Vec2,
    pub button: PointerButton,
}

#[derive(Debug, Reflect, Clone)]
pub struct SnapDragStart {
    pub origin: Vec2,
    pub button: PointerButton,
}

#[derive(Debug, Reflect, Clone)]
pub struct SnapDragEnd {
    pub origin: Vec2,
    pub last_seen: Vec2,
    pub button: PointerButton,
}

pub(super) fn plugin(app: &mut App) {
    app.add_observer(drag_start_snap_watcher);
    app.add_observer(drag_held_snap_watcher);
    app.add_observer(drag_end_snap_watcher);
}

pub(crate) fn drag_held_snap_watcher(
    src: On<Pointer<Drag>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut commands: Commands,
    mut snap_states: Query<(&mut DragSnapState2d, &WorldSnap2dGrid)>,
) {
    let source_entity = src.event_target();
    if let Ok((mut snap_state, world_grid)) = snap_states.get_mut(source_entity) {
        let Ok((cam, xform)) = cameras.get(world_grid.camera) else {
            return;
        };
        let Ok(cursor_pos) = cam.viewport_to_world_2d(xform, src.pointer_location.position) else {
            return;
        };
        let trunced = cursor_pos.trunc();
        if snap_state.last_seen != trunced {
            snap_state.last_seen = trunced;
            commands.trigger(Pointer::<SnapDrag> {
                entity: source_entity,
                pointer_id: src.pointer_id,
                pointer_location: src.pointer_location.clone(),
                event: SnapDrag {
                    origin: snap_state.origin,
                    last_seen: snap_state.last_seen,
                    button: src.button,
                },
            });
        }
    }
}

pub(crate) fn drag_start_snap_watcher(
    src: On<Pointer<DragStart>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut commands: Commands,
    snap_states: Query<&WorldSnap2dGrid>,
) {
    let source_entity = src.event_target();
    if let Ok(world_grid) = snap_states.get(source_entity) {
        let Ok((cam, xform)) = cameras.get(world_grid.camera) else {
            return;
        };
        let Ok(cursor_pos) = cam.viewport_to_world_2d(xform, src.pointer_location.position) else {
            return;
        };
        let trunced = cursor_pos.trunc();
        commands.entity(source_entity).insert(DragSnapState2d {
            origin: trunced,
            last_seen: trunced,
        });

        commands.trigger(Pointer::<SnapDragStart> {
            entity: source_entity,
            pointer_id: src.pointer_id,
            pointer_location: src.pointer_location.clone(),
            event: SnapDragStart {
                origin: trunced,
                button: src.button,
            },
        });
    }
}

pub(crate) fn drag_end_snap_watcher(
    src: On<Pointer<DragEnd>>,
    mut commands: Commands,
    snap_states: Query<&DragSnapState2d>,
) {
    let source_entity = src.event_target();
    if let Ok(snap_state) = snap_states.get(source_entity) {
        commands.entity(source_entity).remove::<DragSnapState2d>();

        commands.trigger(Pointer::<SnapDragEnd> {
            entity: source_entity,
            pointer_id: src.pointer_id,
            pointer_location: src.pointer_location.clone(),
            event: SnapDragEnd {
                origin: snap_state.origin,
                last_seen: snap_state.last_seen,
                button: src.button,
            },
        });
    }
}
