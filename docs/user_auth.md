# User Identity and Roles

## Identity after refresh

- The game does **not** use cookies.
- The frontend stores:
  - preferred player name in browser `localStorage`
  - stable `player_token` in browser `localStorage`
  - room-scoped assigned duplicate names in browser `localStorage`
- The backend also issues a stable **room-scoped** opaque `room_auth_id` for each current room member:
  - used by **Migrate Device** links
  - stable across refresh/reconnect for that room membership
  - **not** the real `player_token`
  - invalidated when that member explicitly leaves, gets kicked, or is otherwise removed
- On load/reconnect, client sends `JoinRoom { room_id, name, token }`.
- Normal joins send the browser `player_token`; device-migration joins send a blank `name` plus the room's
  opaque `room_auth_id` in the `token` slot so the backend can resolve the existing member.
- Backend maps identity by token+assigned room name inside the room:
  - same token reconnects to its existing room identity
  - different token using a taken name is auto-renamed to `Name 2`, `Name 3`, etc.
  - assigned duplicate names are remembered per room only, not as the user's global preferred name
- If the page URL includes `room_auth_id`, that URL override wins over browser `localStorage` identity for that room page load and refresh.
- When `room_auth_id` is used:
  - client joins with the migrated room identity instead of the browser's saved `player_token`
  - browser `localStorage` identity is **not** replaced
  - room-assigned-name local storage is **not** read/written/cleared for that migrated session
  - an invalid/stale `room_auth_id` hard-fails instead of falling back to the browser's saved identity

## localStorage contents

- Key: `name`
- Value: preferred/base player name string
- Key: `player_token`
- Value: stable auth token string
- Key: `room_assigned_name:<room_code>`
- Value: JSON with `{ token, name }` for room-local duplicate-name reconnects

Not stored:

- game state or cards
- `room_auth_id`
- room passwords from migration links

`room_assigned_name:<room_code>` is cleared when the user explicitly leaves, gets kicked, or exits an invalid room flow.

## Migrate Device links

- **Migrate Device** copies a direct game URL:
  - `/game/<room_code>?room_auth_id=<opaque_id>`
  - passworded rooms additionally include `&room_password=<password>`
- The migration URL keeps those query params so refresh continues to use the migrated identity.
- `room_password` from the URL is page-scoped input only:
  - used for that room page load / refresh
  - not persisted to `localStorage` or `sessionStorage`
- Regular invite links remain room-only; migration links are the only links that carry auth + password.
- Because migration links include the room password in the URL when present, treat them as sensitive.

## Creator and moderator model

- `creator` is fixed as the room creator identity.
- `moderators` is a separate privilege group.
- Creator is always a moderator while present in the room.
- Only creator can grant/revoke moderator status.
- Moderators can:
  - start the game from lobby
  - kick non-creator players from lobby
- If no moderator is connected for 5 minutes, server auto-promotes one random connected player to moderator.

## Leave/kick behavior

- Voluntary leave sends `LeaveRoom`:
  - player is removed from room membership
  - their in-hand/selected cards are moved to discard
- Kick sends `Kicked` to target and removes them similarly.

## Longevity

- `localStorage` persists until browser data is cleared (or private session ends).
- No cookie expiry exists, because cookies are not used.

## References

- `src/lib/deviceMigration.ts`
- `src/lib/MigrateDeviceButton.svelte`
- `src/lib/store.ts`
- `src/routes/game/[roomCode]/+page.svelte`
- `src/routes/game/[roomCode]/Joining.svelte`
- `src/lib/gameServer.ts`
- `talespin-server/src/room.rs`
