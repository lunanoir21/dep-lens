use std::fs;
use std::io;
use std::path::Path;

use crate::model::{DependencyType, Ecosystem, LicenseSource, Package};

/// Scan `go.mod` for Go dependencies.
pub fn scan(project_root: &Path) -> io::Result<Vec<Package>> {
    let mod_file = project_root.join("go.mod");
    if !mod_file.is_file() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(mod_file)?;
    Ok(parse_go_mod(&raw))
}

pub fn parse_go_mod(raw: &str) -> Vec<Package> {
    let mut packages = Vec::new();
    let mut in_require = false;

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("require (") {
            in_require = true;
            continue;
        }
        if in_require && trimmed == ")" {
            in_require = false;
            continue;
        }

        if trimmed.starts_with("require ") && !in_require {
            let parts: Vec<&str> = trimmed["require ".len()..].split_whitespace().collect();
            if parts.len() >= 2 {
                packages.push(create_package(parts[0], parts[1], trimmed));
            }
        } else if in_require && !trimmed.is_empty() {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                packages.push(create_package(parts[0], parts[1], trimmed));
            }
        }
    }
    packages
}

fn create_package(name: &str, version: &str, line: &str) -> Package {
    let dependency_type = if line.contains("// indirect") {
        DependencyType::Transitive
    } else {
        DependencyType::Direct
    };
    Package {
        name: name.to_string(),
        version: version.to_string(),
        license: None,
        license_source: LicenseSource::None,
        ecosystem: Ecosystem::Go,
        dependency_type,
    }
}
