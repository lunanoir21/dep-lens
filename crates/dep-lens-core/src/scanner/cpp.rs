use std::fs;
use std::io;
use std::path::Path;

use crate::license::classifier::identify_license_text;
use crate::model::{DependencyType, Ecosystem, LicenseSource, Package};

/// Scan C/C++ dependencies from `vcpkg.json` and `conanfile.txt`. vcpkg
/// licenses come from the manifest's own `license` field (newer ports) or
/// the installed port's `copyright` file under `vcpkg_installed/`.
/// Conan's conanfile carries no license info, so those stay Unknown.
pub fn scan(project_root: &Path) -> io::Result<Vec<Package>> {
    let mut packages = Vec::new();

    let vcpkg = project_root.join("vcpkg.json");
    if vcpkg.is_file() {
        let raw = fs::read_to_string(vcpkg)?;
        let mut found = parse_vcpkg_manifest(&raw).map_err(io::Error::other)?;
        for pkg in &mut found {
            if pkg.license.is_none() {
                if let Some(license) = vcpkg_copyright_license(project_root, &pkg.name) {
                    pkg.license = Some(license);
                    pkg.license_source = LicenseSource::LicenseFile;
                }
            }
        }
        packages.extend(found);
    }

    let conan = project_root.join("conanfile.txt");
    if conan.is_file() {
        let raw = fs::read_to_string(conan)?;
        packages.extend(parse_conanfile(&raw));
    }

    Ok(packages)
}

/// Dependencies from a vcpkg manifest: either plain strings or objects with
/// a `name` field. Versions are rarely pinned in the manifest; use the
/// `version>=` constraint when present, otherwise mark as unpinned.
pub fn parse_vcpkg_manifest(raw: &str) -> Result<Vec<Package>, String> {
    let value: serde_json::Value =
        serde_json::from_str(raw).map_err(|e| format!("invalid vcpkg.json: {e}"))?;
    let mut packages = Vec::new();
    if let Some(deps) = value.get("dependencies").and_then(|d| d.as_array()) {
        for dep in deps {
            let (name, version) = match dep {
                serde_json::Value::String(s) => (s.as_str(), "unpinned".to_string()),
                serde_json::Value::Object(map) => {
                    let Some(name) = map.get("name").and_then(|n| n.as_str()) else {
                        continue;
                    };
                    let version = map
                        .get("version>=")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unpinned")
                        .to_string();
                    (name, version)
                }
                _ => continue,
            };
            packages.push(Package {
                name: name.to_string(),
                version,
                license: None,
                license_source: LicenseSource::None,
                ecosystem: Ecosystem::Cpp,
                dependency_type: DependencyType::Direct,
            });
        }
    }
    Ok(packages)
}

/// vcpkg installs each port's license text as
/// `vcpkg_installed/<triplet>/share/<port>/copyright`.
fn vcpkg_copyright_license(project_root: &Path, port: &str) -> Option<String> {
    let installed = project_root.join("vcpkg_installed");
    let entries = fs::read_dir(&installed).ok()?;
    for entry in entries.flatten() {
        let copyright = entry.path().join("share").join(port).join("copyright");
        if !copyright.is_file() {
            continue;
        }
        if let Ok(raw) = fs::read_to_string(&copyright) {
            if let Some(id) = identify_license_text(&raw) {
                return Some(id.to_string());
            }
        }
    }
    None
}

/// `[requires]` entries of a conanfile.txt: `name/version[@user/channel]`.
pub fn parse_conanfile(raw: &str) -> Vec<Package> {
    let mut packages = Vec::new();
    let mut in_requires = false;
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_requires = trimmed == "[requires]" || trimmed == "[tool_requires]";
            continue;
        }
        if !in_requires || trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let reference = trimmed.split('@').next().unwrap_or(trimmed);
        let Some((name, version)) = reference.split_once('/') else {
            continue;
        };
        packages.push(Package {
            name: name.trim().to_string(),
            version: version.trim().to_string(),
            license: None,
            license_source: LicenseSource::None,
            ecosystem: Ecosystem::Cpp,
            dependency_type: DependencyType::Direct,
        });
    }
    packages
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_vcpkg_string_and_object_dependencies() {
        let raw = r#"{
            "name": "demo",
            "dependencies": [
                "fmt",
                {"name": "boost-asio", "version>=": "1.84.0"},
                {"name": "zlib", "platform": "!windows"}
            ]
        }"#;
        let packages = parse_vcpkg_manifest(raw).unwrap();
        let names: Vec<&str> = packages.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["fmt", "boost-asio", "zlib"]);
        assert_eq!(packages[1].version, "1.84.0");
        assert_eq!(packages[0].version, "unpinned");
        assert_eq!(packages[0].ecosystem, Ecosystem::Cpp);
    }

    #[test]
    fn parses_conanfile_requires() {
        let raw = "[requires]\nzlib/1.3\nfmt/10.2.1@user/stable\n\n[generators]\nCMakeDeps\n\n[tool_requires]\ncmake/3.28.1\n";
        let packages = parse_conanfile(raw);
        let names: Vec<&str> = packages.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["zlib", "fmt", "cmake"]);
        assert_eq!(packages[0].version, "1.3");
        assert_eq!(packages[1].version, "10.2.1");
    }

    #[test]
    fn rejects_invalid_vcpkg_json() {
        assert!(parse_vcpkg_manifest("nope").is_err());
    }

    #[test]
    fn finds_vcpkg_copyright_in_installed_tree() {
        let root = std::env::temp_dir().join(format!("dep-lens-vcpkg-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let share = root
            .join("vcpkg_installed")
            .join("x64-linux")
            .join("share")
            .join("fmt");
        fs::create_dir_all(&share).unwrap();
        fs::write(
            share.join("copyright"),
            "MIT License\n\nPermission is hereby granted, free of charge...",
        )
        .unwrap();

        assert_eq!(
            vcpkg_copyright_license(&root, "fmt"),
            Some("MIT".to_string())
        );
        assert_eq!(vcpkg_copyright_license(&root, "missing"), None);
        let _ = fs::remove_dir_all(&root);
    }
}
