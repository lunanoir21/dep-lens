use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::Path;

use crate::license::classifier::identify_license_text;
use crate::model::{Ecosystem, LicenseSource, Package};

/// File names probed when a package declares no license in its manifest.
const LICENSE_FILE_NAMES: &[&str] = &[
    "LICENSE",
    "LICENSE.md",
    "LICENSE.txt",
    "LICENCE",
    "LICENCE.md",
    "COPYING",
    "LICENSE-MIT",
    "UNLICENSE",
];

/// Scan `<project_root>/node_modules` recursively, reading every package's
/// `package.json` for its license declaration. Returns an empty list when no
/// `node_modules` directory exists. Packages are deduplicated by
/// `name@version` and sorted by name.
pub fn scan(project_root: &Path) -> io::Result<Vec<Package>> {
    let node_modules = project_root.join("node_modules");
    let mut seen = HashSet::new();
    let mut packages = Vec::new();
    if node_modules.is_dir() {
        scan_node_modules(&node_modules, &mut seen, &mut packages)?;
    }
    packages.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.version.cmp(&b.version)));
    Ok(packages)
}

fn scan_node_modules(
    dir: &Path,
    seen: &mut HashSet<String>,
    out: &mut Vec<Package>,
) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with('.') {
            continue;
        }
        if name.starts_with('@') {
            // Scoped registry namespace: actual packages live one level down.
            for scoped in fs::read_dir(&path)? {
                let scoped = scoped?;
                if scoped.path().is_dir() {
                    visit_package(&scoped.path(), seen, out)?;
                }
            }
        } else {
            visit_package(&path, seen, out)?;
        }
    }
    Ok(())
}

fn visit_package(
    pkg_dir: &Path,
    seen: &mut HashSet<String>,
    out: &mut Vec<Package>,
) -> io::Result<()> {
    let manifest = pkg_dir.join("package.json");
    if manifest.is_file() {
        // Unreadable or malformed manifests are skipped rather than fatal:
        // node_modules trees routinely contain broken leftovers.
        if let Ok(raw) = fs::read_to_string(&manifest) {
            if let Some(mut pkg) = parse_manifest(&raw) {
                if needs_license_file_fallback(pkg.license.as_deref()) {
                    if let Some(detected) = detect_license_file(pkg_dir) {
                        pkg.license = Some(detected);
                        pkg.license_source = LicenseSource::LicenseFile;
                    }
                }
                if seen.insert(format!("{}@{}", pkg.name, pkg.version)) {
                    out.push(pkg);
                }
            }
        }
    }
    let nested = pkg_dir.join("node_modules");
    if nested.is_dir() {
        scan_node_modules(&nested, seen, out)?;
    }
    Ok(())
}

/// `true` when the declared license is absent or is a placeholder like
/// `SEE LICENSE IN LICENSE.txt`, in which case the actual LICENSE file is
/// worth inspecting.
fn needs_license_file_fallback(license: Option<&str>) -> bool {
    match license {
        None => true,
        Some(value) => value.trim().to_uppercase().starts_with("SEE LICENSE"),
    }
}

/// Probe well-known license file names and identify the license from the
/// file contents.
fn detect_license_file(pkg_dir: &Path) -> Option<String> {
    for name in LICENSE_FILE_NAMES {
        let path = pkg_dir.join(name);
        if !path.is_file() {
            continue;
        }
        if let Ok(raw) = fs::read_to_string(&path) {
            if let Some(id) = identify_license_text(&raw) {
                return Some(id.to_string());
            }
        }
    }
    None
}

/// Parse one `package.json`. Returns `None` when the JSON is invalid or has
/// no usable `name` field.
pub fn parse_manifest(raw: &str) -> Option<Package> {
    let value: serde_json::Value = serde_json::from_str(raw).ok()?;
    let name = value.get("name")?.as_str()?.trim();
    if name.is_empty() {
        return None;
    }
    let version = value
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("0.0.0")
        .to_string();
    let license = extract_license(&value);
    let license_source = if license.is_some() {
        LicenseSource::Declared
    } else {
        LicenseSource::None
    };
    Some(Package {
        name: name.to_string(),
        version,
        license,
        license_source,
        ecosystem: Ecosystem::Npm,
    })
}

/// Extract the license declaration, covering the modern string form, the
/// deprecated `{ "type": ... }` object form, and the legacy `licenses` array.
fn extract_license(value: &serde_json::Value) -> Option<String> {
    match value.get("license") {
        Some(serde_json::Value::String(s)) if !s.trim().is_empty() => {
            return Some(s.trim().to_string());
        }
        Some(serde_json::Value::Object(map)) => {
            if let Some(t) = map.get("type").and_then(|t| t.as_str()) {
                if !t.trim().is_empty() {
                    return Some(t.trim().to_string());
                }
            }
        }
        _ => {}
    }
    if let Some(serde_json::Value::Array(items)) = value.get("licenses") {
        let types: Vec<&str> = items
            .iter()
            .filter_map(|item| item.get("type").and_then(|t| t.as_str()))
            .filter(|t| !t.trim().is_empty())
            .collect();
        if !types.is_empty() {
            return Some(types.join(" OR "));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write_pkg(dir: &Path, name: &str, json: &str) {
        let pkg_dir = dir.join(name);
        fs::create_dir_all(&pkg_dir).unwrap();
        fs::write(pkg_dir.join("package.json"), json).unwrap();
    }

    fn temp_project(tag: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("dep-lens-npm-test-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("node_modules")).unwrap();
        dir
    }

    #[test]
    fn parses_string_license() {
        let pkg = parse_manifest(r#"{"name":"a","version":"1.0.0","license":"MIT"}"#).unwrap();
        assert_eq!(pkg.name, "a");
        assert_eq!(pkg.version, "1.0.0");
        assert_eq!(pkg.license.as_deref(), Some("MIT"));
        assert_eq!(pkg.ecosystem, Ecosystem::Npm);
    }

    #[test]
    fn parses_object_license() {
        let pkg = parse_manifest(
            r#"{"name":"b","version":"2.0.0","license":{"type":"Apache-2.0","url":"x"}}"#,
        )
        .unwrap();
        assert_eq!(pkg.license.as_deref(), Some("Apache-2.0"));
    }

    #[test]
    fn parses_legacy_licenses_array() {
        let pkg = parse_manifest(
            r#"{"name":"c","version":"3.0.0","licenses":[{"type":"MIT"},{"type":"GPL-2.0"}]}"#,
        )
        .unwrap();
        assert_eq!(pkg.license.as_deref(), Some("MIT OR GPL-2.0"));
    }

    #[test]
    fn missing_license_is_none() {
        let pkg = parse_manifest(r#"{"name":"d","version":"1.0.0"}"#).unwrap();
        assert_eq!(pkg.license, None);
        assert_eq!(pkg.license_source, LicenseSource::None);
    }

    #[test]
    fn declared_license_sets_declared_source() {
        let pkg = parse_manifest(r#"{"name":"d","version":"1.0.0","license":"MIT"}"#).unwrap();
        assert_eq!(pkg.license_source, LicenseSource::Declared);
    }

    #[test]
    fn license_file_fallback_detects_mit() {
        let project = temp_project("file-fallback");
        let nm = project.join("node_modules");
        write_pkg(
            &nm,
            "file-only",
            r#"{"name":"file-only","version":"1.0.0"}"#,
        );
        fs::write(
            nm.join("file-only").join("LICENSE"),
            "MIT License\n\nPermission is hereby granted, free of charge...",
        )
        .unwrap();

        let packages = scan(&project).unwrap();
        let pkg = packages.iter().find(|p| p.name == "file-only").unwrap();
        assert_eq!(pkg.license.as_deref(), Some("MIT"));
        assert_eq!(pkg.license_source, LicenseSource::LicenseFile);

        let _ = fs::remove_dir_all(&project);
    }

    #[test]
    fn see_license_placeholder_triggers_file_fallback() {
        let project = temp_project("see-license");
        let nm = project.join("node_modules");
        write_pkg(
            &nm,
            "see-pkg",
            r#"{"name":"see-pkg","version":"1.0.0","license":"SEE LICENSE IN LICENSE.txt"}"#,
        );
        fs::write(
            nm.join("see-pkg").join("LICENSE.txt"),
            "GNU GENERAL PUBLIC LICENSE\nVersion 3, 29 June 2007",
        )
        .unwrap();

        let packages = scan(&project).unwrap();
        let pkg = packages.iter().find(|p| p.name == "see-pkg").unwrap();
        assert_eq!(pkg.license.as_deref(), Some("GPL-3.0"));
        assert_eq!(pkg.license_source, LicenseSource::LicenseFile);

        let _ = fs::remove_dir_all(&project);
    }

    #[test]
    fn unidentifiable_license_file_keeps_placeholder() {
        let project = temp_project("opaque-file");
        let nm = project.join("node_modules");
        write_pkg(&nm, "opaque", r#"{"name":"opaque","version":"1.0.0"}"#);
        fs::write(nm.join("opaque").join("LICENSE"), "All rights reserved.").unwrap();

        let packages = scan(&project).unwrap();
        let pkg = packages.iter().find(|p| p.name == "opaque").unwrap();
        assert_eq!(pkg.license, None);
        assert_eq!(pkg.license_source, LicenseSource::None);

        let _ = fs::remove_dir_all(&project);
    }

    #[test]
    fn invalid_json_is_skipped() {
        assert!(parse_manifest("not json").is_none());
        assert!(parse_manifest(r#"{"version":"1.0.0"}"#).is_none());
    }

    #[test]
    fn scans_flat_scoped_and_nested_packages() {
        let project = temp_project("walk");
        let nm = project.join("node_modules");
        write_pkg(
            &nm,
            "alpha",
            r#"{"name":"alpha","version":"1.0.0","license":"MIT"}"#,
        );
        write_pkg(
            &nm,
            "@scope/beta",
            r#"{"name":"@scope/beta","version":"2.0.0","license":"GPL-3.0"}"#,
        );
        write_pkg(
            &nm.join("alpha").join("node_modules"),
            "gamma",
            r#"{"name":"gamma","version":"0.1.0","license":"ISC"}"#,
        );
        // Duplicate name@version in a nested tree must be deduplicated.
        write_pkg(
            &nm.join("alpha").join("node_modules"),
            "alpha-dup",
            r#"{"name":"alpha","version":"1.0.0","license":"MIT"}"#,
        );
        fs::create_dir_all(nm.join(".bin")).unwrap();

        let packages = scan(&project).unwrap();
        let names: Vec<&str> = packages.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["@scope/beta", "alpha", "gamma"]);

        let _ = fs::remove_dir_all(&project);
    }

    #[test]
    fn missing_node_modules_yields_empty_list() {
        let dir = std::env::temp_dir().join(format!("dep-lens-npm-empty-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        assert!(scan(&dir).unwrap().is_empty());
        let _ = fs::remove_dir_all(&dir);
    }
}
