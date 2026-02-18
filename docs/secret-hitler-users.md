# Secret Hitler: User/Roles Dynamics Spec (Any-Time Join + Creator/Mods + Observer)

## Purpose

This document defines **product behavior** for adding Talespin-style room/user dynamics to a Secret Hitler implementation, even if it uses a different stack or protocol.

Focus areas:

- Any-time join
- Creator + moderator governance
- Observer mode + join-back flows
- Disconnect/reconnect identity continuity
- Pause/resume when active player count is too low

---

## Scope

In scope:

- Room membership lifecycle
- Role permissions
- Join/observe transitions across game phases
- UX and acceptance behavior

Out of scope:

- Secret Hitler policy/election rule changes
- Transport/framework specifics

---

## Core concepts

### Membership states

- **Active player**: currently part of the playable roster.
- **Observer**: in room, not part of active roster.
- **Disconnected member**: still a member (active or observer), but offline.
- **Removed member**: explicitly left/kicked and no longer in room.

### Roles

- **Creator**: room host identity established at room creation.
- **Moderator**: elevated room-management privileges.

### Phase classes

To keep this portable, map your game phases into:

- **Safe phase**: adding/removing active players is allowed immediately.
- **Atomic phase**: active roster must not change until next round boundary.

Recommended Secret Hitler mapping:

- **Safe**: Lobby/Setup, Paused.
- **Atomic**: election in progress, legislative enactment in progress, executive action in progress, end-of-round resolution in progress.

---

## Global invariants

1. Creator is a single identity for the room.
2. Creator is always a moderator while present in the room.
3. Moderators must be current room members (active or observer).
4. Observers are room members but do not count as active players.
5. Active-player minimum is enforced for playable state (default recommended: **5** for Secret Hitler).

---

## Any-time join behavior

### New member joins room

- If joins are allowed:
  - **Safe phase**: join as active player immediately.
  - **Atomic phase**: join as observer, queued to auto-join on next round by default.
- If joins are disabled midgame: reject new members with clear error.
- Rejoin (same identity) bypasses “new member” checks and restores prior member state.

### Midgame join control

- Moderators can toggle “Allow new players to join.”
- Toggle affects only **new** members; existing members can reconnect.

---

## Observer lifecycle

### Become observer

- Self: member can switch self to observer.
- Moderator: can switch any member to observer.
- Block if target currently holds a locked, in-flight round responsibility and switching would break round integrity.

### Observer join-back

- If current phase is **Safe**: “Join now” promotes observer to active player immediately.
- If current phase is **Atomic**: observer uses “Join next round” (toggle-able; second click cancels).

### Moderator action on observers

- Moderator can trigger same join behavior for any observer:
  - Immediate in Safe phase
  - Queue/cancel in Atomic phase

---

## Creator/moderator governance

### Permissions

- Moderators can:
  - Start game
  - Resume from paused state
  - Kick members
  - Toggle observer status for others
  - Toggle midgame join setting
- Creator + moderators can promote moderators.
- **Only creator** can demote moderators.
- Creator cannot be demoted while present.

---

## Moderator continuity (host migration)

- If no connected moderator exists for 5 minutes:
  - Auto-promote one random connected member (active or observer) to moderator.
- If at least one moderator reconnects before timeout, clear timer.
- Disconnected moderators keep their role while still members.

---

## Identity and reconnect

- Identity must include a stable client token plus display name.
- Reconnect with same identity restores prior membership state (active vs observer).
- If name is already claimed by a different token, reject as “name already taken.”
- If same identity reconnects from another session, either reject older session or supersede it consistently (pick one policy and keep it consistent).

---

## Leave and kick semantics

- Leave/kick removes member from room membership.
- Remove role grants tied to membership (including moderator).
- If removed member was creator, room may continue without creator.
- Show personal feedback message:
  - Left room
  - Kicked from room

---

## Pause/resume behavior

- If active players drop below minimum playable count:
  - Transition room to **Paused**
  - Show clear paused reason (“Need at least N active players…”)
- While paused:
  - New joins are processed by safe-phase rules.
  - Observer “Join now” should promote immediately.
- Resume:
  - Moderator-only action
  - Allowed only once minimum active player count is restored

---

## UX acceptance requirements

### Labels and presence

- Show badges: `(creator)`, `(mod)`, `(observer)`, `(offline)`.
- Show active players and observers separately.

### Action visibility/disabled states

- Hide/disable controls user cannot execute.
- When disabled, show short reason.
- Use clear, user-level errors for rejected actions.

### Observer controls

- Observer sees:
  - `Join now` in Safe phase
  - `Join next round` / `Cancel pending join` in Atomic phase
- Active member sees `Become observer` when allowed.

---

## Implementation checklist for coding agent

1. Add room membership model (active/observer/disconnected/removed).
2. Add creator/moderator role model with invariants.
3. Add per-phase classifier: Safe vs Atomic.
4. Implement join resolver:
   - New join safe -> active
   - New join atomic -> observer queued
   - Respect “allow new joins” toggle
5. Implement observer conversion and join-back (immediate vs queued).
6. Add permission guards for all moderator/creator actions.
7. Add no-moderator timeout + auto-promotion job (5 minutes).
8. Add paused state + resume guard on minimum active players.
9. Add reconnect identity flow using stable token + name.
10. Add UI labels/actions/disabled reasons and user-facing toasts/errors.

---

## Test scenarios

1. Creator is always moderator while present.
2. Non-creator moderator cannot demote another moderator.
3. Creator cannot be demoted.
4. Moderator can kick non-self member.
5. Safe-phase new join becomes active immediately.
6. Atomic-phase new join becomes observer with pending next-round join.
7. Observer join-back in safe phase is immediate.
8. Observer join-back in atomic phase toggles pending state.
9. Pending observer join can be canceled.
10. Midgame join toggle blocks new members but allows reconnecting existing members.
11. Disconnect does not remove membership or moderator role.
12. Reconnect with same identity restores previous state.
13. Reconnect with same name but wrong token is rejected.
14. No connected moderator for 5 minutes triggers auto-promotion.
15. Dropping below minimum active count enters paused state.
16. Resume denied when active count still below minimum.
17. Resume allowed for moderator when minimum restored.
18. Paused state still allows observer immediate join-back.

---

## Defaults and assumptions

- Default minimum active players for Secret Hitler: **5** (configurable).
- Moderator absence auto-promotion delay: **5 minutes**.
- Candidate pool for auto-promotion includes connected active players and connected observers.
- Any-time join parity means behavior parity, not protocol/schema parity.
