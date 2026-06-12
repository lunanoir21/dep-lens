import type { ClassifiedPackage, Report } from '../types.js';

export function pkg(overrides: Partial<ClassifiedPackage> = {}): ClassifiedPackage {
  return {
    name: 'example',
    version: '1.0.0',
    license: 'MIT',
    licenseSource: 'declared',
    ecosystem: 'npm',
    dependencyType: 'direct',
    category: 'Permissive',
    riskScore: 0,
    riskLevel: 'low',
    commercialUse: 'yes',
    ...overrides,
  };
}

export function report(packages: ClassifiedPackage[]): Report {
  const summary = {
    total: packages.length,
    permissive: packages.filter((p) => p.category === 'Permissive').length,
    weakCopyleft: packages.filter((p) => p.category === 'WeakCopyleft').length,
    strongCopyleft: packages.filter((p) => p.category === 'StrongCopyleft').length,
    unknown: packages.filter((p) => p.category === 'Unknown').length,
  };
  return {
    project: 'fixture',
    scannedAt: '2026-06-12T00:00:00Z',
    path: '/tmp/fixture',
    summary,
    packages,
  };
}
