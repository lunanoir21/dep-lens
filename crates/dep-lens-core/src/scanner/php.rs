use std::fs;
use std::io;
use std::path::Path;

use crate::model::{DependencyType, Ecosystem, LicenseSource, Package};

/// Scan `composer.lock` for PHP dependencies.
pub fn scan(project_root: &Path) -> io::Result<Vec<Package>> {
    let lockfile = project_root.join("composer.lock");
    if !lockfile.is_file() {
        return Ok(Vec::new());
    }
    let raw_lock = fs::read_to_string(lockfile)?;

    let mut direct_deps = std::collections::HashSet::new();
    if let Ok(raw_json) = fs::read_to_string(project_root.join("composer.json")) {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw_json) {
            for key in ["require", "require-dev"] {
                if let Some(deps) = value.get(key).and_then(|v| v.as_object()) {
                    for name in deps.keys() {
                        direct_deps.insert(name.clone());
                    }
                }
            }
        }
    }

    let mut packages = parse_lockfile(&raw_lock).map_err(io::Error::other)?;
    for pkg in &mut packages {
        if direct_deps.contains(&pkg.name) {
            pkg.dependency_type = DependencyType::Direct;
        }
    }
    Ok(packages)
}

pub fn parse_lockfile(raw: &str) -> Result<Vec<Package>, String> {
    let value: serde_json::Value = serde_json::from_str(raw).map_err(|e| e.to_string())?;
    let mut packages = Vec::new();

    // composer.lock has "packages" (runtime) and "packages-dev" (dev)
    for key in ["packages", "packages-dev"] {
        if let Some(list) = value.get(key).and_then(|v| v.as_array()) {
            for pkg in list {
                let Some(name) = pkg.get("name").and_then(|v| v.as_str()) else {
                    continue;
                };
                let version = pkg
                    .get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or("0.0.0");

                let license = if let Some(licenses) = pkg.get("license").and_then(|v| v.as_array())
                {
                    let items: Vec<&str> = licenses.iter().filter_map(|v| v.as_str()).collect();
                    if items.is_empty() {
                        None
                    } else {
                        Some(items.join(" OR "))
                    }
                } else {
                    pkg.get("license")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                };

                let license_source = if license.is_some() {
                    LicenseSource::Declared
                } else {
                    LicenseSource::None
                };

                packages.push(Package {
                    name: name.to_string(),
                    version: version.to_string(),
                    license,
                    license_source,
                    ecosystem: Ecosystem::Php,
                    // composer.lock includes transitive dependencies in a flat list.
                    // To distinguish direct, we'd need composer.json.
                    // For now, default to Transitive, or we can read composer.json.
                    dependency_type: DependencyType::Transitive,
                });
            }
        }
    }

    Ok(packages)
}
