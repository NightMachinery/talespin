# Hand pinning

Talespin/Dixit players can privately pin cards in their hand. Pins are only sent to the owning
player in stage-specific messages (`pinned_cards`) and are not included in shared room state.

Pinned cards are protected from:

- automatic round-start discards; and
- moderator **Refresh hands**.

Pinned cards are still normal playable cards. If a pinned card is chosen for the table, it leaves
the hand when voting begins and its pin is cleared.

Moderator round-start discard options:

- **Discard all unpinned cards at round start** is enabled by default. Each active player keeps
  pinned cards and discards every unpinned hand card, then draws back up to the hand size.
- If that option is disabled, the numeric discard count is used instead, but only unpinned cards
  are eligible. The count clamps per player when fewer unpinned cards are available.

Pins are also cleaned when cards leave a hand because of manual refresh, hand-size trimming, or
player/observer removal.
