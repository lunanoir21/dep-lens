use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::model::{DependencyType, Ecosystem, LicenseSource, Package};
use crate::scanner::detect_license_in_dir;

/// Scan `Gemfile.lock` for Ruby dependencies. Licenses come from gemspec
/// declarations or LICENSE files under `vendor/bundle` (or the directory
/// pointed at by DEP_LENS_GEM_HOME).
pub fn scan(project_root: &Path) -> io::Result<Vec<Package>> {
    let lockfile = project_root.join("Gemfile.lock");
    if !lockfile.is_file() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(lockfile)?;
    let mut packages = parse_lockfile(&raw);

    let gem_homes = find_gem_homes(project_root);
    for pkg in &mut packages {
        if pkg.license.is_some() {
            continue;
        }
        for home in &gem_homes {
            if let Some((license, source)) = detect_gem_license(home, &pkg.name, &pkg.version) {
                pkg.license = Some(license);
                pkg.license_source = source;
                break;
            }
        }
    }
    Ok(packages)
}

/// Bundler vendor directories that contain `gems/` and `specifications/`.
fn find_gem_homes(project_root: &Path) -> Vec<PathBuf> {
    if let Ok(custom) = std::env::var("DEP_LENS_GEM_HOME") {
        return vec![PathBuf::from(custom)];
    }
    let mut homes = Vec::new();
    let ruby_root = project_root.join("vendor").join("bundle").join("ruby");
    if let Ok(entries) = fs::read_dir(&ruby_root) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                homes.push(entry.path());
            }
        }
    }
    homes
}

/// License for one gem: the `license`/`licenses` declaration in its gemspec
/// under `specifications/`, falling back to LICENSE files in the unpacked
/// gem directory under `gems/`.
fn detect_gem_license(
    gem_home: &Path,
    name: &str,
    version: &str,
) -> Option<(String, LicenseSource)> {
    let gemspec = gem_home
        .join("specifications")
        .join(format!("{name}-{version}.gemspec"));
    if let Ok(raw) = fs::read_to_string(&gemspec) {
        if let Some(license) = parse_gemspec_license(&raw) {
            return Some((license, LicenseSource::Declared));
        }
    }
    let gem_dir = gem_home.join("gems").join(format!("{name}-{version}"));
    if gem_dir.is_dir() {
        if let Some(license) = detect_license_in_dir(&gem_dir) {
            return Some((license, LicenseSource::LicenseFile));
        }
    }
    None
}

/// Extract `s.license = "MIT"` or `s.licenses = ["MIT", "Ruby"]` from a
/// gemspec (also handles the `.freeze` suffix RubyGems emits).
pub fn parse_gemspec_license(raw: &str) -> Option<String> {
    for line in raw.lines() {
        let trimmed = line.trim();
        let Some(idx) = trimmed.find(".license") else {
            continue;
        };
        let rest = &trimmed[idx + ".license".len()..];
        let rest = rest.strip_prefix('s').unwrap_or(rest); // `.licenses`
        let Some((_, value)) = rest.split_once('=') else {
            continue;
        };
        let licenses: Vec<String> = value
            .split('"')
            .skip(1)
            .step_by(2)
            .map(str::to_string)
            .filter(|s| !s.is_empty())
            .collect();
        if !licenses.is_empty() {
            return Some(licenses.join(" OR "));
        }
    }
    None
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

#[cfg(test)]
mod tests {
    use super::*;

    const LOCKFILE: &str = "GEM\n  remote: https://rubygems.org/\n  specs:\n    rails (7.1.0)\n      rake (>= 12.2)\n    rake (13.1.0)\n\nPLATFORMS\n  ruby\n\nDEPENDENCIES\n  rails\n\nBUNDLED WITH\n   2.5.4\n";

    #[test]
    fn parses_specs_and_direct_dependencies() {
        let packages = parse_lockfile(LOCKFILE);
        let names: Vec<&str> = packages.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["rails", "rake"]);
        assert_eq!(packages[0].version, "7.1.0");
        assert_eq!(packages[0].dependency_type, DependencyType::Direct);
        assert_eq!(packages[1].dependency_type, DependencyType::Transitive);
    }

    #[test]
    fn parses_gemspec_single_license() {
        let raw = "Gem::Specification.new do |s|\n  s.name = \"rails\".freeze\n  s.license = \"MIT\".freeze\nend\n";
        assert_eq!(parse_gemspec_license(raw), Some("MIT".to_string()));
    }

    #[test]
    fn parses_gemspec_license_array() {
        let raw = "  s.licenses = [\"Ruby\".freeze, \"BSD-2-Clause\".freeze]\n";
        assert_eq!(
            parse_gemspec_license(raw),
            Some("Ruby OR BSD-2-Clause".to_string())
        );
    }

    #[test]
    fn gemspec_without_license_yields_none() {
        assert_eq!(parse_gemspec_license("s.name = \"x\""), None);
    }

    #[test]
    fn detects_license_from_vendor_bundle() {
        let root = std::env::temp_dir().join(format!("dep-lens-ruby-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let spec_dir = root.join("specifications");
        let gem_dir = root.join("gems").join("rake-13.1.0");
        fs::create_dir_all(&spec_dir).unwrap();
        fs::create_dir_all(&gem_dir).unwrap();
        fs::write(
            spec_dir.join("rails-7.1.0.gemspec"),
            "s.license = \"MIT\".freeze\n",
        )
        .unwrap();
        fs::write(
            gem_dir.join("MIT-LICENSE"),
            "MIT License\n\nPermission is hereby granted, free of charge...",
        )
        .unwrap();

        assert_eq!(
            detect_gem_license(&root, "rails", "7.1.0"),
            Some(("MIT".to_string(), LicenseSource::Declared))
        );
        assert_eq!(
            detect_gem_license(&root, "rake", "13.1.0"),
            Some(("MIT".to_string(), LicenseSource::LicenseFile))
        );
        assert_eq!(detect_gem_license(&root, "missing", "1.0.0"), None);
        let _ = fs::remove_dir_all(&root);
    }
}
