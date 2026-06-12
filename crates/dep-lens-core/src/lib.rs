//! dep-lens core: scans npm and Cargo dependency trees, classifies licenses,
//! assigns commercial risk scores, and renders JSON/HTML reports.

pub mod license;
pub mod model;
pub mod report;
pub mod scanner;
pub mod util;

use model::{ClassifiedPackage, LicenseCategory, Package, Report, Summary};

/// Placeholder license string for packages with no usable declaration.
pub const UNKNOWN_LICENSE: &str = "UNKNOWN";

/// Classify raw packages, drop ignored names, and assemble the final report.
/// `scanned_at` is injected by the caller so this function stays deterministic.
pub fn build_report(
    project: &str,
    path: &str,
    scanned_at: &str,
    mut packages: Vec<Package>,
    ignore: &[String],
) -> Report {
    packages.retain(|p| !ignore.iter().any(|i| i.eq_ignore_ascii_case(&p.name)));

    let mut summary = Summary {
        total: packages.len(),
        ..Summary::default()
    };
    let classified: Vec<ClassifiedPackage> = packages
        .into_iter()
        .map(|pkg| {
            let category = license::classifier::classify(pkg.license.as_deref());
            match category {
                LicenseCategory::Permissive => summary.permissive += 1,
                LicenseCategory::WeakCopyleft => summary.weak_copyleft += 1,
                LicenseCategory::StrongCopyleft => summary.strong_copyleft += 1,
                LicenseCategory::Unknown => summary.unknown += 1,
            }
            let license_str = pkg.license.unwrap_or_else(|| UNKNOWN_LICENSE.to_string());
            let assessment = license::risk::assess(category, &license_str);
            ClassifiedPackage {
                name: pkg.name,
                version: pkg.version,
                license: license_str,
                license_source: pkg.license_source,
                ecosystem: pkg.ecosystem,
                dependency_type: pkg.dependency_type,
                category,
                risk_score: assessment.score,
                risk_level: assessment.level,
                commercial_use: assessment.commercial_use,
            }
        })
        .collect();

    Report {
        project: project.to_string(),
        scanned_at: scanned_at.to_string(),
        path: path.to_string(),
        summary,
        packages: classified,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use model::{CommercialUse, Ecosystem, RiskLevel};

    fn raw(name: &str, license: Option<&str>) -> Package {
        Package {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            license: license.map(str::to_string),
            license_source: if license.is_some() {
                model::LicenseSource::Declared
            } else {
                model::LicenseSource::None
            },
            ecosystem: Ecosystem::Npm,
            dependency_type: model::DependencyType::Direct,
        }
    }

    #[test]
    fn builds_summary_counts() {
        let report = build_report(
            "proj",
            "/tmp/proj",
            "2026-06-12T00:00:00Z",
            vec![
                raw("a", Some("MIT")),
                raw("b", Some("LGPL-3.0")),
                raw("c", Some("AGPL-3.0")),
                raw("d", None),
            ],
            &[],
        );
        assert_eq!(report.summary.total, 4);
        assert_eq!(report.summary.permissive, 1);
        assert_eq!(report.summary.weak_copyleft, 1);
        assert_eq!(report.summary.strong_copyleft, 1);
        assert_eq!(report.summary.unknown, 1);
    }

    #[test]
    fn ignore_list_filters_packages_case_insensitively() {
        let report = build_report(
            "proj",
            "/tmp/proj",
            "2026-06-12T00:00:00Z",
            vec![raw("Alpha", Some("GPL-3.0")), raw("beta", Some("MIT"))],
            &["alpha".to_string()],
        );
        assert_eq!(report.summary.total, 1);
        assert_eq!(report.packages[0].name, "beta");
        assert_eq!(report.summary.strong_copyleft, 0);
    }

    #[test]
    fn missing_license_becomes_unknown_placeholder() {
        let report = build_report(
            "proj",
            "/tmp/proj",
            "2026-06-12T00:00:00Z",
            vec![raw("a", None)],
            &[],
        );
        let pkg = &report.packages[0];
        assert_eq!(pkg.license, UNKNOWN_LICENSE);
        assert_eq!(pkg.category, LicenseCategory::Unknown);
        assert_eq!(pkg.risk_level, RiskLevel::Medium);
        assert_eq!(pkg.commercial_use, CommercialUse::Review);
    }

    #[test]
    fn risk_fields_propagate() {
        let report = build_report(
            "proj",
            "/tmp/proj",
            "2026-06-12T00:00:00Z",
            vec![raw("agpl-pkg", Some("AGPL-3.0"))],
            &[],
        );
        assert_eq!(report.packages[0].risk_score, 100);
        assert_eq!(report.packages[0].risk_level, RiskLevel::High);
        assert_eq!(report.packages[0].commercial_use, CommercialUse::Restricted);
    }
}
