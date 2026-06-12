use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::model::{DependencyType, Ecosystem, LicenseSource, Package};
use crate::scanner::detect_license_in_dir;

/// Scan `go.mod` for Go dependencies. Licenses are identified from the
/// LICENSE files Go keeps in its module cache (`$GOPATH/pkg/mod`).
pub fn scan(project_root: &Path) -> io::Result<Vec<Package>> {
    let mod_file = project_root.join("go.mod");
    if !mod_file.is_file() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(mod_file)?;
    let mut packages = parse_go_mod(&raw);
    let cache = module_cache_dir();
    for pkg in &mut packages {
        if let Some(license) = detect_module_license(&cache, &pkg.name, &pkg.version) {
            pkg.license = Some(license);
            pkg.license_source = LicenseSource::LicenseFile;
        }
    }
    Ok(packages)
}

/// Module cache root: DEP_LENS_GOPATH override (used by tests and fixtures),
/// then GOPATH, then the Go default `~/go`.
fn module_cache_dir() -> PathBuf {
    let gopath = std::env::var("DEP_LENS_GOPATH")
        .or_else(|_| std::env::var("GOPATH"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::var("HOME")
                .map(|home| Path::new(&home).join("go"))
                .unwrap_or_else(|_| PathBuf::from("go"))
        });
    gopath.join("pkg").join("mod")
}

/// Escape a module path the way the Go module cache does: every uppercase
/// letter becomes `!` followed by its lowercase form (github.com/Azure ->
/// github.com/!azure).
pub fn escape_module_path(module: &str) -> String {
    let mut out = String::with_capacity(module.len());
    for c in module.chars() {
        if c.is_ascii_uppercase() {
            out.push('!');
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

fn detect_module_license(cache: &Path, module: &str, version: &str) -> Option<String> {
    let dir = cache.join(format!("{}@{}", escape_module_path(module), version));
    if !dir.is_dir() {
        return None;
    }
    detect_license_in_dir(&dir)
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
        } else if in_require && !trimmed.is_empty() && !trimmed.starts_with("//") {
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

#[cfg(test)]
mod tests {
    use super::*;

    const GO_MOD: &str = "module example.com/app\n\ngo 1.22\n\nrequire (\n\tgithub.com/pkg/errors v0.9.1\n\tgolang.org/x/text v0.14.0 // indirect\n)\n\nrequire github.com/stretchr/testify v1.9.0\n";

    #[test]
    fn parses_block_and_single_requires() {
        let packages = parse_go_mod(GO_MOD);
        let names: Vec<&str> = packages.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(
            names,
            vec![
                "github.com/pkg/errors",
                "golang.org/x/text",
                "github.com/stretchr/testify"
            ]
        );
        assert_eq!(packages[0].version, "v0.9.1");
        assert_eq!(packages[0].ecosystem, Ecosystem::Go);
    }

    #[test]
    fn indirect_modules_are_transitive() {
        let packages = parse_go_mod(GO_MOD);
        assert_eq!(packages[0].dependency_type, DependencyType::Direct);
        assert_eq!(packages[1].dependency_type, DependencyType::Transitive);
        assert_eq!(packages[2].dependency_type, DependencyType::Direct);
    }

    #[test]
    fn escapes_uppercase_in_module_paths() {
        assert_eq!(
            escape_module_path("github.com/Azure/azure-sdk"),
            "github.com/!azure/azure-sdk"
        );
        assert_eq!(escape_module_path("golang.org/x/text"), "golang.org/x/text");
        assert_eq!(
            escape_module_path("github.com/BurntSushi/toml"),
            "github.com/!burnt!sushi/toml"
        );
    }

    #[test]
    fn finds_license_in_fake_module_cache() {
        let root = std::env::temp_dir().join(format!("dep-lens-go-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let module_dir = root
            .join("pkg")
            .join("mod")
            .join("github.com")
            .join("pkg")
            .join("errors@v0.9.1");
        fs::create_dir_all(&module_dir).unwrap();
        fs::write(
            module_dir.join("LICENSE"),
            "Redistribution and use in source and binary forms, with or without modification...",
        )
        .unwrap();

        let cache = root.join("pkg").join("mod");
        assert_eq!(
            detect_module_license(&cache, "github.com/pkg/errors", "v0.9.1"),
            Some("BSD-2-Clause".to_string())
        );
        assert_eq!(
            detect_module_license(&cache, "github.com/missing/mod", "v1.0.0"),
            None
        );
        let _ = fs::remove_dir_all(&root);
    }
}
