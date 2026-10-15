# Herdr API feasibility

Investigated 2026-10-14 against installed Herdr 0.9.5 (protocol 22), the bundled `herdr api schema --json`, the live read-only `herdr api snapshot` response, and upstream source at commit `916fcf8de029c6cac965b50f3cdb9687cd7d1537`. This document records API constraints and plugin implementation choices; the validation boundary below describes current automated coverage.

## Required documentation and examples

- [Plugin lifecycle, manifest, environment, storage, and keybindings](https://herdr.dev/docs/plugins/)
- [CLI reference](https://herdr.dev/docs/cli-reference/)
- [Socket API and session.snapshot](https://herdr.dev/docs/socket-api/)
- [Configuration](https://herdr.dev/docs/configuration/)
- [Config reference](https://herdr.dev/docs/config-reference/)
- [Author-provided plugin examples](https://github.com/ogulcancelik/herdr-plugin-examples), especially [the Rust manifest](https://github.com/ogulcancelik/herdr-plugin-examples/blob/main/rust-release-check/herdr-plugin.toml). The documentation explicitly describes these as examples, not maintained official plugins.

## Capability decisions

| Question | Evidence and decision |
| --- | --- |
| Register commands/actions/UI | Manifest `[[actions]]` and `[[panes]]`; argv commands run from the plugin root. Runtime registration is not supported. `open` invokes the manifest popup; `pick` runs inside it. |
| Session-modal popup without layout changes | `placement = "popup"`; singleton session resource, no pane ID, closes on command exit. Unlike overlay/split/tab placements, it is absent from the tiled layout. |
| Position control | Public manifest/open schema supports width and height only. `resolve_popup_geometry` horizontally and vertically centers the popup. Use centered presentation; top alignment needs a new Herdr API. |
| Open workspace/tab discovery | One `herdr api snapshot` request invokes `session.snapshot`. Result is `{ "result": { "type": "session_snapshot", "snapshot": {...} } }`. Parse workspace/tab arrays, focus IDs, `number`, `active_tab_id`, and optional worktree provenance; ignore pane/agent/layout data and future fields. |
| Open worktrees | Already-open worktrees are ordinary workspace records with optional provenance. Group linked records beneath a unique open nonlinked record with the same nonblank `repo_key`; incomplete provenance, absent roots, and ambiguous roots remain standalone. Focus keeps the original workspace IDs. Never call `worktree.open`, create operations, or filesystem worktree discovery to generate candidates. |
| Active context | Top-level `focused_workspace_id` and `focused_tab_id` identify global focus. Workspace `active_tab_id` is remembered even for inactive workspaces; pane focus stays owned by Herdr. |
| Existing workspace focus | `herdr workspace focus ID` maps to `workspace.focus`; server validates the ID then calls `switch_workspace`, preserving remembered active tab. |
| Existing tab focus | `herdr tab focus ID` maps to `tab.focus`; server resolves tab and parent together before `switch_workspace_tab`. No preceding workspace request is necessary; that avoids a partial switch if the tab has closed. |
| Focus history/MRU | Snapshot has no timestamps, history, previous-workspace field, or MRU array. Its arrays are UI order. No history read method appears in bundled schema. See MRU analysis below. |
| Theme | Runtime environment/context and public schema have no plugin theme object/API. The picker now reads saved Herdr configuration and reproduces the built-in palettes, aliases, custom overrides, and light/dark resolution from the pinned revision. This is source-version compatibility rather than a live exported palette: unsaved previews and future palette changes are not available automatically. The config is never written. |
| Launch shortcut | `[[keys.command]]` with `type = "plugin_action"`, `command = "herdr-picker.open"`, and `key = "cmd+p"`. Shortcut lives in Herdr config, not plugin TOML. `prefix+f` is the terminal-compatible fallback. |
| Plugin configuration | Herdr supplies `HERDR_PLUGIN_CONFIG_DIR` and creates the directory. File format is explicitly plugin-owned. Read `config.toml` with just `search_mode`; do not write it at runtime. `HERDR_PLUGIN_STATE_DIR` is not needed. |
| Close after switching | Restore terminal modes, focus while the popup is still active, and let process exit close it. No explicit `popup.close` request is needed. Do not issue it before focus: it tears down the popup runtime and can terminate its process. |
| Mouse delivery | Popup terminal supports standard mouse modes. Client source routes mouse to the popup ahead of shell controls, translates coordinates, and carries its mouse-reporting mode. Request Crossterm mouse capture. Keyboard remains available if the terminal does not forward mouse. |
| Branch labels | Snapshot worktree provenance has repository/checkout paths and `is_linked_worktree`, but no branch. `worktree.list` returns optional branch names; additional repository discovery is possible. Omitted in this initial snapshot-only implementation to keep launch bounded and avoid Git discovery/trust failures. This is a documented implementation reduction. |

## MRU analysis and explicit limitation

The public socket supports `workspace.focused` and `tab.focused` subscriptions. A continuously connected subscriber could observe changes after subscribing, but this requires the long-running listener prohibited by this project's requirements. A snapshot cannot recover previously missed history.

Manifest `[[events]]` focus hooks are also supported and run one-shot processes, so a daemon is not strictly necessary to observe individual changes. However, upstream `run_plugin_event_hooks` starts commands asynchronously. The exported `EventEnvelope` has only `event` and `data`: no event sequence, timestamp, or server-incarnation identifier. Independent hook processes may acquire a file lock or write in the opposite order from the actual focus changes. Locking avoids corrupt files but cannot recover the missing event order. Startup hooks are asynchronous too, and plugin config/state directories survive sessions, restarts, and uninstall. A startup reset races with early focus hooks. Inferring a session identity from socket inode/mtime would rely on filesystem lifecycle behavior outside the public API contract.

Plugin command logs have server-recorded start times and IDs, but their records omit invocation workspace context and are bounded. Using their ID spelling or bounded history as an authoritative MRU store would introduce another undocumented dependency. Updating a file only when this picker opens would miss navigation through the sidebar, built-in keys, and other clients. None of these alternatives justifies claiming accurate session MRU.

Therefore this release **does not implement true MRU**. It displays the group containing the focused workspace first and the remaining groups in numbered root workspace UI order. Worktrees retain numbered workspace UI order during hierarchical searches; matching tabs appear horizontally beside their owner in numbered tab order. It preselects the first other workspace, or the only workspace if there is one. This is deterministic and supports quick switching, but cannot guarantee switching back to the previous workspace. Nothing is persisted across invocations or sessions. A future public snapshot field containing server-owned focus history would allow the requirement without a daemon or inferred ordering.

## Focus, cleanup, and failures

Server workspace and tab handlers validate IDs before changing focus. Tab focus also selects its parent workspace. Popup teardown removes only its terminal runtime and sets terminal/navigation mode; it does not restore an obsolete workspace selection. Thus focus first, then command exit is supported by source inspection. Snapshot entries can become stale, so failed focus closes the UI, attempts to focus the original snapshot tab (or workspace), sends `notification show`, writes stderr, and exits nonzero. Restoration is best effort, especially on connection timeouts where the server outcome is ambiguous.

The terminal guard restores raw mode, mouse/paste reporting, alternate screen, and cursor on normal exit and Rust unwinding. API calls have a five-second deadline and drain stdout/stderr concurrently; no shell interprets labels or IDs. Configuration is read before entering raw mode. Discovery uses a short-lived worker so Escape remains responsive while waiting; cancellation kills and reaps the CLI child and joins the worker before exiting. Typing during discovery is queued and applied to the snapshot. Herdr reports popup creation errors through the action request and normal plugin action logs. Notifications use Herdr's configured toast delivery rather than a custom message channel.

The picker fetches only once per invocation. There is no polling, subscription, refresh, background process, state store, or filesystem scan. Additional requests happen only for opening the popup, switching, failure restoration, and error notification.

## Source evidence

All links below are pinned to the inspected revision:

- [Snapshot construction in UI order](https://github.com/herdrdev/herdr/blob/916fcf8de029c6cac965b50f3cdb9687cd7d1537/src/app/api/session.rs)
- [Snapshot schema](https://github.com/herdrdev/herdr/blob/916fcf8de029c6cac965b50f3cdb9687cd7d1537/src/api/schema/session.rs)
- [Workspace records/provenance](https://github.com/herdrdev/herdr/blob/916fcf8de029c6cac965b50f3cdb9687cd7d1537/src/api/schema/workspaces.rs)
- [Workspace focus handler](https://github.com/herdrdev/herdr/blob/916fcf8de029c6cac965b50f3cdb9687cd7d1537/src/app/api/workspaces.rs)
- [Tab focus handler](https://github.com/herdrdev/herdr/blob/916fcf8de029c6cac965b50f3cdb9687cd7d1537/src/app/api/tabs.rs)
- [Popup lifecycle and teardown tests](https://github.com/herdrdev/herdr/blob/916fcf8de029c6cac965b50f3cdb9687cd7d1537/src/app/popup.rs)
- [Popup geometry](https://github.com/herdrdev/herdr/blob/916fcf8de029c6cac965b50f3cdb9687cd7d1537/src/popup_size.rs)
- [Client popup input and mouse tests](https://github.com/herdrdev/herdr/blob/916fcf8de029c6cac965b50f3cdb9687cd7d1537/src/client/shell/tests/popup_focus_projection.rs)
- [Plugin environment and asynchronous hook launching](https://github.com/herdrdev/herdr/blob/916fcf8de029c6cac965b50f3cdb9687cd7d1537/src/app/api/plugins/runtime.rs)
- [Event envelope and hook whitelist](https://github.com/herdrdev/herdr/blob/916fcf8de029c6cac965b50f3cdb9687cd7d1537/src/api/schema/events.rs)
- [Plugin context and log records](https://github.com/herdrdev/herdr/blob/916fcf8de029c6cac965b50f3cdb9687cd7d1537/src/api/schema/plugins.rs)

## Validation boundary

Rust and PTY tests exercise parsing, navigation, focus requests, terminal cleanup, and error handling using synthetic fixtures and a fake CLI on macOS and Linux. The pinned Linux container runs a real Herdr 0.9.5 server with temporary repositories, linked worktrees, plugin registration, and split panes. `just test-focus` asserts workspace/tab/pane focus, remembered active tabs, unchanged layout structure, popup teardown, resumed shell input, mouse activation, cancellation, and recovery from a closed destination. `just test-snapshots` verifies rendered output and real host keyboard delivery. No user's configuration or session is used. Native terminal shortcut interception and OS-specific appearance-report forwarding remain environment-dependent. See [automated Herdr testing](snapshots.md).

## Picker theme compatibility

The theme adapter and Settings-style action footer follow the same pinned
revision's [palette definitions](https://github.com/herdrdev/herdr/blob/916fcf8de029c6cac965b50f3cdb9687cd7d1537/src/app/state.rs),
[resolution rules](https://github.com/herdrdev/herdr/blob/916fcf8de029c6cac965b50f3cdb9687cd7d1537/src/app/mod.rs),
[theme configuration](https://github.com/herdrdev/herdr/blob/916fcf8de029c6cac965b50f3cdb9687cd7d1537/src/config/theme.rs),
and [Settings renderer](https://github.com/herdrdev/herdr/blob/916fcf8de029c6cac965b50f3cdb9687cd7d1537/src/client/shell/settings_overlay.rs).
Adapted palette code retains Apache-2.0 attribution in `src/theme.rs` and
`LICENSES/Herdr-Apache-2.0.txt`. Appearance uses a nonblocking CSI ?996n query
and mode 2031 reports. All keyboard and mouse events continue through one
input reader; reporting is disabled by the terminal guard on exit.
