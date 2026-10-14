# herdr-picker

A small, on-demand Rust plugin that fuzzy-finds **open Herdr workspaces and tabs**. Type immediately, move with the keyboard, and press Enter to focus a destination. Opened worktrees behave like any other workspace. Nothing is created or closed.

```text
> color codex                                      2 results
─────────────────────────────────────────────────────────
    ColorPicker
›     Codex

 ↑↓/Ctrl+P,N Navigate   Enter Switch   Esc Close
```

Herdr's built-in sidebar and navigation keys move through its existing UI. This plugin adds one searchable palette containing selectable workspace rows and individual tab destinations, including matching across both names.

**Current limits:** Herdr 0.9.5 popups are centered, and its snapshot does not contain MRU history. This release orders the current workspace first, followed by Herdr's workspace UI order, and initially selects the first other workspace. It cannot promise `Cmd+P → Enter` returns to the previously visited workspace. Branch labels are omitted. These differences, the alternatives investigated, and source references are detailed in [API feasibility](docs/api-research.md).

## Requirements and installation

- Herdr **0.9.5 or later** (0.9.5 is the tested API baseline).
- macOS or Linux, with a UTF-8 terminal.
- Stable Rust and Cargo (Rust 1.88 minimum for this source release).
- Git for installation from GitHub. No daemon, Node, or Python runtime is needed by the plugin.

Build and link this directory:

```sh
cargo build --release --locked
herdr plugin link "$PWD"
herdr plugin action invoke herdr-picker.open
```

`plugin link` does not run build commands. Rebuild after source changes. Linking registers the plugin globally for the current user; it does not edit keyboard shortcuts.

Once a maintainer publishes this repository to GitHub, replace `OWNER` with that account:

```sh
herdr plugin install OWNER/herdr-picker
herdr plugin action invoke herdr-picker.open
```

Herdr runs the manifest's locked Cargo release build during GitHub installation. Review the installation preview as with any executable plugin. This project does not assume that a published repository or prebuilt binary exists.

To open the pane entrypoint directly:

```sh
herdr plugin pane open --plugin herdr-picker --entrypoint picker
```

To unlink local development files without deleting them:

```sh
herdr plugin unlink herdr-picker
```

## Launch shortcut

Add the following to your existing Herdr configuration (normally `~/.config/herdr/config.toml`; respect `HERDR_CONFIG_PATH` if set):

```toml
[[keys.command]]
key = "cmd+p"
type = "plugin_action"
command = "herdr-picker.open"
description = "Search workspaces and tabs"
```

`cmd+p` is the desired default binding, supplied as an example because plugins cannot register a default shortcut in their manifest. Herdr owns the shortcut; there is no redundant `shortcut` field in the plugin's config. Change `key` to configure it. Remove any conflicting binding for the same chord. Reload with `herdr server reload-config` where supported, or reattach Herdr to load the config.

A reliable fallback is your Herdr prefix followed by `f`:

```toml
[[keys.command]]
key = "prefix+f"
type = "plugin_action"
command = "herdr-picker.open"
description = "Search workspaces and tabs"
```

Use Herdr's configured prefix (the factory default is `Ctrl+B`), then `f`. This avoids needing terminal support for a Command/Super chord. On Linux you can use the same fallback, or a supported modified chord of your choice. Merely setting `cmd+p` in Herdr cannot make an outer terminal forward it.

### WezTerm

If WezTerm intercepts Command-P, merge this into its existing `config.keys` table:

```lua
local wezterm = require 'wezterm'
-- Inside config.keys:
{ key = 'p', mods = 'CMD', action = wezterm.action.SendString '\x1b[112;9u' },
```

This forwards CSI-u `p` with Super/Command (modifier 9) for Herdr's `cmd+p` binding. Keep the matching Herdr binding above. The example is based on [WezTerm's key actions](https://wezterm.org/config/keys.html); forwarding in an actual WezTerm session remains a manual verification item. Other terminals need their own key forwarding configuration, or use `prefix+f`. No configuration is edited automatically. Complete snippets are in [examples](examples/).

## Search modes

The plugin config lives in Herdr's supported per-plugin config directory:

```sh
herdr plugin config-dir herdr-picker
```

Create `config.toml` inside the printed directory:

```toml
search_mode = "hierarchical"
```

`hierarchical` is the default if the file or setting is absent. Every workspace is expanded, and every visible workspace/tab row is selectable. A workspace name match includes all its tabs. A tab match includes its selectable parent workspace. Groups keep their workspace order, and tabs keep Herdr's UI order, even during filtering.

Set `search_mode = "flat"` for independent results such as `ColorPicker / Codex` and `ColorPicker`. Nonempty searches rank by fuzzy relevance; empty searches use the same workspace order as hierarchical mode. Tab results can match their parent name as context.

Matching ignores case, supports noncontiguous characters, and requires **every whitespace-separated term** to match a workspace or tab name. Each term must fit within one name; separate terms can match different levels. `color codex` finds `ColorPicker / Codex`. `color` can also match `ColorPicker / Claude Code` because `cod` genuinely matches “Code.” No branches, paths, panes, agent metadata, or unopened worktrees participate. Full names stay searchable after visual truncation.

Changing the query immediately selects the first result in the current mode's ordering. Searching never switches automatically. Clearing the query selects the first result too; only a newly opened picker uses the initial other-workspace selection.

## Controls

| Input | Action |
| --- | --- |
| Typing / paste | Filter immediately |
| ↑ / Ctrl+P | Previous row |
| ↓ / Ctrl+N | Next row |
| Enter | Focus selected workspace or tab and exit |
| Esc / Ctrl+C | Exit without switching |
| Left click | Focus clicked result and exit |
| Mouse wheel | Move selection and scroll results |

Navigation stops at the first/last row. Arrow left/right have no tree behavior. Empty results show “No matching results”; Enter does nothing. Every invocation starts with an empty query. `●` marks the currently active workspace/tab; `›` and reverse video mark selection. Matches are bold and underlined, including on selected rows. Labels stay on one line and truncate with `…`.

The centered popup is 80% wide and at most 22 outer rows, clamped by Herdr to the terminal size. All results remain reachable by scrolling. The terminal UI responds to resize; extremely small areas retain Escape guidance. It requests terminal mouse reporting; Herdr 0.9.5 source forwards popup mouse events. Enable Herdr's mouse capture if the outer terminal otherwise handles clicks.

## Development

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo build --release
python3 tests/pty_smoke.py
```

The Rust tests do not need a running Herdr server. Python 3 is used only by the optional PTY smoke suite, which launches the real release binary against a fake CLI and checks focus requests, resize, terminal cleanup, cancellation, and failure recovery. CI runs both suites on macOS and Linux. See [manual end-to-end checks](docs/manual-testing.md) before publishing a release.

A read-only connection/response check:

```sh
./target/release/herdr-picker check
```

`HERDR_BIN_PATH` is used when set; otherwise the plugin finds `herdr` on PATH. The CLI inherits Herdr's session and socket environment. `pick` is an internal popup entrypoint and requires `HERDR_PLUGIN_CONFIG_DIR`. Do not run it inside an ordinary tiled pane and expect a popup.

```text
herdr-plugin.toml      Manifest, action, popup entrypoint, install build
src/api.rs            CLI transport, bounded requests, parsing, focus
src/model.rs          Snapshot models and ID/parent validation
src/search.rs         Fuzzy terms, highlights, hierarchical/flat results
src/ordering.rs       Explicit ordering and initial-selection fallback
src/app.rs            Selection, scrolling, keyboard/mouse state
src/ui.rs             Rendering and terminal lifecycle
src/config.rs         Strict TOML configuration
src/main.rs           Entrypoints and error reporting/recovery
tests/                Search, navigation, API, render, and PTY tests
docs/                 Feasibility evidence and manual test procedure
examples/             Herdr, picker, and WezTerm configuration
```

## Errors and troubleshooting

- **Cannot start Herdr / no server:** check `herdr status server`, session selection, and `HERDR_BIN_PATH`. Requests time out after five seconds; Escape also cancels a waiting discovery request. The picker does not start a server.
- **Plugin is unavailable:** build first, link the project root, and inspect `herdr plugin list`. The minimum Herdr version and platform are validated by Herdr.
- **Popup does not open:** close Settings, Copy mode, or another modal. Herdr rejects concurrent modals with `ui_busy`. Inspect `herdr plugin log list --plugin herdr-picker` for action failures.
- **Shortcut does nothing:** verify the action works from the CLI, then check the Herdr binding and outer terminal forwarding. Try the prefix fallback.
- **Configuration error:** use exactly `hierarchical` or `flat`. Unknown fields are errors, so typos do not silently change behavior.
- **Destination closed after opening:** the server rejects the stale ID; the picker exits, attempts to restore the original tab/workspace, and reports an error through Herdr notifications and stderr. There is no refresh loop. Reopen for a new snapshot.
- **No visible error toast:** Herdr's `[ui.toast]` settings control notification delivery. If the notification API also fails, stderr contains both errors. CLI actions are logged by Herdr; popup stderr is terminal output and may disappear on exit.
- **Mouse unavailable:** check Herdr/terminal mouse capture and use keyboard navigation. A terminal modifier used to bypass mouse reporting intentionally prevents delivery to the popup.
- **Colors differ from Herdr chrome:** no theme object is exported to plugins. Default terminal colors and text attributes are the fallback; no theme file is parsed or overwritten.

## Known limitations and verification status

True session MRU, previous-workspace preselection, top alignment, and a plugin theme object are unavailable through the chosen public API surface. No persistent MRU file or background listener is used. Branch display is omitted because the single snapshot lacks branch data: `worktree.list` could enrich it using extra repository discovery calls, but this release intentionally does not make those calls. That is an implementation tradeoff, not a claim that Herdr cannot report branches at all.

Only the local selected Herdr session is covered; this release does not aggregate machines or sessions. Unknown response fields are ignored, while missing/duplicate IDs and broken parent relationships are rejected. Focus uses stable public IDs and preserves remembered tab/pane context through Herdr's focus methods. A focus failure triggers best-effort restoration; if the original destination also closed or the server is unavailable, restoration cannot be guaranteed.

macOS builds, automated tests, PTY behavior, and live snapshot parsing were verified during development. Linux is configured in CI but was not executed locally. Actual popup installation/launch, live workspace/tab switching, active-pane preservation, mouse delivery, and WezTerm Command-P forwarding still require the documented manual test. No repository has been published, and no user shortcuts or terminal configuration have been changed.

MIT licensed. Once published, maintainers can add the GitHub topic `herdr-plugin` for Herdr's community plugin index.
