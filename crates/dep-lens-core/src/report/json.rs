use crate::model::Report;

/// Serialize a report as compact JSON. This is the wire format the Node.js
/// wrapper parses from the binary's stdout.
pub fn render(report: &Report) -> serde_json::Result<String> {
    serde_json::to_string(report)
}

/// Serialize a report as pretty-printed JSON for human consumption.
pub fn render_pretty(report: &Report) -> serde_json::Result<String> {
    serde_json::to_string_pretty(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;

    fn sample_report() -> Report {
        Report {
            project: "sample".to_string(),
            scanned_at: "2026-06-12T00:00:00Z".to_string(),
            path: "/tmp/sample".to_string(),
            summary: Summary {
                total: 1,
                permissive: 1,
                weak_copyleft: 0,
                strong_copyleft: 0,
                unknown: 0,
            },
            packages: vec![ClassifiedPackage {
                name: "alpha".to_string(),
                version: "1.0.0".to_string(),
                license: "MIT".to_string(),
                license_source: LicenseSource::Declared,
                ecosystem: Ecosystem::Npm,
                category: LicenseCategory::Permissive,
                risk_score: 0,
                risk_level: RiskLevel::Low,
                commercial_use: CommercialUse::Yes,
            }],
        }
    }

    #[test]
    fn roundtrips_through_json() {
        let report = sample_report();
        let json = render(&report).unwrap();
        let parsed: Report = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, report);
    }

    #[test]
    fn uses_camel_case_keys() {
        let json = render(&sample_report()).unwrap();
        assert!(json.contains("\"scannedAt\""));
        assert!(json.contains("\"riskScore\""));
        assert!(json.contains("\"riskLevel\""));
        assert!(json.contains("\"commercialUse\""));
        assert!(json.contains("\"weakCopyleft\""));
        assert!(json.contains("\"strongCopyleft\""));
        assert!(json.contains("\"licenseSource\":\"declared\""));
    }

    #[test]
    fn enums_serialize_as_expected_strings() {
        let json = render(&sample_report()).unwrap();
        assert!(json.contains("\"category\":\"Permissive\""));
        assert!(json.contains("\"riskLevel\":\"low\""));
        assert!(json.contains("\"commercialUse\":\"yes\""));
        assert!(json.contains("\"ecosystem\":\"npm\""));
    }
}
