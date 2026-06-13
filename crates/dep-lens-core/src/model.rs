use serde::{Deserialize, Serialize};

/// Package ecosystem a dependency was discovered in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Ecosystem {
    Npm,
    Cargo,
    Go,
    Python,
    Ruby,
    Php,
    Java,
    Dart,
    Cpp,
}

/// Whether a dependency is direct (declared in the project manifest) or
/// transitive (pulled in by another dependency).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DependencyType {
    Direct,
    Transitive,
}

/// License category buckets used for commercial risk classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LicenseCategory {
    Permissive,
    WeakCopyleft,
    StrongCopyleft,
    Unknown,
}

/// Coarse risk level derived from the numeric risk score.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

/// Whether a license is generally safe for commercial use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommercialUse {
    Yes,
    Caution,
    Restricted,
    Review,
}

/// Where a package's license information came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LicenseSource {
    /// Declared in the package manifest (package.json / Cargo.toml).
    Declared,
    /// Identified heuristically from a LICENSE/COPYING file.
    LicenseFile,
    /// No license information found anywhere.
    None,
}

/// A raw dependency before license classification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub license: Option<String>,
    pub license_source: LicenseSource,
    pub ecosystem: Ecosystem,
    pub dependency_type: DependencyType,
}

/// A dependency after classification and risk assessment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassifiedPackage {
    pub name: String,
    pub version: String,
    pub license: String,
    pub license_source: LicenseSource,
    pub ecosystem: Ecosystem,
    pub dependency_type: DependencyType,
    pub category: LicenseCategory,
    pub risk_score: u8,
    pub risk_level: RiskLevel,
    pub commercial_use: CommercialUse,
}

/// Aggregate counts over all classified packages.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    pub total: usize,
    pub permissive: usize,
    pub weak_copyleft: usize,
    pub strong_copyleft: usize,
    pub unknown: usize,
}

/// Full scan report. Serialized as JSON on stdout by the binary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub project: String,
    pub scanned_at: String,
    pub path: String,
    pub summary: Summary,
    pub packages: Vec<ClassifiedPackage>,
}
