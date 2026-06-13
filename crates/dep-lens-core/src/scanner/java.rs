use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::model::{DependencyType, Ecosystem, LicenseSource, Package};

/// Scan Maven `pom.xml` (and Gradle `gradle.lockfile` when present) for Java
/// dependencies. Licenses are read from the cached POMs in the local Maven
/// repository (`~/.m2/repository`, overridable via DEP_LENS_M2).
pub fn scan(project_root: &Path) -> io::Result<Vec<Package>> {
    let mut packages = Vec::new();

    let pom = project_root.join("pom.xml");
    if pom.is_file() {
        let raw = fs::read_to_string(pom)?;
        packages.extend(parse_pom_dependencies(&raw));
    }
    let gradle_lock = project_root.join("gradle.lockfile");
    if gradle_lock.is_file() {
        let raw = fs::read_to_string(gradle_lock)?;
        packages.extend(parse_gradle_lockfile(&raw));
    } else {
        // Most Gradle projects never enable dependency locking; fall back to
        // the coordinates declared directly in the build script.
        for name in ["build.gradle", "build.gradle.kts"] {
            let build_file = project_root.join(name);
            if build_file.is_file() {
                let raw = fs::read_to_string(build_file)?;
                packages.extend(parse_gradle_build_file(&raw));
                break;
            }
        }
    }
    if packages.is_empty() {
        return Ok(packages);
    }

    let repo = maven_repository();
    for pkg in &mut packages {
        if pkg.license.is_some() {
            continue;
        }
        if let Some(license) = license_from_m2(&repo, &pkg.name, &pkg.version) {
            pkg.license = Some(license);
            pkg.license_source = LicenseSource::Declared;
        }
    }

    // Gradle lockfiles repeat coordinates per configuration; deduplicate.
    packages.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.version.cmp(&b.version)));
    packages.dedup_by(|a, b| a.name == b.name && a.version == b.version);
    Ok(packages)
}

fn maven_repository() -> PathBuf {
    if let Ok(custom) = std::env::var("DEP_LENS_M2") {
        return PathBuf::from(custom);
    }
    super::home_dir()
        .map(|home| home.join(".m2").join("repository"))
        .unwrap_or_else(|| PathBuf::from(".m2/repository"))
}

/// Extract the text of the first `<tag>...</tag>` occurrence in `block`.
fn xml_tag<'a>(block: &'a str, tag: &str) -> Option<&'a str> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = block.find(&open)? + open.len();
    let end = block[start..].find(&close)? + start;
    Some(block[start..end].trim())
}

/// Parse `<dependency>` blocks from a pom.xml. Test-scoped dependencies are
/// still reported (they carry license obligations for distribution of test
/// tooling), but version-less managed dependencies are skipped.
pub fn parse_pom_dependencies(raw: &str) -> Vec<Package> {
    let mut packages = Vec::new();
    let mut rest = raw;
    while let Some(start) = rest.find("<dependency>") {
        let after = &rest[start..];
        let Some(end) = after.find("</dependency>") else {
            break;
        };
        let block = &after[..end];
        rest = &after[end + "</dependency>".len()..];

        let (Some(group), Some(artifact)) =
            (xml_tag(block, "groupId"), xml_tag(block, "artifactId"))
        else {
            continue;
        };
        // Versions managed by a parent BOM (or property placeholders) cannot
        // be resolved without full Maven; skip rather than guess.
        let Some(version) = xml_tag(block, "version") else {
            continue;
        };
        if version.contains("${") {
            continue;
        }
        packages.push(Package {
            name: format!("{group}:{artifact}"),
            version: version.to_string(),
            license: None,
            license_source: LicenseSource::None,
            ecosystem: Ecosystem::Java,
            dependency_type: DependencyType::Direct,
        });
    }
    packages
}

/// Parse Gradle's `gradle.lockfile`: `group:artifact:version=configurations`.
pub fn parse_gradle_lockfile(raw: &str) -> Vec<Package> {
    let mut packages = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("empty=") {
            continue;
        }
        let coordinates = trimmed.split('=').next().unwrap_or("");
        let parts: Vec<&str> = coordinates.split(':').collect();
        if parts.len() != 3 {
            continue;
        }
        packages.push(Package {
            name: format!("{}:{}", parts[0], parts[1]),
            version: parts[2].to_string(),
            license: None,
            license_source: LicenseSource::None,
            ecosystem: Ecosystem::Java,
            // The lockfile flattens the graph; direct info would need the
            // build script, so be conservative.
            dependency_type: DependencyType::Transitive,
        });
    }
    packages
}

/// Parse `group:artifact:version` coordinates out of quoted strings in a
/// Gradle build script (Groovy or Kotlin DSL), e.g.
/// `implementation("com.squareup.okhttp3:okhttp:4.12.0")` or
/// `testImplementation 'org.junit.jupiter:junit-jupiter:5.10.0'`. Entries
/// using version catalogs or variable interpolation (`libs.guava`,
/// `"...:$version"`) have no literal version and are skipped.
pub fn parse_gradle_build_file(raw: &str) -> Vec<Package> {
    let mut packages = Vec::new();
    for quote in ['"', '\''] {
        let mut rest = raw;
        while let Some(start) = rest.find(quote) {
            let after = &rest[start + 1..];
            let Some(end) = after.find(quote) else { break };
            let candidate = &after[..end];
            rest = &after[end + 1..];

            let parts: Vec<&str> = candidate.split(':').collect();
            if parts.len() == 3
                && parts
                    .iter()
                    .all(|p| !p.is_empty() && !p.contains(['$', '{', '}']))
            {
                packages.push(Package {
                    name: format!("{}:{}", parts[0], parts[1]),
                    version: parts[2].to_string(),
                    license: None,
                    license_source: LicenseSource::None,
                    ecosystem: Ecosystem::Java,
                    dependency_type: DependencyType::Direct,
                });
            }
        }
    }
    packages
}

/// License name from the dependency's POM in the local Maven repository:
/// `<repo>/<group as path>/<artifact>/<version>/<artifact>-<version>.pom`.
fn license_from_m2(repo: &Path, coordinates: &str, version: &str) -> Option<String> {
    let (group, artifact) = coordinates.split_once(':')?;
    let mut path = repo.to_path_buf();
    for segment in group.split('.') {
        path = path.join(segment);
    }
    let pom = path
        .join(artifact)
        .join(version)
        .join(format!("{artifact}-{version}.pom"));
    let raw = fs::read_to_string(pom).ok()?;
    parse_pom_license(&raw)
}

/// First `<license><name>...</name></license>` entry of a POM.
pub fn parse_pom_license(raw: &str) -> Option<String> {
    let licenses_block = {
        let start = raw.find("<licenses>")?;
        let end = raw[start..].find("</licenses>")? + start;
        &raw[start..end]
    };
    xml_tag(licenses_block, "name")
        .map(str::to_string)
        .filter(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    const POM: &str = r#"<project>
  <dependencies>
    <dependency>
      <groupId>com.google.guava</groupId>
      <artifactId>guava</artifactId>
      <version>33.0.0-jre</version>
    </dependency>
    <dependency>
      <groupId>org.junit.jupiter</groupId>
      <artifactId>junit-jupiter</artifactId>
      <version>5.10.0</version>
      <scope>test</scope>
    </dependency>
    <dependency>
      <groupId>org.managed</groupId>
      <artifactId>no-version</artifactId>
    </dependency>
    <dependency>
      <groupId>org.props</groupId>
      <artifactId>prop-version</artifactId>
      <version>${some.version}</version>
    </dependency>
  </dependencies>
</project>"#;

    #[test]
    fn parses_pom_dependencies() {
        let packages = parse_pom_dependencies(POM);
        let names: Vec<&str> = packages.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(
            names,
            vec!["com.google.guava:guava", "org.junit.jupiter:junit-jupiter"]
        );
        assert_eq!(packages[0].version, "33.0.0-jre");
        assert_eq!(packages[0].ecosystem, Ecosystem::Java);
        assert_eq!(packages[0].dependency_type, DependencyType::Direct);
    }

    #[test]
    fn parses_gradle_lockfile() {
        let raw = "# This is a Gradle generated file\ncom.squareup.okhttp3:okhttp:4.12.0=compileClasspath,runtimeClasspath\norg.jetbrains.kotlin:kotlin-stdlib:1.9.22=compileClasspath\nempty=annotationProcessor\n";
        let packages = parse_gradle_lockfile(raw);
        let names: Vec<&str> = packages.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(
            names,
            vec![
                "com.squareup.okhttp3:okhttp",
                "org.jetbrains.kotlin:kotlin-stdlib"
            ]
        );
        assert_eq!(packages[0].version, "4.12.0");
    }

    #[test]
    fn parses_gradle_build_file_coordinates() {
        let raw = r#"
dependencies {
    implementation("com.squareup.okhttp3:okhttp:4.12.0")
    testImplementation 'org.junit.jupiter:junit-jupiter:5.10.0'
    implementation libs.guava
    implementation project(':core')
    implementation("com.example:versioned:$appVersion")
}
"#;
        let packages = parse_gradle_build_file(raw);
        let names: Vec<&str> = packages.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(
            names,
            vec![
                "com.squareup.okhttp3:okhttp",
                "org.junit.jupiter:junit-jupiter"
            ]
        );
        assert_eq!(packages[0].version, "4.12.0");
        assert_eq!(packages[0].dependency_type, DependencyType::Direct);
    }

    #[test]
    fn parses_pom_license_block() {
        let raw = "<project><licenses><license><name>The Apache Software License, Version 2.0</name><url>x</url></license></licenses></project>";
        assert_eq!(
            parse_pom_license(raw),
            Some("The Apache Software License, Version 2.0".to_string())
        );
        assert_eq!(parse_pom_license("<project></project>"), None);
    }

    #[test]
    fn reads_license_from_fake_m2() {
        let repo = std::env::temp_dir().join(format!("dep-lens-m2-{}", std::process::id()));
        let _ = fs::remove_dir_all(&repo);
        let dir = repo
            .join("com")
            .join("google")
            .join("guava")
            .join("guava")
            .join("33.0.0-jre");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("guava-33.0.0-jre.pom"),
            "<project><licenses><license><name>Apache License, Version 2.0</name></license></licenses></project>",
        )
        .unwrap();

        assert_eq!(
            license_from_m2(&repo, "com.google.guava:guava", "33.0.0-jre"),
            Some("Apache License, Version 2.0".to_string())
        );
        assert_eq!(license_from_m2(&repo, "org.missing:artifact", "1.0"), None);
        let _ = fs::remove_dir_all(&repo);
    }
}
