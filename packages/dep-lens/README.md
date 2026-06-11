# dep-lens

Scan the licenses of your npm/yarn/pnpm and Cargo dependencies and report
commercial-use risk, in an interactive terminal UI or as JSON/HTML for CI.

```sh
npm install -g dep-lens

dep-lens                      # interactive TUI
dep-lens --json               # raw JSON to stdout
dep-lens --html report.html   # standalone HTML report
dep-lens --fail-on gpl        # exit 1 when strong copyleft is found (CI gate)
dep-lens --path ../my-app --ignore left-pad
```

Full documentation: https://github.com/dep-lens/dep-lens

This package contains the CLI and TUI; the native scanner binary is delivered
through a platform-specific optional dependency (Linux x64, macOS x64/arm64,
Windows x64).

License: MIT
