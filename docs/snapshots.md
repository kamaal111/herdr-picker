# Visual snapshot testing

Run `just ready` for existing Rust and PTY checks, and `just test-snapshots`
for image regressions. Docker must be running; its Linux x86_64 emulation is
used on Apple Silicon too. The first run downloads the pinned environment and
builds the plugin. Build and Cargo download caches live under
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
| `flat-search` | Real server, 160×40 | `herdr-picker feature/search codex` breadcrumb |
| `no-matches` | Real server, 160×40 | `zzzzzzzz`, zero results, disabled OK |
| `narrow-search` | Real server, 32×20 | Truncation and compact navigation help |
| `scrolled-selection` | Real server, 160×40 | Last result selected beyond the first viewport |
| `light-search` | Real server, 160×40 | Catppuccin Latte with `codex` prefilled |
| `empty-snapshot` | Production renderer, 109×20 | No workspaces or tabs |
| `loading` | Production renderer, 109×20 | Loading label and disabled OK |
| `tiny-terminal` | Production renderer, 6×1 | Escape guidance |

The seed has eight workspace records: roots `herdr-picker`, `TCG`, `dotfiles`,
plus five real linked worktrees. It includes multi-tab destinations and
single-tab inline rows, with fifteen tabs in total. `docs/readme` has only a
Terminal tab, so `codex` removes it from results while the host sidebar still
shows it. Only deterministic synthetic project data is committed.

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

`just ready` intentionally stays independent of Docker. Both it and the visual
snapshot check must pass before publishing a UI change.
