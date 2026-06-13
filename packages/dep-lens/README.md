# dep-lens

Scan your project's third-party dependencies across **9 ecosystems** (npm,
Cargo, Go, Python, Ruby, PHP, Java, Dart/Flutter, C/C++), classify every
license (permissive / weak copyleft / strong copyleft / unknown), score
commercial-use risk, and browse the results in a fast, colorful terminal UI
or as JSON/CSV/Markdown/HTML for CI.

```sh
npm install -g @lunanoir/dep-lens

dep-lens                      # interactive TUI
dep-lens --tr                 # Turkish UI (Turkce arayuz)
dep-lens --json               # raw JSON to stdout
dep-lens --html report.html   # standalone HTML report
dep-lens --test               # self-check: verify the scanner on this project
dep-lens --fail-on gpl        # exit 1 when strong copyleft is found (CI gate)
dep-lens --path ../my-app --ignore left-pad
```

Full documentation: https://github.com/lunanoir21/dep-lens

This package contains the CLI and TUI; the native scanner binary is delivered
through a platform-specific optional dependency (Linux x64, macOS x64/arm64,
Windows x64).

License: MIT
