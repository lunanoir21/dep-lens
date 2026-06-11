use crate::model::{CommercialUse, LicenseCategory, RiskLevel};

/// Result of assessing one license for commercial risk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RiskAssessment {
    pub score: u8,
    pub level: RiskLevel,
    pub commercial_use: CommercialUse,
}

/// Map a numeric score (0-100) to a coarse level.
pub fn level_for_score(score: u8) -> RiskLevel {
    match score {
        0..=29 => RiskLevel::Low,
        30..=69 => RiskLevel::Medium,
        _ => RiskLevel::High,
    }
}

/// Assess commercial risk for a classified license.
///
/// Scores by family:
/// - Permissive: 0
/// - Weak copyleft: MPL 40, EPL 45, LGPL 50, CDDL 55 (roughly ordered by how
///   far the copyleft obligation reaches beyond the library itself)
/// - Strong copyleft: 90; AGPL and SSPL score 100 because their network
///   clauses also cover SaaS usage without distribution
/// - Unknown: 60, since an unidentified license cannot be assumed safe
pub fn assess(category: LicenseCategory, license: &str) -> RiskAssessment {
    let upper = license.to_uppercase();
    let score: u8 = match category {
        LicenseCategory::Permissive => 0,
        LicenseCategory::WeakCopyleft => {
            if upper.contains("CDDL") {
                55
            } else if upper.contains("LGPL") {
                50
            } else if upper.contains("EPL") {
                45
            } else {
                40
            }
        }
        LicenseCategory::StrongCopyleft => {
            if upper.contains("AGPL") || upper.contains("SSPL") {
                100
            } else {
                90
            }
        }
        LicenseCategory::Unknown => 60,
    };
    let commercial_use = match category {
        LicenseCategory::Permissive => CommercialUse::Yes,
        LicenseCategory::WeakCopyleft => CommercialUse::Caution,
        LicenseCategory::StrongCopyleft => CommercialUse::Restricted,
        LicenseCategory::Unknown => CommercialUse::Review,
    };
    RiskAssessment {
        score,
        level: level_for_score(score),
        commercial_use,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permissive_is_low_risk_and_commercially_usable() {
        let a = assess(LicenseCategory::Permissive, "MIT");
        assert_eq!(a.score, 0);
        assert_eq!(a.level, RiskLevel::Low);
        assert_eq!(a.commercial_use, CommercialUse::Yes);
    }

    #[test]
    fn weak_copyleft_is_medium_risk() {
        let a = assess(LicenseCategory::WeakCopyleft, "MPL-2.0");
        assert_eq!(a.score, 40);
        assert_eq!(a.level, RiskLevel::Medium);
        assert_eq!(a.commercial_use, CommercialUse::Caution);
    }

    #[test]
    fn weak_copyleft_scores_scale_with_reach() {
        assert_eq!(assess(LicenseCategory::WeakCopyleft, "MPL-2.0").score, 40);
        assert_eq!(assess(LicenseCategory::WeakCopyleft, "EPL-2.0").score, 45);
        assert_eq!(assess(LicenseCategory::WeakCopyleft, "LGPL-2.1").score, 50);
        assert_eq!(assess(LicenseCategory::WeakCopyleft, "CDDL-1.0").score, 55);
    }

    #[test]
    fn strong_copyleft_is_high_risk() {
        let a = assess(LicenseCategory::StrongCopyleft, "GPL-3.0");
        assert_eq!(a.score, 90);
        assert_eq!(a.level, RiskLevel::High);
        assert_eq!(a.commercial_use, CommercialUse::Restricted);
    }

    #[test]
    fn network_copyleft_scores_maximum() {
        assert_eq!(
            assess(LicenseCategory::StrongCopyleft, "AGPL-3.0").score,
            100
        );
        assert_eq!(
            assess(LicenseCategory::StrongCopyleft, "SSPL-1.0").score,
            100
        );
    }

    #[test]
    fn unknown_requires_review() {
        let a = assess(LicenseCategory::Unknown, "UNKNOWN");
        assert_eq!(a.score, 60);
        assert_eq!(a.level, RiskLevel::Medium);
        assert_eq!(a.commercial_use, CommercialUse::Review);
    }

    #[test]
    fn level_boundaries() {
        assert_eq!(level_for_score(0), RiskLevel::Low);
        assert_eq!(level_for_score(29), RiskLevel::Low);
        assert_eq!(level_for_score(30), RiskLevel::Medium);
        assert_eq!(level_for_score(69), RiskLevel::Medium);
        assert_eq!(level_for_score(70), RiskLevel::High);
        assert_eq!(level_for_score(100), RiskLevel::High);
    }
}
