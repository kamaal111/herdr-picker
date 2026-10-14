# Manual end-to-end verification

Use a disposable Herdr session you control. These steps deliberately change focus and create test fixtures; do not run them against somebody else's working session. Automated tests do not need this setup.

## Prepare

1. Build with `cargo build --release --locked` and link this project root with `herdr plugin link "$PWD"`. Confirm Herdr 0.9.5 or newer and enabled plugin status with `herdr plugin list`.
2. In a disposable named session, create two or more workspaces and multiple tabs, including duplicated tab names such as “Codex,” “Claude Code,” and “Terminal.” Give one workspace a long Unicode name and include enough tabs to exceed 22 popup rows. Use the Herdr UI or documented create commands for this manual setup; the plugin itself never creates them.
3. Open a root Git workspace and at least two of its worktrees in that session. Leave another worktree unopened on disk. Include a non-Git workspace too.
4. Arrange multiple panes in a tab, select a non-root pane, and note workspace/tab/pane IDs using `herdr api snapshot`. Preserve the test session's selection while collecting evidence.
5. Launch `herdr plugin action invoke herdr-picker.open` in the correct session environment. Test both the action and direct `plugin pane open` command. Confirm popup dimensions and that the underlying tab layout is unchanged.

## Discovery, cancellation, and selection

- Confirm all open workspaces and tabs are present, including the opened worktree, and the unopened checkout is absent.
- Verify `●` indicates the active workspace and tab, while `›`/reverse video indicates selection.
- Confirm the current workspace's group is first, the remaining groups follow root workspace numbered order, root tabs precede worktrees, worktrees and tabs follow UI order, and the initial selection is the first other workspace. This is the documented fallback, **not MRU**. Do not record an MRU test as passing.
- Type immediately without focusing the input. Search `color codex`, noncontiguous abbreviations, mixed case, multiple terms, and nonsense. Both terms must match real names. Branch and path text must not produce results unless that same text appears in a name.
- Search a root workspace name: all its worktrees and tabs appear. Search a worktree name: its tabs appear with its selectable root. Search only a tab name: all its ancestors remain selectable. Search terms spanning root, worktree, and tab names. Check the displayed count includes both workspace and tab rows for multi-tab workspaces.
- Include a workspace and a nested worktree with exactly one tab. In both modes, verify one `Workspace - Tab` row, matching and highlights across both names, one result in the count, and workspace focus via Enter and click. Check narrow/Unicode labels remain readable. A multi-tab workspace filtered to one matching tab must keep its usual layout.
- Close the root workspace and reopen the picker: its worktrees remain standalone and selectable. Rename an open root: its worktrees stay nested beneath it. Keep unrelated repositories with identical names separate.
- Navigate through every row with arrows and Ctrl+N/P. Left/right must neither hide nor expand rows. Type after scrolling; selection and scroll reset to the first result.
- Press Enter with no results: nothing happens. Search with exactly one result: no focus changes until Enter.
- Press Escape with a nonempty query: close immediately, retain the original workspace, tab, and pane. Reopen: query is empty.
- Click root, worktree, and tab rows, scroll with the wheel, then resume keyboard navigation. Verify coordinate translation in the actual host popup.
- Resize to a small terminal and then enlarge it. All results must remain reachable, labels must remain on one line, and full labels must still match after truncation.

## Focus and cleanup

1. Select a workspace and press Enter. Verify its remembered active tab and pane return, popup disappears, and normal input reaches the destination immediately.
2. Select a particular tab in another workspace. Verify both parent and tab focus change, the previous pane in that tab remains active, and no layout nodes were added.
3. Repeat with duplicated display names; compare exact snapshot IDs after each switch.
4. Alternate keyboard/mouse selection and repeated open/Escape. Check raw mode, cursor, alternate screen, and mouse capture are restored correctly after every close.
5. While a picker snapshot is open, close the intended test destination from another shell/client. Select the stale row: picker exits, restores original context when available, and reports a concise error notification. Close the original destination too and confirm restoration failure is also reported.
6. Test unavailable server/invalid config/missing executable only in the disposable environment. Do not stop a shared server. Errors must not leave a raw terminal or an open picker.
7. Open Settings or another modal, then invoke the action externally. Confirm Herdr returns `ui_busy` and the action error is reported/logged.

## Flat mode and shortcuts

- Put `search_mode = "flat"` in the plugin config directory. Reopen and check `workspace / tab` and `workspace / worktree / tab` labels, independent selectable workspace results, relevance ranking, and matching highlights across all ancestor and destination labels. Invalid values must report errors. Restore the intended mode afterward.
- Merge `examples/herdr.toml` into the disposable config. Check `cmd+p` using the Herdr binding. Also test `prefix+f`. Record terminal/OS versions and whether each chord reaches Herdr.
- Change Herdr theme. Check legibility using terminal defaults, reverse selection, underline/bold matches, and dim help. Do not claim exact Herdr theme-token matching; there is no public theme object.
- Run the same procedure on Linux before declaring Linux behavior verified.

## Release evidence

Record Herdr version/protocol, OS, terminal, configuration changes, focus IDs before/after, and any failures. Run the four Cargo verification commands and `python3 tests/pty_smoke.py`. Validate installation from the eventual GitHub repository in a clean environment with Rust/Cargo and Git installed; the placeholder `OWNER/herdr-picker` cannot be tested before publication. No publish step is performed by this project.
