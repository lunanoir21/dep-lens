import type { CommercialUse, LicenseCategory, RiskLevel } from '../types.js';

/**
 * Color policy: strong copyleft and other risky entries are red, items
 * needing caution are yellow, safe entries are green, unidentifiable
 * entries are gray. ANSI color names only; no emoji anywhere in the UI.
 */

export function categoryColor(category: LicenseCategory): string {
  switch (category) {
    case 'Permissive':
      return 'green';
    case 'WeakCopyleft':
      return 'yellow';
    case 'StrongCopyleft':
      return 'red';
    case 'Unknown':
      return 'gray';
  }
}

export function riskColor(level: RiskLevel): string {
  switch (level) {
    case 'low':
      return 'green';
    case 'medium':
      return 'yellow';
    case 'high':
      return 'red';
  }
}

export function commercialColor(use: CommercialUse): string {
  switch (use) {
    case 'yes':
      return 'green';
    case 'caution':
      return 'yellow';
    case 'restricted':
      return 'red';
    case 'review':
      return 'gray';
  }
}

