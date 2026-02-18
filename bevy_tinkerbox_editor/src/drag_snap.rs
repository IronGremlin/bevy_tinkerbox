/// Module which supports 'snap' behaviors for drag events.
///
use ::bevy::prelude::*;

/// Component holding history for a dragged entity with 2d snaping enabled.
#[derive(Component)]
pub struct DragSnapState2d {
    /// The point in 2d worldspace where the cursor was hovering when drag began
    pub origin: Vec2,
    /// The point in 2d worldspace where the cursor was last observed (either in this frame or the one immediately preceeding it).
    pub last_seen: Vec2,
}

/// Component which designates an entity as supporting snapped drag events, where each snapped drag event is emitted
/// for 1.0 length axis-aligned segments in world-space.
#[derive(Component, Clone)]
pub struct WorldSnap2dGrid {
    /// Camera used to map cursor position to world-space co-ordinates.
    pub camera: Entity,
}

/// Extended Pointer event for a snapped drag.
///
/// Represents an in-progress drag interaction.
///
/// Emitted when the cursor passes past a snap threshold.
///
/// Unlike normal drag events this will not be emitted for each frame the input is held down,
/// it will be emitted once whenever a new snap point is closest to the cursor.
///
/// Emitted Co-ordinates will be those of the snapped point, not necessarily those of the cursor.
///
#[derive(Debug, Reflect, Clone)]
pub struct SnapDrag {
    /// Snapped world-space co-ordinate where the snapped-drag began.
    pub origin: Vec2,
    /// The world-space co-ordinate where the cursor was observed when this event was triggered.
    pub last_seen: Vec2,
    /// The button depressed when this event was triggered.
    pub button: PointerButton,
}

/// Extended Pointer event for a snapped drag.
///
/// Represents the origin snap point at the beginning of a snapped drag.
///
#[derive(Debug, Reflect, Clone)]
pub struct SnapDragStart {
    /// Snapped world-space co-ordinate where the snapped-drag began.
    pub origin: Vec2,
    /// The button depressed when this event was triggered.
    pub button: PointerButton,
}

/// Extended Pointer event for a snapped drag.
///
/// Represents the closest snap point at the end of a snapped drag.
///
#[derive(Debug, Reflect, Clone)]
pub struct SnapDragEnd {
    /// Snapped world-space co-ordinate where the snapped-drag began.
    pub origin: Vec2,
    /// The world-space co-ordinate where the cursor was observed when this event was triggered.
    pub last_seen: Vec2,
    /// The button depressed when this event was triggered.
    pub button: PointerButton,
}
// TODO - this should probably actually be its own plugin struct.
pub(super) fn plugin(app: &mut App) {
    app.add_observer(drag_start_snap_watcher);
    app.add_observer(drag_held_snap_watcher);
    app.add_observer(drag_end_snap_watcher);
}
/// Observer function supporting the behavior for snapped 2d grid drag events.
///
/// This function "filter-maps" Drag events to [SnapDrag] events when the cursor passes a snap threshold
/// while dragging. See [SnapDrag] for more details.
///
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
/// Observer function supporting the behavior for snapped 2d grid drag events.
///
/// This function maps [DragStart] to [SnapDragStart]. See [SnapDragStart] for more detail.
///
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
/// Observer function supporting the behavior for snapped 2d grid drag events.
///
/// This function maps [DragEnd] to [SnapDragEnd]. See [SnapDragEnd] for more detail.
///
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
