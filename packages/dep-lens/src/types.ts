/**
 * TypeScript mirror of the JSON contract emitted by the dep-lens-core binary.
 * Field names and enum string values must stay in sync with
 * crates/dep-lens-core/src/model.rs.
 */

export type Ecosystem = 'npm' | 'cargo';

export type LicenseCategory =
  | 'Permissive'
  | 'WeakCopyleft'
  | 'StrongCopyleft'
  | 'Unknown';

export type RiskLevel = 'low' | 'medium' | 'high';

export type CommercialUse = 'yes' | 'caution' | 'restricted' | 'review';

export type LicenseSource = 'declared' | 'licenseFile' | 'none';

export interface ClassifiedPackage {
  name: string;
  version: string;
  license: string;
  licenseSource: LicenseSource;
  ecosystem: Ecosystem;
  category: LicenseCategory;
  riskScore: number;
  riskLevel: RiskLevel;
  commercialUse: CommercialUse;
}

export interface Summary {
  total: number;
  permissive: number;
  weakCopyleft: number;
  strongCopyleft: number;
  unknown: number;
}

export interface Report {
  project: string;
  scannedAt: string;
  path: string;
  summary: Summary;
  packages: ClassifiedPackage[];
}
