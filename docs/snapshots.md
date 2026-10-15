# Integration and snapshot testing

Requires Docker. The suites use a pinned Linux x86_64 environment (emulated
on Apple Silicon) with Herdr 0.9.5. No host Herdr installation is needed;
your config and running sessions are untouched. The first run downloads the
environment and builds the plugin; caches live under `target/snapshots/`.

## Run checks

```sh
just test-focus
just test-snapshots
just test-snapshots showcase-dracula          # compare one case
just test-snapshots --repeat 2                 # check capture determinism
just test-snapshots --self-test                # check comparison failures
```

The focus suite checks workspace/tab activation, remembered panes, layout
preservation, cancellation, and stale-destination recovery against real Herdr.
The snapshot suite captures terminal content with fixed fonts and dimensions;
loading, empty-data, and tiny-terminal cases use the production renderer's
test backend. Native terminal shortcuts and OS appearance forwarding need
manual checks on the target terminal.

## Update baselines

```sh
just record-snapshots                         # all cases
just record-snapshots showcase-dracula        # one case
```

Review PNGs and commit intentional changes with the UI change. Missing
baselines or any changed pixel fail comparison. Comparison never updates
baselines; recording is forbidden when `CI` is set. The README uses
[the showcase baseline](../tests/snapshots/baselines/showcase-dracula.png)
directly.

## Diagnose failures

Visual diagnostics are under `target/snapshots/CASE/`: `expected.png`,
`actual.png`, and `difference.png` for mismatches; `error.txt`, available
server logs, screen text, and raw ANSI output for setup or capture failures.
Focus diagnostics are under `target/snapshots/focus-TEST/`, including
before/after API snapshots and server logs.

Read diagnostics before recording: a startup or decoder failure must not
become a baseline. CI uploads previews and diagnostics as `visual-snapshots`
for seven days. See [development checks](development.md#checks) for the full
verification workflow.
