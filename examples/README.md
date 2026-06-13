# Example projects

One self-contained fixture per supported ecosystem, used to manually verify
(and demo) each scanner. Every fixture is small on purpose: just enough
manifest/lockfile/cache structure for `dep-lens-core` to find and classify a
couple of real packages without touching the network.

Ecosystems that read licenses from a local cache (Go module cache, Maven
`~/.m2`, pub cache, RubyGems `GEM_HOME`) ship a tiny fake cache inside the
fixture and need the matching `DEP_LENS_*` override so the scanner looks
there instead of your real machine-wide cache.

| Fixture | Manifest | Cache override |
| --- | --- | --- |
| `npm-test-project` | `package.json` + `node_modules` | - |
| `cargo-test-project` | `Cargo.toml` (path dependency) | - |
| `go-test-project` | `go.mod` | `DEP_LENS_GOPATH=go-test-project/.cache/gopath` |
| `python-test-project` | `pyproject.toml` (no lockfile) | - |
| `ruby-test-project` | `Gemfile.lock` | `DEP_LENS_GEM_HOME=ruby-test-project/vendor/bundle/ruby/3.3.0` |
| `php-test-project` | `composer.json` + `composer.lock` | - |
| `java-test-project` | `build.gradle` (no lockfile) | `DEP_LENS_M2=java-test-project/.m2/repository` |
| `dart-test-project` | `pubspec.lock` | `DEP_LENS_PUB_CACHE=dart-test-project/.pub-cache` |
| `cpp-test-project` | `vcpkg.json` + `vcpkg_installed` | - |

## Run them all

```sh
./examples/verify.sh
```

This builds the Rust core if needed and prints a one-line summary
(`total / permissive / weak / strong / unknown`) for every fixture above,
failing if any scanner errors or returns zero packages.

## Run one by hand

```sh
cd examples/python-test-project
node ../../packages/dep-lens/dist/cli.js --json
```

Or, for the cache-backed ecosystems:

```sh
DEP_LENS_GOPATH=$PWD/.cache/gopath \
  node ../../packages/dep-lens/dist/cli.js --path examples/go-test-project --json
```

### What each fixture demonstrates

- **python** - no lockfile at all (just `pyproject.toml` with PEP 621
  `dependencies`); dep-lens still lists every declared dependency with its
  version specifier instead of returning an empty report.
- **java** - no `gradle.lockfile`; coordinates are parsed straight out of
  `build.gradle`'s `implementation(...)` / `testImplementation` calls.
- **go** - license text comes from the module cache (`$GOPATH/pkg/mod`).
- **dart** - license text comes from the pub cache
  (`~/.pub-cache/hosted/pub.dev`).
- **ruby** - one license from a gemspec (`rails`), one from a vendored
  `MIT-LICENSE` file (`rake`).
- **cpp** - one vcpkg port with an installed `copyright` file (`fmt`, MIT),
  one with no license info available (`openssl`, Unknown).
