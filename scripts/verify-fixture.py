#!/usr/bin/env python3
"""End-to-end verification of dep-lens against test-project/.

Runs the CLI in JSON mode and asserts the exact classification of every
fixture package, then exercises --fail-on, --ignore, and --html.

Usage: python3 scripts/verify-fixture.py
"""

import json
import os
import subprocess
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CLI = os.path.join(ROOT, "packages", "dep-lens", "dist", "cli.js")
FIXTURE = os.path.join(ROOT, "test-project")

# name -> manifest JSON; nested paths use "/" (scoped and nested packages).
FIXTURE_MANIFESTS = {
    "mit-lib": {"name": "mit-lib", "version": "1.0.0", "license": "MIT"},
    "apache-lib": {"name": "apache-lib", "version": "2.3.1", "license": "Apache-2.0"},
    "isc-object-lib": {
        "name": "isc-object-lib",
        "version": "0.4.0",
        "license": {"type": "ISC", "url": "https://opensource.org/licenses/ISC"},
    },
    "legacy-bsd-lib": {
        "name": "legacy-bsd-lib",
        "version": "3.2.1",
        "licenses": [{"type": "BSD-3-Clause"}],
    },
    "zlib-lib": {"name": "zlib-lib", "version": "1.2.0", "license": "Zlib"},
    "lgpl-lib": {"name": "lgpl-lib", "version": "2.1.0", "license": "LGPL-2.1"},
    "mpl-lib": {"name": "mpl-lib", "version": "5.0.0", "license": "MPL-2.0"},
    "epl-lib": {"name": "epl-lib", "version": "1.1.0", "license": "EPL-2.0"},
    "gpl-lib": {"name": "gpl-lib", "version": "4.0.2", "license": "GPL-3.0-only"},
    "agpl-lib": {"name": "agpl-lib", "version": "6.6.6", "license": "AGPL-3.0"},
    "dual-or-lib": {"name": "dual-or-lib", "version": "1.5.0", "license": "GPL-2.0 OR MIT"},
    "strict-and-lib": {
        "name": "strict-and-lib",
        "version": "0.9.0",
        "license": "MIT AND GPL-2.0",
    },
    "@corp/scoped-lib": {
        "name": "@corp/scoped-lib",
        "version": "2.0.0",
        "license": "BSD-2-Clause",
    },
    "no-license-lib": {
        "name": "no-license-lib",
        "version": "0.0.1",
        "description": "nothing declared",
    },
    "wtfpl-lib": {"name": "wtfpl-lib", "version": "2.0.0", "license": "WTFPL"},
    "mit-lib/node_modules/nested-unlicense": {
        "name": "nested-unlicense",
        "version": "1.0.0",
        "license": "Unlicense",
    },
    "file-only-mit": {"name": "file-only-mit", "version": "1.1.0"},
    "see-license-gpl": {
        "name": "see-license-gpl",
        "version": "3.0.0",
        "license": "SEE LICENSE IN COPYING",
    },
    "proprietary-lib": {"name": "proprietary-lib", "version": "1.0.0", "license": "Proprietary"},
}

FIXTURE_EXTRA_FILES = {
    "file-only-mit/LICENSE": (
        "MIT License\n\nCopyright (c) 2026 Example\n\nPermission is hereby granted, "
        "free of charge, to any person obtaining a copy\nof this software...\n"
    ),
    "see-license-gpl/COPYING": (
        "GNU GENERAL PUBLIC LICENSE\nVersion 3, 29 June 2007\n\n"
        "Copyright (C) 2007 Free Software Foundation...\n"
    ),
}


def ensure_fixture():
    """Recreate the fake npm packages if they are missing.

    Running `npm install` inside test-project wipes node_modules (npm prunes
    everything not in the lockfile), so the fixture self-heals here.
    """
    node_modules = os.path.join(FIXTURE, "node_modules")
    probe = os.path.join(node_modules, "mit-lib", "package.json")
    if os.path.isfile(probe):
        return
    print("fixture node_modules missing or pruned; recreating...")
    lockfile = os.path.join(FIXTURE, "package-lock.json")
    if os.path.isfile(lockfile):
        os.unlink(lockfile)
    for rel, manifest in FIXTURE_MANIFESTS.items():
        pkg_dir = os.path.join(node_modules, *rel.split("/"))
        os.makedirs(pkg_dir, exist_ok=True)
        with open(os.path.join(pkg_dir, "package.json"), "w", encoding="utf-8") as handle:
            json.dump(manifest, handle)
    for rel, content in FIXTURE_EXTRA_FILES.items():
        path = os.path.join(node_modules, *rel.split("/"))
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(content)
    os.makedirs(os.path.join(node_modules, ".bin"), exist_ok=True)

# name -> (license, category, riskScore, commercialUse, licenseSource, ecosystem)
EXPECTED = {
    "mit-lib": ("MIT", "Permissive", 0, "yes", "declared", "npm"),
    "apache-lib": ("Apache-2.0", "Permissive", 0, "yes", "declared", "npm"),
    "isc-object-lib": ("ISC", "Permissive", 0, "yes", "declared", "npm"),
    "legacy-bsd-lib": ("BSD-3-Clause", "Permissive", 0, "yes", "declared", "npm"),
    "zlib-lib": ("Zlib", "Permissive", 0, "yes", "declared", "npm"),
    "wtfpl-lib": ("WTFPL", "Permissive", 0, "yes", "declared", "npm"),
    "@corp/scoped-lib": ("BSD-2-Clause", "Permissive", 0, "yes", "declared", "npm"),
    "nested-unlicense": ("Unlicense", "Permissive", 0, "yes", "declared", "npm"),
    "dual-or-lib": ("GPL-2.0 OR MIT", "Permissive", 0, "yes", "declared", "npm"),
    "file-only-mit": ("MIT", "Permissive", 0, "yes", "licenseFile", "npm"),
    "lgpl-lib": ("LGPL-2.1", "WeakCopyleft", 50, "caution", "declared", "npm"),
    "mpl-lib": ("MPL-2.0", "WeakCopyleft", 40, "caution", "declared", "npm"),
    "epl-lib": ("EPL-2.0", "WeakCopyleft", 45, "caution", "declared", "npm"),
    "gpl-lib": ("GPL-3.0-only", "StrongCopyleft", 90, "restricted", "declared", "npm"),
    "agpl-lib": ("AGPL-3.0", "StrongCopyleft", 100, "restricted", "declared", "npm"),
    "strict-and-lib": ("MIT AND GPL-2.0", "StrongCopyleft", 90, "restricted", "declared", "npm"),
    "see-license-gpl": ("GPL-3.0", "StrongCopyleft", 90, "restricted", "licenseFile", "npm"),
    "no-license-lib": ("UNKNOWN", "Unknown", 60, "review", "none", "npm"),
    "proprietary-lib": ("Proprietary", "Unknown", 60, "review", "declared", "npm"),
    "mit-crate": ("MIT", "Permissive", 0, "yes", "declared", "cargo"),
    "gpl-crate": ("GPL-3.0-only", "StrongCopyleft", 90, "restricted", "declared", "cargo"),
    "filelicense-crate": ("LGPL-2.1", "WeakCopyleft", 50, "caution", "licenseFile", "cargo"),
}

EXPECTED_SUMMARY = {
    "total": 22,
    "permissive": 11,
    "weakCopyleft": 4,
    "strongCopyleft": 5,
    "unknown": 2,
}

failures = []


def check(condition, message):
    if condition:
        print(f"  ok    {message}")
    else:
        print(f"  FAIL  {message}")
        failures.append(message)


def run_cli(*args):
    return subprocess.run(
        ["node", CLI, *args],
        capture_output=True,
        text=True,
        cwd=ROOT,
    )


def main():
    ensure_fixture()
    print("== JSON scan ==")
    result = run_cli("--json", "--path", FIXTURE)
    check(result.returncode == 0, f"exit code 0 (got {result.returncode})")
    report = json.loads(result.stdout)
    check(report["project"] == "test-project", "project name detected")
    check(report["summary"] == EXPECTED_SUMMARY, f"summary matches {EXPECTED_SUMMARY}")

    packages = {p["name"]: p for p in report["packages"]}
    check(len(packages) == len(EXPECTED), f"{len(EXPECTED)} unique packages found")
    check("test-project" not in packages, "workspace member itself excluded")

    for name, (license_, category, score, commercial, source, eco) in sorted(EXPECTED.items()):
        pkg = packages.get(name)
        if pkg is None:
            check(False, f"{name}: present in report")
            continue
        actual = (
            pkg["license"],
            pkg["category"],
            pkg["riskScore"],
            pkg["commercialUse"],
            pkg["licenseSource"],
            pkg["ecosystem"],
        )
        check(
            actual == (license_, category, score, commercial, source, eco),
            f"{name}: {actual}",
        )

    def offender_listed(stderr, name):
        # Offender lines look like "  name@version (license)"; prefix-match
        # the full line so "gpl-lib" does not match inside "agpl-lib".
        return any(line.strip().startswith(f"{name}@") for line in stderr.splitlines())

    print("== --fail-on ==")
    result = run_cli("--json", "--path", FIXTURE, "--fail-on", "gpl")
    check(result.returncode == 1, "--fail-on gpl exits 1")
    for name in ["gpl-lib", "agpl-lib", "strict-and-lib", "see-license-gpl", "gpl-crate"]:
        check(offender_listed(result.stderr, name), f"--fail-on gpl lists {name}")
    check(not offender_listed(result.stderr, "lgpl-lib"), "--fail-on gpl does not list LGPL")
    check(
        not offender_listed(result.stderr, "dual-or-lib"),
        "--fail-on gpl does not list dual-licensed",
    )

    result = run_cli("--json", "--path", FIXTURE, "--fail-on", "agpl")
    check(result.returncode == 1, "--fail-on agpl exits 1")
    check(offender_listed(result.stderr, "agpl-lib"), "--fail-on agpl lists agpl-lib")
    check(
        not offender_listed(result.stderr, "gpl-lib"),
        "--fail-on agpl does not list plain GPL",
    )

    print("== --ignore ==")
    ignored = ["gpl-lib", "agpl-lib", "strict-and-lib", "see-license-gpl", "gpl-crate"]
    result = run_cli(
        "--json", "--path", FIXTURE, "--ignore", ",".join(ignored), "--fail-on", "gpl"
    )
    check(result.returncode == 0, "--ignore whitelists all strong copyleft, exit 0")
    report = json.loads(result.stdout)
    check(report["summary"]["total"] == 17, "summary recomputed after ignore")
    check(report["summary"]["strongCopyleft"] == 0, "no strong copyleft left")

    print("== --html ==")
    with tempfile.NamedTemporaryFile(suffix=".html", delete=False) as handle:
        html_path = handle.name
    try:
        result = run_cli("--html", html_path, "--path", FIXTURE)
        check(result.returncode == 0, "--html exits 0")
        with open(html_path, encoding="utf-8") as handle:
            html = handle.read()
        check("agpl-lib" in html, "HTML contains packages")
        check('id="search"' in html, "HTML is interactive (search box)")
        check('data-filter="strong"' in html, "HTML has category filter cards")
    finally:
        os.unlink(html_path)

    print()
    if failures:
        print(f"{len(failures)} FAILURE(S)")
        sys.exit(1)
    print("all checks passed")


if __name__ == "__main__":
    main()
