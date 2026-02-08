# User Identity and Persistence

## How the game knows who a player is after refresh

- It does **not** use cookies.
- The frontend stores the player's name in browser `localStorage` and reuses it after refresh.
- On page load (and on websocket reconnect), the client sends `JoinRoom { room_id, name }`.
- The backend matches by player name within the room:
  - if that name exists and is currently disconnected, it reattaches the player
  - if that name is already connected elsewhere, it rejects with `Name already taken`

## What is stored in localStorage

- Key: `name`
- Value: the player name string entered on the home page

What is **not** stored there:

- room code
- auth tokens
- card/game state

## Longevity

- Since this uses `localStorage`, it persists until the user clears site data (or private/incognito session ends).
- There is no cookie expiration involved because cookies are not used for this.

## Code References

- `src/lib/store.ts`
- `src/routes/game/[roomCode]/+page.svelte`
- `talespin-server/src/room.rs`
