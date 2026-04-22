# Stage timer sync

The shared stage countdown now resyncs immediately when the room changes stages.

## Behavior

- Entering any timed stage updates the client countdown without requiring a page refresh.
- Reconnect / reload mid-stage keeps the countdown aligned with the server deadline.
- Untimed stages (`Results`, `BeautyResults`, `StellaResults`, `Paused`, `End`) clear the shared countdown immediately.

## Implementation notes

- Stage-specific websocket payloads now carry the same `server_time_ms` and
  `current_stage_deadline_s` timer metadata that `RoomState` exposes.
- The stage shell seeds its local `performance.now()` sync baseline immediately on mount so a
  long-lived tab does not inherit a stale zero baseline when the UI remounts for a new stage.
