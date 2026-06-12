use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::model::{DependencyType, Ecosystem, LicenseSource, Package};

/// Scan Python dependencies from `poetry.lock`, `Pipfile.lock`, or
/// `requirements.txt` (in that order of preference). Licenses are read from
/// the `*.dist-info/METADATA` files of an installed environment
/// (`.venv`/`venv` in the project, or DEP_LENS_SITE_PACKAGES).
pub fn scan(project_root: &Path) -> io::Result<Vec<Package>> {
    let mut packages = read_lockfile(project_root)?;
    if packages.is_empty() {
        return Ok(packages);
    }

    let direct = direct_dependencies(project_root);
    let site_packages = find_site_packages(project_root);
    for pkg in &mut packages {
        if direct.contains(&normalize_name(&pkg.name)) {
            pkg.dependency_type = DependencyType::Direct;
        }
        if pkg.license.is_none() {
            for dir in &site_packages {
                if let Some(license) = license_from_dist_info(dir, &pkg.name, &pkg.version) {
                    pkg.license = Some(license);
                    pkg.license_source = LicenseSource::Declared;
                    break;
                }
            }
        }
    }
    Ok(packages)
}

fn read_lockfile(project_root: &Path) -> io::Result<Vec<Package>> {
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
    let requirements = project_root.join("requirements.txt");
    if requirements.is_file() {
        let raw = fs::read_to_string(requirements)?;
        return Ok(parse_requirements(&raw));
    }
    Ok(Vec::new())
}

/// PyPI treats `-`, `_`, and `.` as equivalent and names as case-insensitive.
fn normalize_name(name: &str) -> String {
    name.to_lowercase().replace(['-', '.'], "_")
}

/// Direct dependency names from `pyproject.toml`: both the PEP 621
/// `[project] dependencies` array and Poetry's
/// `[tool.poetry.dependencies]` table.
fn direct_dependencies(project_root: &Path) -> HashSet<String> {
    let mut direct = HashSet::new();
    let Ok(raw) = fs::read_to_string(project_root.join("pyproject.toml")) else {
        return direct;
    };
    let mut section = String::new();
    let mut in_pep621_deps = false;
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            section = trimmed.trim_matches(['[', ']']).to_string();
            in_pep621_deps = false;
            continue;
        }
        if section == "project" && trimmed.starts_with("dependencies") && trimmed.contains('[') {
            in_pep621_deps = !trimmed.contains(']');
            for spec in extract_quoted(trimmed) {
                direct.insert(normalize_name(&requirement_name(&spec)));
            }
            continue;
        }
        if in_pep621_deps {
            if trimmed.contains(']') {
                in_pep621_deps = false;
            }
            for spec in extract_quoted(trimmed) {
                direct.insert(normalize_name(&requirement_name(&spec)));
            }
            continue;
        }
        if section == "tool.poetry.dependencies" || section == "tool.poetry.dev-dependencies" {
            if let Some((name, _)) = trimmed.split_once('=') {
                let name = name.trim();
                if !name.is_empty() && name != "python" {
                    direct.insert(normalize_name(name));
                }
            }
        }
    }
    direct
}

/// All double-quoted strings on a line.
fn extract_quoted(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = line;
    while let Some(start) = rest.find('"') {
        let after = &rest[start + 1..];
        let Some(end) = after.find('"') else { break };
        out.push(after[..end].to_string());
        rest = &after[end + 1..];
    }
    out
}

/// Package name from a PEP 508 requirement spec ("requests>=2.31,<3").
fn requirement_name(spec: &str) -> String {
    spec.chars()
        .take_while(|c| c.is_alphanumeric() || matches!(c, '-' | '_' | '.'))
        .collect()
}

/// Candidate site-packages directories: DEP_LENS_SITE_PACKAGES override,
/// then `.venv`/`venv` inside the project.
fn find_site_packages(project_root: &Path) -> Vec<PathBuf> {
    if let Ok(custom) = std::env::var("DEP_LENS_SITE_PACKAGES") {
        return vec![PathBuf::from(custom)];
    }
    let mut found = Vec::new();
    for venv in [".venv", "venv"] {
        let lib = project_root.join(venv).join("lib");
        let Ok(entries) = fs::read_dir(&lib) else {
            continue;
        };
        for entry in entries.flatten() {
            let site = entry.path().join("site-packages");
            if site.is_dir() {
                found.push(site);
            }
        }
    }
    found
}

/// License from `<name>-<version>.dist-info/METADATA`: prefers the
/// `License:` header, falls back to trove classifiers
/// (`Classifier: License :: OSI Approved :: MIT License`).
fn license_from_dist_info(site_packages: &Path, name: &str, version: &str) -> Option<String> {
    let dist_info = site_packages.join(format!("{}-{}.dist-info", normalize_name(name), version));
    let metadata = dist_info.join("METADATA");
    let raw = fs::read_to_string(metadata).ok()?;
    parse_metadata_license(&raw)
}

pub fn parse_metadata_license(raw: &str) -> Option<String> {
    let mut classifier_license = None;
    for line in raw.lines() {
        if line.is_empty() {
            break; // headers end at the first blank line
        }
        if let Some(value) = line.strip_prefix("License:") {
            let value = value.trim();
            if !value.is_empty() && !value.eq_ignore_ascii_case("unknown") {
                return Some(value.to_string());
            }
        }
        if let Some(value) = line.strip_prefix("Classifier: License ::") {
            let last = value.rsplit("::").next().map(str::trim).unwrap_or("");
            if !last.is_empty() && classifier_license.is_none() {
                classifier_license = Some(last.to_string());
            }
        }
    }
    classifier_license
}

fn parse_poetry_lock(raw: &str) -> Vec<Package> {
    let mut packages = Vec::new();
    let mut current_name = None;
    let mut current_version = None;

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed == "[[package]]" {
            if let (Some(name), Some(version)) = (current_name.take(), current_version.take()) {
                packages.push(python_package(name, version));
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
        packages.push(python_package(name, version));
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
                packages.push(python_package(name.clone(), version));
            }
        }
    }

    Ok(packages)
}

/// Parse `requirements.txt`: only pinned `name==version` lines yield
/// packages; comments, options, and unpinned specs are skipped.
pub fn parse_requirements(raw: &str) -> Vec<Package> {
    let mut packages = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('-') {
            continue;
        }
        let Some((name, version)) = trimmed.split_once("==") else {
            continue;
        };
        let version = version
            .split([';', ' ', '#'])
            .next()
            .unwrap_or("")
            .trim();
        let name = requirement_name(name.trim());
        if !name.is_empty() && !version.is_empty() {
            packages.push(python_package(name, version.to_string()));
        }
    }
    packages
}

fn python_package(name: String, version: String) -> Package {
    Package {
        name,
        version,
        license: None,
        license_source: LicenseSource::None,
        ecosystem: Ecosystem::Python,
        dependency_type: DependencyType::Transitive,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_poetry_lock_packages() {
        let raw = "[[package]]\nname = \"requests\"\nversion = \"2.31.0\"\n\n[[package]]\nname = \"flask\"\nversion = \"3.0.0\"\n";
        let packages = parse_poetry_lock(raw);
        let names: Vec<&str> = packages.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["requests", "flask"]);
        assert_eq!(packages[0].version, "2.31.0");
        assert_eq!(packages[0].ecosystem, Ecosystem::Python);
    }

    #[test]
    fn parses_requirements_txt() {
        let raw = "# comment\nrequests==2.31.0\nflask==3.0.0 ; python_version >= \"3.8\"\n-r other.txt\nunpinned>=1.0\n";
        let packages = parse_requirements(raw);
        let names: Vec<&str> = packages.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["requests", "flask"]);
        assert_eq!(packages[1].version, "3.0.0");
    }

    #[test]
    fn metadata_license_header_wins() {
        let raw = "Metadata-Version: 2.1\nName: requests\nLicense: Apache-2.0\nClassifier: License :: OSI Approved :: MIT License\n\nBody text";
        assert_eq!(parse_metadata_license(raw), Some("Apache-2.0".to_string()));
    }

    #[test]
    fn metadata_falls_back_to_classifier() {
        let raw = "Metadata-Version: 2.1\nName: flask\nLicense: UNKNOWN\nClassifier: License :: OSI Approved :: BSD License\n\nBody";
        assert_eq!(parse_metadata_license(raw), Some("BSD License".to_string()));
    }

    #[test]
    fn metadata_without_license_yields_none() {
        let raw = "Metadata-Version: 2.1\nName: opaque\n\nLicense: MIT appears after blank line so ignored";
        assert_eq!(parse_metadata_license(raw), None);
    }

    #[test]
    fn normalizes_pypi_names() {
        assert_eq!(normalize_name("Flask-SQLAlchemy"), "flask_sqlalchemy");
        assert_eq!(normalize_name("zope.interface"), "zope_interface");
    }

    #[test]
    fn requirement_name_strips_version_spec() {
        assert_eq!(requirement_name("requests>=2.31,<3"), "requests");
        assert_eq!(requirement_name("Flask[async]==3.0"), "Flask");
    }
}
