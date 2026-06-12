use std::collections::HashSet;
use std::io;
use std::path::Path;
use std::process::Command;

use crate::license::classifier::identify_license_text;
use crate::model::{Ecosystem, LicenseSource, Package};

/// Collect Cargo dependencies by invoking `cargo metadata --format-version 1`
/// against `<project_root>/Cargo.toml`. Returns an empty list when the project
/// has no `Cargo.toml`. Workspace members themselves are excluded so the
/// report only contains third-party dependencies.
pub fn scan(project_root: &Path) -> io::Result<Vec<Package>> {
    let manifest = project_root.join("Cargo.toml");
    if !manifest.is_file() {
        return Ok(Vec::new());
    }
    let output = Command::new("cargo")
        .arg("metadata")
        .arg("--format-version")
        .arg("1")
        .arg("--manifest-path")
        .arg(&manifest)
        .output()?;
    if !output.status.success() {
        return Err(io::Error::other(format!(
            "cargo metadata failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    parse_metadata(&String::from_utf8_lossy(&output.stdout)).map_err(io::Error::other)
}

/// Parse the JSON produced by `cargo metadata --format-version 1`.
/// Split from `scan` so it can be unit tested without running Cargo.
pub fn parse_metadata(raw: &str) -> Result<Vec<Package>, String> {
    let value: serde_json::Value =
        serde_json::from_str(raw).map_err(|e| format!("invalid cargo metadata JSON: {e}"))?;
    let members: HashSet<&str> = value
        .get("workspace_members")
        .and_then(|m| m.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();

    let mut direct_ids = HashSet::new();
    if let Some(nodes) = value
        .get("resolve")
        .and_then(|r| r.get("nodes"))
        .and_then(|n| n.as_array())
    {
        for node in nodes {
            if let Some(id) = node.get("id").and_then(|v| v.as_str()) {
                if members.contains(id) {
                    if let Some(deps) = node.get("dependencies").and_then(|d| d.as_array()) {
                        for dep_id in deps {
                            if let Some(s) = dep_id.as_str() {
                                direct_ids.insert(s);
                            }
                        }
                    }
                }
            }
        }
    }

    let mut packages = Vec::new();
    if let Some(list) = value.get("packages").and_then(|p| p.as_array()) {
        for pkg in list {
            let id = pkg.get("id").and_then(|v| v.as_str()).unwrap_or("");
            if members.contains(id) {
                continue;
            }
            let Some(name) = pkg.get("name").and_then(|v| v.as_str()) else {
                continue;
            };
            let version = pkg
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("0.0.0")
                .to_string();
            let mut license = pkg
                .get("license")
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string);
            let mut license_source = if license.is_some() {
                LicenseSource::Declared
            } else {
                LicenseSource::None
            };
            if license.is_none() {
                if let Some(detected) = detect_from_license_file(pkg) {
                    license = Some(detected);
                    license_source = LicenseSource::LicenseFile;
                }
            }
            let dependency_type = if direct_ids.contains(id) {
                crate::model::DependencyType::Direct
            } else {
                crate::model::DependencyType::Transitive
            };
            packages.push(Package {
                name: name.to_string(),
                version,
                license,
                license_source,
                ecosystem: Ecosystem::Cargo,
                dependency_type,
            });
        }
    }
    packages.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.version.cmp(&b.version)));
    Ok(packages)
}

/// Resolve `license_file` relative to the crate's manifest and identify the
/// license from its text. Returns `None` when the file is missing or the
/// text is not recognizable.
fn detect_from_license_file(pkg: &serde_json::Value) -> Option<String> {
    let license_file = pkg.get("license_file").and_then(|v| v.as_str())?;
    let manifest_path = pkg.get("manifest_path").and_then(|v| v.as_str())?;
    let path = Path::new(manifest_path).parent()?.join(license_file);
    let raw = std::fs::read_to_string(path).ok()?;
    identify_license_text(&raw).map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = r#"{
        "packages": [
            {"id": "registry+serde@1.0.0", "name": "serde", "version": "1.0.0", "license": "MIT OR Apache-2.0"},
            {"id": "registry+left-pad@0.9.0", "name": "left-pad", "version": "0.9.0", "license": null},
            {"id": "path+my-app@0.1.0", "name": "my-app", "version": "0.1.0", "license": "GPL-3.0"}
        ],
        "workspace_members": ["path+my-app@0.1.0"]
    }"#;

    #[test]
    fn parses_packages_and_excludes_workspace_members() {
        let packages = parse_metadata(FIXTURE).unwrap();
        let names: Vec<&str> = packages.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["left-pad", "serde"]);
    }

    #[test]
    fn captures_license_and_ecosystem() {
        let packages = parse_metadata(FIXTURE).unwrap();
        let serde_pkg = packages.iter().find(|p| p.name == "serde").unwrap();
        assert_eq!(serde_pkg.license.as_deref(), Some("MIT OR Apache-2.0"));
        assert_eq!(serde_pkg.ecosystem, Ecosystem::Cargo);
        let left_pad = packages.iter().find(|p| p.name == "left-pad").unwrap();
        assert_eq!(left_pad.license, None);
    }

    #[test]
    fn rejects_invalid_json() {
        assert!(parse_metadata("nope").is_err());
    }

    #[test]
    fn empty_metadata_yields_empty_list() {
        assert!(
            parse_metadata(r#"{"packages": [], "workspace_members": []}"#)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn missing_cargo_toml_yields_empty_list() {
        let dir = std::env::temp_dir().join(format!("dep-lens-cargo-none-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        assert!(scan(&dir).unwrap().is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
