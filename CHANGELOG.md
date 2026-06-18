# Changelog

All notable changes to this project are documented in this file. The format is
based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the
project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.0.0] - 2026-06-18

### Added

- **Multi-ecosystem support**: Go (`go.mod`), Python (`poetry.lock` /
  `Pipfile.lock` / `requirements.txt` / `pyproject.toml` / `uv.lock`), Ruby
  (`Gemfile.lock`), PHP (`composer.lock`), Java (`pom.xml` / `build.gradle` /
  `gradle.lockfile`), Dart/Flutter (`pubspec.lock`), and C/C++ (`vcpkg.json`
  / `conanfile.txt`). Each ecosystem gets the same license classification,
  risk scoring, and commercial-use verdict as npm and Cargo.
- `install.sh`: one-command source install. Detects the operating system and
  login shell (fish/zsh/bash), validates Node.js and Rust prerequisites,
  builds both halves, installs a `~/.local/bin/dep-lens` launcher, and adds
  it to PATH (`fish_add_path` on fish, guarded rc-file lines elsewhere).
  `--uninstall` removes the launcher.
- `--tr` flag: fully localized Turkish TUI (labels, table headers, detail
  pane, help overlay, advice and status messages) backed by a typed message
  catalog where a missing translation is a compile error.
- TUI animations, all plain ASCII: animated scanning screen (spinner,
  trailing dots, elapsed seconds) shown while the scan runs inside the TUI,
  ease-out count-up summary counters, a growing color-coded category ratio
  bar, progressive table row reveal on first render, and auto-clearing
  status messages. Scan errors now render as a dedicated TUI screen and set
  exit code 2.
- License detection from LICENSE/COPYING files for packages that declare
  nothing (or only `SEE LICENSE IN ...`) in their manifest; covers Cargo's
  `license-file` field too. The `licenseSource` report field records whether
  a license was `declared`, detected from a `licenseFile`, or `none`.
- Expanded license knowledge: Zlib, BSL-1.0, Artistic-2.0, Python-2.0, MIT-0,
  WTFPL, BlueOak (permissive); MPL-1.1, EPL-1.0, CDDL (weak copyleft);
  GPL-1.0, SSPL-1.0, EUPL, OSL-3.0 (strong copyleft).
- Per-family risk scores for weak copyleft: MPL 40, EPL 45, LGPL 50, CDDL 55;
  SSPL now scores 100 alongside AGPL.
- TUI: package detail pane (enter), help overlay (h), quick category filters
  (1–4, 0 to clear), g/G jump to top/bottom, responsive column widths.
- Interactive HTML report: live search, clickable column sorting, summary
  cards that double as category filters, license source column.
- `test-project/` fixture covering all supported license scenarios across 6
  ecosystems (npm, Cargo, Go, Python, Ruby, PHP) with 31 packages, and
  `scripts/verify-fixture.py` asserting exact classification plus `--fail-on`,
  `--ignore`, and `--html` behavior — wired into CI.
- Project landing page at `docs/assets/index.html`.
- CI: Node.js version matrix now covers Node 18, 20, and 22.
- Release: npm provenance attestations via `--provenance`.

### Fixed

- Ecosystem self-test detection now aligns with what the scanner can actually
  process: Dart detects on `pubspec.lock`, Ruby on `Gemfile.lock`, PHP on
  `composer.lock` — preventing false self-test failures in projects that have
  a manifest file but have not yet run the dependency installer.

## [1.0.0] - 2026-06-14

### Added

- First stable release with npm and Cargo scanning.
- Setup wizard moved from `postinstall` to first-run (resolves npm 11
  `allow-scripts` gate). `dep-lens --setup` re-runs the wizard on demand.

## [0.1.0] - 2026-06-12

### Added

- Rust core (`dep-lens-core`) that scans `node_modules` trees and Cargo
  dependency graphs (`cargo metadata --format-version 1`).
- License classification into Permissive, Weak Copyleft, Strong Copyleft, and
  Unknown, with SPDX expression folding (`OR`, `AND`, `WITH`, `-only`,
  `-or-later`).
- Commercial risk scoring (0-100) with per-package risk level and commercial
  usability verdict; AGPL scores the maximum due to its network clause.
- JSON and standalone HTML report renderers.
- Node.js CLI (`dep-lens`) with an interactive ink TUI: header bar, colored
  summary line, filterable and sortable package table, export menu.
- CLI flags: `--json`, `--html <file>`, `--fail-on gpl|agpl`, `--path <dir>`,
  `--ignore <names>`.
- Prebuilt binaries for Linux x64, macOS x64/arm64, and Windows x64 delivered
  through platform-specific optional dependencies.
