# dep-lens

Scan the licenses of your npm/yarn/pnpm and Cargo dependencies and report
commercial-use risk. Rust core for fast scanning, Node.js wrapper with a fully
interactive terminal UI.

```
+--------------------------------------------------------------------+
| dep-lens my-project          scanned 2026-06-12T08:30:00Z  pkgs 312 |
+--------------------------------------------------------------------+
 Permissive 287 (92.0%)  Weak 14 (4.5%)  Strong 3 (1.0%)  Unknown 8 (2.6%)
+--------------------------------------------------------------------+
| PACKAGE ^         VERSION   LICENSE     CATEGORY         RISK  ...  |
| left-pad          1.3.0     MIT         Permissive       0     yes  |
| some-agpl-lib     2.1.0     AGPL-3.0    Strong Copyleft  100   ...  |
+--------------------------------------------------------------------+
 up/down move  f filter  s sort  r reverse  e export  q quit
```

## Install

```sh
npm install -g dep-lens
```

The package ships prebuilt native binaries for Linux x64, macOS x64/arm64, and
Windows x64 via optional dependencies; the right one is selected automatically.

### From source (one command)

```sh
git clone https://github.com/dep-lens/dep-lens && cd dep-lens
./install.sh
```

The installer detects your operating system and login shell (fish, zsh, or
bash), checks that Node.js >= 18.18 and Rust are available, builds both
halves, installs a launcher to `~/.local/bin/dep-lens`, and adds that
directory to your PATH if it is not there yet (via `fish_add_path` on fish,
or a guarded line in `.zshrc` / `.bashrc`). Re-run it any time to rebuild;
`./install.sh --uninstall` removes the launcher.

## Usage

```sh
# Interactive TUI for the current directory
dep-lens

# Turkish UI (Turkce arayuz)
dep-lens --tr

# Scan another project
dep-lens --path ../my-app

# Raw JSON to stdout (also the default when stdout is not a TTY)
dep-lens --json

# Standalone HTML report
dep-lens --html report.html

# Exclude vetted packages
dep-lens --ignore left-pad --ignore internal-pkg-a,internal-pkg-b

# CI/CD gate: exit code 1 when strong copyleft licenses are present
dep-lens --fail-on gpl     # GPL-2.0, GPL-3.0, AGPL-3.0
dep-lens --fail-on agpl    # AGPL-3.0 only
```

### TUI keys

| Key           | Action                                                  |
| ------------- | ------------------------------------------------------- |
| up / down     | Move selection (pgup/pgdn jump 10, g/G top/bottom)      |
| enter         | Package detail pane (license source, advice)            |
| f             | Free-text filter by package name, license, or category  |
| 1 / 2 / 3 / 4 | Quick filter: Permissive / Weak / Strong / Unknown      |
| 0             | Clear all filters                                       |
| s             | Cycle sort column                                       |
| r             | Reverse sort direction                                  |
| e             | Export menu (JSON or HTML file)                         |
| h             | Help overlay                                            |
| q             | Quit                                                    |

Risky packages are highlighted red, caution-level packages yellow, clean
packages green, and unidentified licenses gray.

The TUI is fully localized; `--tr` switches every label, advice text, and
status message to Turkish. While scanning, an animated progress screen shows
an ASCII spinner and elapsed time; once results land, the summary counters
count up with an ease-out curve, a color-coded ratio bar (`#` permissive,
`=` weak, `!` strong, `?` unknown) grows to its final proportions, and table
rows reveal progressively. Status messages clear themselves after a few
seconds. All animation is plain ASCII and ANSI colors; no emoji anywhere.

### CI example

```yaml
- name: License gate
  run: npx dep-lens --json --fail-on gpl
```

## License classification

| Category        | Licenses                                                                                     | Risk score                       | Commercial use |
| --------------- | -------------------------------------------------------------------------------------------- | -------------------------------- | -------------- |
| Permissive      | MIT, Apache-2.0, BSD-2/3-Clause, ISC, 0BSD, Unlicense, CC0, Zlib, BSL-1.0, Artistic-2.0, ... | 0                                | yes            |
| Weak Copyleft   | LGPL-2.0/2.1/3.0, MPL-1.1/2.0, EPL-1.0/2.0, CDDL-1.0/1.1                                      | MPL 40, EPL 45, LGPL 50, CDDL 55 | caution        |
| Strong Copyleft | GPL-1.0/2.0/3.0, AGPL-1.0/3.0, SSPL-1.0, EUPL-1.1/1.2, OSL-3.0                                | 90 (AGPL/SSPL 100)               | restricted     |
| Unknown         | Anything else or missing                                                                      | 60                               | review         |

SPDX expressions are folded: `MIT OR Apache-2.0` counts as the least
restrictive option (dual licensing lets you choose), `MIT AND GPL-2.0` as the
most restrictive. `-only` / `-or-later` suffixes and `WITH` exception clauses
are normalized away.

When a package declares no license (or only `SEE LICENSE IN ...`), dep-lens
reads its LICENSE/COPYING file and identifies MIT, Apache, the GPL family,
BSD, ISC, MPL, EPL, BSL, Unlicense, and CC0 from the text. The JSON report
records where each license came from in the `licenseSource` field
(`declared`, `licenseFile`, or `none`).

This tool produces an automated report to support a review. It is not legal
advice.

## How it works

- `crates/dep-lens-core` (Rust) walks `node_modules` reading each package's
  `package.json`, runs `cargo metadata --format-version 1` for Cargo projects,
  classifies licenses, scores risk, and prints a JSON (or HTML) report to
  stdout.
- `packages/dep-lens` (TypeScript) spawns that binary, parses the JSON, and
  renders the interactive TUI with [ink](https://github.com/vadimdemedes/ink).

## Development

Requirements: Rust stable, Node.js >= 18.18.

```sh
npm install            # workspace deps (platform binary packages are skipped)
npm run build          # cargo build --release + tsc
npm test               # cargo test + node --test
npm run lint           # clippy -D warnings + rustfmt check

# Run the CLI from the working tree (uses target/release automatically)
node packages/dep-lens/dist/cli.js --path .

# End-to-end check against the bundled fixture project
python3 scripts/verify-fixture.py
```

`test-project/` is a fixture with fake packages covering every supported
license scenario (declared, LICENSE-file-only, `SEE LICENSE IN` placeholders,
SPDX expressions, scoped and nested npm packages, Cargo path dependencies).
`scripts/verify-fixture.py` scans it and asserts the exact classification of
all 22 packages plus the `--fail-on`, `--ignore`, and `--html` behavior.

`DEP_LENS_BINARY=/path/to/dep-lens-core` overrides binary resolution.

## License

[MIT](LICENSE)
