use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::model::{DependencyType, Ecosystem, LicenseSource, Package};
use crate::scanner::detect_license_in_dir;

/// Scan `pubspec.lock` for Dart/Flutter dependencies. Licenses are
/// identified from LICENSE files in the pub cache
/// (`~/.pub-cache/hosted/pub.dev`, overridable via DEP_LENS_PUB_CACHE).
pub fn scan(project_root: &Path) -> io::Result<Vec<Package>> {
    let lockfile = project_root.join("pubspec.lock");
    if !lockfile.is_file() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(lockfile)?;
    let mut packages = parse_pubspec_lock(&raw);

    let cache = pub_cache_hosted();
    for pkg in &mut packages {
        let dir = cache.join(format!("{}-{}", pkg.name, pkg.version));
        if let Some(license) = detect_license_in_dir(&dir) {
            pkg.license = Some(license);
            pkg.license_source = LicenseSource::LicenseFile;
        }
    }
    Ok(packages)
}

fn pub_cache_hosted() -> PathBuf {
    if let Ok(custom) = std::env::var("DEP_LENS_PUB_CACHE") {
        return PathBuf::from(custom).join("hosted").join("pub.dev");
    }
    let base = std::env::var("PUB_CACHE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            // Windows default is %LOCALAPPDATA%\Pub\Cache; everywhere else
            // it's ~/.pub-cache.
            std::env::var("LOCALAPPDATA")
                .map(|local| Path::new(&local).join("Pub").join("Cache"))
                .or_else(|_| {
                    super::home_dir()
                        .map(|home| home.join(".pub-cache"))
                        .ok_or(())
                })
                .unwrap_or_else(|_| PathBuf::from(".pub-cache"))
        });
    base.join("hosted").join("pub.dev")
}

/// Parse pubspec.lock, a stable subset of YAML:
///
/// ```text
/// packages:
///   http:
///     dependency: "direct main"
///     version: "1.2.0"
/// ```
pub fn parse_pubspec_lock(raw: &str) -> Vec<Package> {
    let mut packages = Vec::new();
    let mut in_packages = false;
    let mut current_name: Option<String> = None;
    let mut current_direct = false;
    let mut current_version: Option<String> = None;

    let flush = |name: &mut Option<String>,
                 version: &mut Option<String>,
                 direct: bool,
                 out: &mut Vec<Package>| {
        if let (Some(name), Some(version)) = (name.take(), version.take()) {
            out.push(Package {
                name,
                version,
                license: None,
                license_source: LicenseSource::None,
                ecosystem: Ecosystem::Dart,
                dependency_type: if direct {
                    DependencyType::Direct
                } else {
                    DependencyType::Transitive
                },
            });
        }
    };

    for line in raw.lines() {
        if line.trim_end() == "packages:" {
            in_packages = true;
            continue;
        }
        if !in_packages {
            continue;
        }
        // Top-level key after the packages block (e.g. "sdks:") ends it.
        if !line.starts_with(' ') && !line.trim().is_empty() {
            break;
        }
        let indent = line.len() - line.trim_start().len();
        let trimmed = line.trim();
        if indent == 2 && trimmed.ends_with(':') {
            flush(
                &mut current_name,
                &mut current_version,
                current_direct,
                &mut packages,
            );
            current_name = Some(trimmed.trim_end_matches(':').to_string());
            current_direct = false;
            current_version = None;
            continue;
        }
        if indent >= 4 {
            if let Some(rest) = trimmed.strip_prefix("dependency:") {
                current_direct = rest.contains("direct");
            } else if let Some(rest) = trimmed.strip_prefix("version:") {
                current_version = Some(rest.trim().trim_matches('"').to_string());
            }
        }
    }
    flush(
        &mut current_name,
        &mut current_version,
        current_direct,
        &mut packages,
    );
    packages
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOCK: &str = "packages:\n  http:\n    dependency: \"direct main\"\n    description:\n      name: http\n      sha256: \"abc\"\n    source: hosted\n    version: \"1.2.0\"\n  meta:\n    dependency: transitive\n    description:\n      name: meta\n    source: hosted\n    version: \"1.11.0\"\nsdks:\n  dart: \">=3.0.0 <4.0.0\"\n";

    #[test]
    fn parses_packages_with_dependency_kind() {
        let packages = parse_pubspec_lock(LOCK);
        let names: Vec<&str> = packages.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["http", "meta"]);
        assert_eq!(packages[0].version, "1.2.0");
        assert_eq!(packages[0].dependency_type, DependencyType::Direct);
        assert_eq!(packages[1].dependency_type, DependencyType::Transitive);
        assert_eq!(packages[0].ecosystem, Ecosystem::Dart);
    }

    #[test]
    fn stops_at_sdks_section() {
        let packages = parse_pubspec_lock(LOCK);
        assert_eq!(packages.len(), 2);
    }

    #[test]
    fn empty_lockfile_yields_nothing() {
        assert!(parse_pubspec_lock("sdks:\n  dart: \">=3.0.0\"\n").is_empty());
    }
}
