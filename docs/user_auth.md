# User Identity and Roles

## Identity after refresh

- The game does **not** use cookies.
- The frontend stores only the player name in browser `localStorage`.
- On load/reconnect, client sends `JoinRoom { room_id, name }`.
- Backend maps identity by `name` inside the room:
  - if same name exists and is disconnected, it reconnects that player
  - if same name is already connected, server rejects with `Name already taken`

## localStorage contents

- Key: `name`
- Value: player name string

Not stored:

- room code
- auth/session tokens
- game state or cards

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
