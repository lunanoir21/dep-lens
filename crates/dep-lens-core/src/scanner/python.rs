use std::fs;
use std::io;
use std::path::Path;

use crate::model::{DependencyType, Ecosystem, LicenseSource, Package};

/// Scan `poetry.lock` or `Pipfile.lock` for Python dependencies.
pub fn scan(project_root: &Path) -> io::Result<Vec<Package>> {
    let poetry_lock = project_root.join("poetry.lock");
    if poetry_lock.is_file() {
        let raw = fs::read_to_string(poetry_lock)?;
        return Ok(parse_poetry_lock(&raw));
    }

    let pipfile_lock = project_root.join("Pipfile.lock");
    if pipfile_lock.is_file() {
        let raw = fs::read_to_string(pipfile_lock)?;
        return parse_pipfile_lock(&raw).map_err(io::Error::other);
    }

    Ok(Vec::new())
}

fn parse_poetry_lock(raw: &str) -> Vec<Package> {
    let mut packages = Vec::new();
    let mut current_name = None;
    let mut current_version = None;

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed == "[[package]]" {
            if let (Some(name), Some(version)) = (current_name.take(), current_version.take()) {
                packages.push(Package {
                    name,
                    version,
                    license: None,
                    license_source: LicenseSource::None,
                    ecosystem: Ecosystem::Python,
                    dependency_type: DependencyType::Transitive,
                });
            }
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("name = ") {
            current_name = Some(rest.trim_matches('"').to_string());
        } else if let Some(rest) = trimmed.strip_prefix("version = ") {
            current_version = Some(rest.trim_matches('"').to_string());
        }
    }
    if let (Some(name), Some(version)) = (current_name, current_version) {
        packages.push(Package {
            name,
            version,
            license: None,
            license_source: LicenseSource::None,
            ecosystem: Ecosystem::Python,
            dependency_type: DependencyType::Transitive,
        });
    }
    packages
}

fn parse_pipfile_lock(raw: &str) -> Result<Vec<Package>, String> {
    let value: serde_json::Value = serde_json::from_str(raw).map_err(|e| e.to_string())?;
    let mut packages = Vec::new();

    for section in ["default", "develop"] {
        if let Some(deps) = value.get(section).and_then(|v| v.as_object()) {
            for (name, info) in deps {
                let version = info
                    .get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .trim_start_matches("==")
                    .to_string();
                packages.push(Package {
                    name: name.clone(),
                    version,
                    license: None,
                    license_source: LicenseSource::None,
                    ecosystem: Ecosystem::Python,
                    dependency_type: DependencyType::Transitive,
                });
            }
        }
    }

    Ok(packages)
}
