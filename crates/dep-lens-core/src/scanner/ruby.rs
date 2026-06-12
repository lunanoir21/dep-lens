use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::Path;

use crate::model::{DependencyType, Ecosystem, LicenseSource, Package};

/// Scan `Gemfile.lock` for Ruby dependencies.
pub fn scan(project_root: &Path) -> io::Result<Vec<Package>> {
    let lockfile = project_root.join("Gemfile.lock");
    if !lockfile.is_file() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(lockfile)?;
    Ok(parse_lockfile(&raw))
}

pub fn parse_lockfile(raw: &str) -> Vec<Package> {
    let mut packages = Vec::new();
    let mut direct_deps = HashSet::new();
    let mut in_specs = false;
    let mut in_dependencies = false;

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed == "specs:" {
            in_specs = true;
            in_dependencies = false;
            continue;
        }
        if trimmed == "DEPENDENCIES" {
            in_dependencies = true;
            in_specs = false;
            continue;
        }
        if trimmed.is_empty()
            || (line.starts_with(|c: char| !c.is_whitespace()) && !trimmed.ends_with(':'))
        {
            in_specs = false;
            if trimmed != "DEPENDENCIES" {
                in_dependencies = false;
            }
        }

        if in_specs && line.starts_with("    ") && !line.starts_with("      ") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                let name = parts[0].to_string();
                let version = parts[1].trim_matches(|c| c == '(' || c == ')').to_string();
                packages.push(Package {
                    name,
                    version,
                    license: None, // Gemfile.lock doesn't store licenses
                    license_source: LicenseSource::None,
                    ecosystem: Ecosystem::Ruby,
                    dependency_type: DependencyType::Transitive,
                });
            }
        }

        if in_dependencies && line.starts_with("  ") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if !parts.is_empty() {
                let name = parts[0].trim_matches('!').to_string();
                direct_deps.insert(name);
            }
        }
    }

    for pkg in &mut packages {
        if direct_deps.contains(&pkg.name) {
            pkg.dependency_type = DependencyType::Direct;
        }
    }

    packages
}
