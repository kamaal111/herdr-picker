# Development

Use stable Rust/Cargo and [Just](https://github.com/casey/just).
Run `just` to list recipes.

## Local plugin

With Herdr 0.9.5 or later installed, start or attach to a session, then run
from this checkout:

```sh
just plugin-link
just plugin-check
just plugin-open
```

`plugin-link` builds and registers this checkout for the current user.
`plugin-check` validates discovery without changing focus. `plugin-open`
rebuilds and opens the linked picker. These commands honor `HERDR_BIN_PATH`.
Keep the checkout in place while linked; `just plugin-unlink` unregisters it
without deleting files. Configure a [launch shortcut](../README.md#launch-shortcut)
separately. The `pick` binary command is an internal Herdr popup entrypoint.

## Checks

```sh
just ready
```

This runs formatting, Clippy, Rust tests, and PTY smoke tests; Python 3 is
required for the PTY suite. No running Herdr server is needed. For individual
checks, use `just quality`, `just test`, or `just test-pty`.

With Docker running, also run:

```sh
just test-focus
just test-snapshots
```

These exercise a real Herdr server and compare visual baselines in isolation.
All three checks must pass before publishing a UI change. CI runs `just ready`
on macOS and Linux and the Docker suites on Linux. See
[snapshot testing](snapshots.md) for baseline updates and diagnostics.

## Releases

The [Release workflow](../.github/workflows/release.yml) uses Release Please
with the built-in `GITHUB_TOKEN`. In GitHub's **Settings → Actions → General →
Workflow permissions**, enable **Allow GitHub Actions to create and approve
pull requests**. Require CI checks on `main` through branch protection.

The automation setup alone does not request a release. The first `fix:` or
`feat:` commit on `main` starts the flow below.

1. Land changes on `main` with Conventional Commit titles (`fix:` for patches,
   `feat:` for features, `!` for breaking changes). Use the same format for
   squash commit titles. Before 1.0, breaking changes bump the minor version;
   `docs:` and `chore:` alone do not request a release.
2. Review the generated release PR's versions and changelog. Approve workflow
   runs if GitHub requests it, and wait for CI to pass.
3. Merge the PR preserving its generated title and release metadata. The
   workflow creates the version tag and GitHub Release. No manual version
   edits, crates.io publication, or binary upload are needed.

For a transient failure, rerun **Actions → Release → Run workflow** on `main`.
Source installations follow `main`, so merged changes are available through
`herdr plugin update herdr-picker` before a versioned release is published.
