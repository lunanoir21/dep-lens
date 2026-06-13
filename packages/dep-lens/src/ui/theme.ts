import type { CommercialUse, LicenseCategory, RiskLevel } from '../types.js';

/**
 * Color policy: strong copyleft and other risky entries are red, items
 * needing caution are amber, safe entries are green, unidentifiable
 * entries are slate. Truecolor hex values (rendered via chalk) for a
 * richer palette than the default ANSI 16; no emoji anywhere in the UI.
 */

export const PALETTE = {
  good: '#4ade80', // emerald
  ok: '#fbbf24', // amber
  bad: '#fb7185', // rose
  unknown: '#94a3b8', // slate
  brand: '#38bdf8', // sky
  accent: '#a78bfa', // violet
  dim: '#64748b', // slate (dim text)
} as const;

/** Health score thresholds (0-100) shared by the header face and bar. */
export const HEALTH_GOOD_THRESHOLD = 80;
export const HEALTH_OK_THRESHOLD = 50;

export function healthColor(score: number): string {
  if (score >= HEALTH_GOOD_THRESHOLD) return PALETTE.good;
  if (score >= HEALTH_OK_THRESHOLD) return PALETTE.ok;
  return PALETTE.bad;
}

export function categoryColor(category: LicenseCategory): string {
  switch (category) {
    case 'Permissive':
      return PALETTE.good;
    case 'WeakCopyleft':
      return PALETTE.ok;
    case 'StrongCopyleft':
      return PALETTE.bad;
    case 'Unknown':
      return PALETTE.unknown;
  }
}

export function riskColor(level: RiskLevel): string {
  switch (level) {
    case 'low':
      return PALETTE.good;
    case 'medium':
      return PALETTE.ok;
    case 'high':
      return PALETTE.bad;
  }
}

export function commercialColor(use: CommercialUse): string {
  switch (use) {
    case 'yes':
      return PALETTE.good;
    case 'caution':
      return PALETTE.ok;
    case 'restricted':
      return PALETTE.bad;
    case 'review':
      return PALETTE.unknown;
  }
}
