# Fullscreen option

The game `Options` card includes a local fullscreen toggle for browsers that support the
Fullscreen API.

- The button is available to all users from `SidebarOptions.svelte`, not from the mobile sticky
  action panel.
- It requests fullscreen on `document.documentElement` so browser chrome and tab bars can be hidden
  on small screens when the browser allows it.
- The button follows the live `fullscreenchange` event, so the label updates when fullscreen is
  exited through browser UI.
- Fullscreen state is not persisted. It is browser/session state only.
