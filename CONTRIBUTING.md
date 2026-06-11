# Contributing to dep-lens

Thanks for taking the time to contribute.

## Repository layout

```
crates/dep-lens-core/      Rust library + binary (scanning, classification, reports)
  src/scanner/npm.rs       node_modules walker
  src/scanner/cargo.rs     cargo metadata collector
  src/license/classifier.rs  SPDX classification
  src/license/risk.rs      commercial risk scoring
  src/report/{json,html}.rs  output renderers
packages/dep-lens/         Node.js CLI + ink TUI
  src/bridge.ts            spawns the Rust binary, parses JSON
  src/ui/                  TUI components
packages/dep-lens-*/       per-platform binary packages (filled by CI)
```

## Setup

```sh
npm install
npm run build
npm test
```

`npm install` intentionally skips optional dependencies (see `.npmrc`): the
platform binary packages only exist on the registry after a release. During
development the CLI falls back to `target/release/dep-lens-core` or
`target/debug/dep-lens-core`.

## Quality bars

Both are enforced by CI on every pull request:

- Rust: `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`
  must pass; add unit tests next to the code you change.
- TypeScript: compiles under `strict` mode; tests run with the built-in
  `node --test` runner from `packages/dep-lens/src/test/`.

UI rules: no emoji anywhere; ASCII box-drawing characters and ANSI color names
only. Keep the JSON contract in `src/types.ts` in sync with
`crates/dep-lens-core/src/model.rs`.

## Pull requests

1. Fork and create a topic branch.
2. Keep changes focused; include tests.
3. Make sure `npm test` and `npm run lint` pass locally.
4. Describe what changed and why in the PR body.

## Releases (maintainers)

1. Bump the version in `Cargo.toml` (workspace), `packages/dep-lens/package.json`
   (including its `optionalDependencies` pins), and every
   `packages/dep-lens-*/package.json`.
2. Update `CHANGELOG.md`.
3. Tag `vX.Y.Z` and push the tag. The release workflow cross-compiles the
   binary for all supported platforms and publishes the npm packages.
