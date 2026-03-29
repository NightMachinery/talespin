# User Identity and Roles

## Identity after refresh

- The game does **not** use cookies.
- The frontend stores:
  - preferred player name in browser `localStorage`
  - stable `player_token` in browser `localStorage`
  - room-scoped assigned duplicate names in browser `localStorage`
- On load/reconnect, client sends `JoinRoom { room_id, name, token }`.
- Backend maps identity by token+assigned room name inside the room:
  - same token reconnects to its existing room identity
  - different token using a taken name is auto-renamed to `Name 2`, `Name 3`, etc.
  - assigned duplicate names are remembered per room only, not as the user's global preferred name

## localStorage contents

- Key: `name`
- Value: preferred/base player name string
- Key: `player_token`
- Value: stable auth token string
- Key: `room_assigned_name:<room_code>`
- Value: JSON with `{ token, name }` for room-local duplicate-name reconnects

Not stored:

- game state or cards

`room_assigned_name:<room_code>` is cleared when the user explicitly leaves, gets kicked, or exits an invalid room flow.

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

- `src/lib/store.ts`
- `src/routes/game/[roomCode]/+page.svelte`
- `src/routes/game/[roomCode]/Joining.svelte`
- `src/lib/gameServer.ts`
- `talespin-server/src/room.rs`
