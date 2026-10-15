# Automated Herdr integration and visual testing

Run `just ready` for Rust and fake-CLI PTY checks, `just test-focus` for live
focus and pane-preservation assertions, and `just test-snapshots` for image
regressions. CI runs all three. The latter two require Docker; Linux x86_64
emulation is used on Apple Silicon too. The first run downloads the pinned
environment and builds the plugin. Build and Cargo download caches live under
`target/snapshots/`; no Herdr installation is required on the host.

## Capture and comparison

The harness launches Herdr 0.9.5 in a fresh named session with temporary config,
state, plugin registration, sockets, repositories, and worktrees. The real
release binary opens through `herdr-picker.open`. Input goes through a PTY
client; pyte decodes the terminal cells, which Pillow renders into PNGs using
the bundled DejaVu Sans Mono 2.37 regular and bold fonts. These are captures of
the terminal content, not desktop or OS window decorations.

The canonical container pins its base by digest, Debian packages by archive
date, Herdr by SHA-256, and Python packages by version. Font files and their
redistribution license are in `tests/snapshots/fonts/`. Stable theme settings,
shell prompts, locale, dimensions, and project labels remove machine-specific
content. Startup and screen settling have bounded deadlines. Every case owns
and cleans up its server/client processes; the host Herdr session is never
used.

| Case | Source | Behavior |
| --- | --- | --- |
| `showcase-dracula` | Real server, 160×40 | Full window with `codex` prefilled; README image |
| `hierarchical-unfiltered` | Real server, 160×40 | Expanded hierarchy and initial selection |
| `flat-search` | Real server, 160×40 | Workspace breadcrumb and Codex tab on one row |
| `no-matches` | Real server, 160×40 | `zzzzzzzz`, zero results, disabled OK |
| `narrow-search` | Real server, 32×20 | Truncation and compact navigation help |
| `scrolled-selection` | Real server, 160×12 | Last workspace selected beyond the first viewport |
| `light-search` | Real server, 160×40 | Catppuccin Latte with `codex` prefilled |
| `tab-cycling` | Real server, 60×20 | Tab wraps forward; Shift+Tab selects the last tab in a shifted strip |
| `empty-snapshot` | Production renderer, 109×20 | No workspaces or tabs |
| `loading` | Production renderer, 109×20 | Loading label and disabled OK |
| `tiny-terminal` | Production renderer, 6×1 | Escape guidance |

The seed has eight workspace records: roots `herdr-picker`, `TCG`, `dotfiles`,
plus five real linked worktrees. It includes multi-tab destinations and
single-tab workspace rows, with fifteen tabs in total. `docs/readme` has only a
Terminal tab, so `codex` removes it from results while the host sidebar still
shows it. Only deterministic synthetic project data is committed.

## Focus and pane preservation

```sh
just test-focus
```

Each test starts a fresh real Herdr session using the same isolated fixture
and plugin launch path as the visual suite. It creates a split and remembers a
non-root pane, then exercises:

- Workspace Enter and OK-button activation, restoring the remembered tab/pane.
- Explicit tab activation in flat mode across linked worktrees, overriding a
  different remembered tab while preserving the selected tab's pane.
- A mouse click on a sole tab through Herdr's actual popup coordinate routing.
- Escape after selecting another destination, retaining the original context.
- A destination tab closed after discovery, restoring original focus on failure.

Assertions check exact workspace/tab/pane IDs after popup teardown, remembered
active tabs, unchanged pane IDs and layout structure (split IDs, directions,
ratios, and zoom state), and absence of layout nodes created by the popup.
Keyboard activation and cancellation also verify that shell input resumes.
Viewport rectangles are excluded because focusing an inactive tab can resize
it to the client; its pane identities and split structure must remain intact.
These tests never record or depend on PNG baselines.

Diagnostics are written under `target/snapshots/focus-TEST/`: before/after API
snapshots, available terminal output, and server logs. CI uploads them with the
visual diagnostics. Native terminal shortcut interception and OS appearance
forwarding are outside the pinned Linux environment's coverage.

## Updating baselines

```sh
just record-snapshots                         # all cases
just record-snapshots showcase-dracula        # one case
just test-snapshots showcase-dracula          # compare one case
just test-snapshots --repeat 2                 # independent captures must agree
just test-snapshots --self-test                # exercise comparison failures
```

Review PNGs before committing. Missing baselines and any changed pixel fail
comparison; recording is explicit and forbidden when `CI` is set. The README
links directly to `tests/snapshots/baselines/showcase-dracula.png`, so there is
no separately maintained documentation image.

Generated images and diagnostics are under `target/snapshots/CASE/`. A mismatch
produces `expected.png`, `actual.png`, and `difference.png`. Setup or settling
failures produce `error.txt`, plus available server logs, screen text, and raw
ANSI output. CI uploads these as `visual-snapshots` for seven days, including
actual previews on successful runs. Read diagnostics before changing a
baseline; a startup or decoder failure must not be recorded as a valid image.

`just ready` intentionally stays independent of Docker. It, `just test-focus`,
and the visual snapshot check must pass before publishing a UI change.
