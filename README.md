> **Deprecated:** This plugin has been deprecated in favor of
> [herdrpp](https://github.com/kamaal111/herdrpp), which includes this picker feature.

# herdr-picker

Fuzzy-find open Herdr workspaces, worktrees, and tabs. Type to search, move
with the keyboard, and press Enter to switch.

![Herdr Picker searching workspaces, worktrees, and tabs](tests/snapshots/baselines/showcase-dracula.png)

## Installation

Requires macOS or Linux, a UTF-8 terminal, Herdr 0.9.5 or later, Git, and
Rust/Cargo (Rust 1.88 minimum). Herdr builds the plugin from source:

```sh
herdr plugin install kamaal111/herdr-picker
herdr plugin action invoke herdr-picker.open
```

To update, run on each machine:

```sh
herdr plugin update herdr-picker
```

Installations follow `main`. If switching from a local development link, run
`herdr plugin unlink herdr-picker` before installing; your source files and
existing shortcut are preserved.

## Launch shortcut

Add this to your Herdr config, normally `~/.config/herdr/config.toml`
(or the file specified by `HERDR_CONFIG_PATH`):

```toml
[[keys.command]]
key = "cmd+p"
type = "plugin_action"
command = "herdr-picker.open"
description = "Search workspaces, worktrees and tabs"
```

Remove any conflicting binding, then run `herdr server reload-config` where
supported, or reattach Herdr. If your terminal intercepts Cmd+P, change `key`
to `"prefix+f"` and press your Herdr prefix (default Ctrl+B), then `f`.
See [example bindings](examples/herdr.toml).

## Search

Search is case-insensitive and fuzzy. Every space-separated term must match;
terms can match different workspace, worktree, or tab names in the hierarchy.
For example, `herdr-picker multimachine codex` finds the Codex tab in the
multimachine worktree of herdr-picker.

The default **hierarchical** mode nests linked worktrees beneath their open
root workspace and keeps ancestors visible when a descendant matches.
**Flat** mode shows rows with ancestor breadcrumbs and ranks search results
by match quality. Both show matching tabs to the right of their workspace.

To change modes, find the plugin config directory:

```sh
herdr plugin config-dir herdr-picker
```

Create `config.toml` there:

```toml
search_mode = "flat"
```

Use `"hierarchical"` to restore the default, or omit the file.
See [example config](examples/config.toml).

## Controls

| Input | Action |
| --- | --- |
| Typing / paste | Filter immediately |
| ↑ / Ctrl+P | Previous workspace or worktree |
| ↓ / Ctrl+N | Next workspace or worktree |
| → / Tab | First or next matching tab, wrapping |
| ← / Shift+Tab | Previous matching tab, wrapping; from a workspace, select its last matching tab |
| Enter / click ↵ OK | Focus selection and exit |
| Esc / Ctrl+C / click Esc Cancel | Exit without switching |
| Left click | Focus clicked workspace or tab and exit |
| Mouse wheel | Move between workspace rows and scroll |

Selecting a workspace restores its remembered tab and pane. Selecting a tab
focuses it explicitly. Up/down clears tab selection and stops at the first or
last workspace row. `●` marks the active destination; `›` marks selection.
Enter does nothing when there are no results. Use Cancel to dismiss with the
mouse; outside clicks are ignored.

## Theme

The picker follows Herdr's saved theme, including custom colors and automatic
light/dark switching when the terminal reports its appearance. Reopen after
saving theme changes. Palette compatibility is based on Herdr 0.9.5;
unsaved Settings previews are unavailable to the plugin.

## Limitations

- Searches open destinations in the selected local session only; no branch labels.
- The current workspace group appears first, followed by Herdr's workspace
  order. The picker initially selects the first other workspace; it has no
  MRU history and cannot guarantee a return to the previously visited workspace.
- The popup is centered, 80% wide, and at most 22 rows tall.
- Results are fetched once per opening. Reopen to discover changes.

## Troubleshooting

- **Plugin unavailable:** inspect `herdr plugin list`. For a local checkout,
  follow the [development setup](docs/development.md#local-plugin).
- **No server:** check `herdr status server` and your selected session. Set
  `HERDR_BIN_PATH` if Herdr is outside PATH. Escape cancels discovery.
- **Popup does not open:** close Settings, Copy mode, or another modal, then
  inspect `herdr plugin log list --plugin herdr-picker`.
- **Shortcut does nothing:** try the action invocation from installation,
  then check the binding and terminal forwarding; use `prefix+f` if needed.
- **Configuration error:** only `search_mode = "hierarchical"` or `"flat"`
  is accepted. Unknown fields are errors.
- **Destination closed:** the picker exits and attempts to restore original
  focus. Reopen for fresh results. Errors go to stderr and Herdr notifications,
  whose visibility is controlled by Herdr's `[ui.toast]` settings.
- **Mouse unavailable:** enable Herdr/terminal mouse capture or use the keyboard.
- **Colors differ:** save Herdr Settings and reopen; check `HERDR_CONFIG_PATH`
  and custom overrides. Appearance reporting depends on your terminal.

## Development

See [development and releases](docs/development.md) and
[integration and snapshot testing](docs/snapshots.md).

## License

Project code is [MIT licensed](LICENSE). Adapted Herdr palette code retains
Apache-2.0 attribution in [src/theme.rs](src/theme.rs) and
[LICENSES/Herdr-Apache-2.0.txt](LICENSES/Herdr-Apache-2.0.txt).
Bundled DejaVu Sans Mono snapshot fonts use the
[Bitstream Vera license](tests/snapshots/fonts/LICENSE).
