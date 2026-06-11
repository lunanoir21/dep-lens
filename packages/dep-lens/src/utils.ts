import type {
  ClassifiedPackage,
  CommercialUse,
  LicenseCategory,
  Report,
  Summary,
} from './types.js';
import type { FailOn } from './args.js';

export type SortColumn =
  | 'name'
  | 'version'
  | 'license'
  | 'category'
  | 'riskScore'
  | 'commercialUse';

export const SORT_COLUMNS: readonly SortColumn[] = [
  'name',
  'version',
  'license',
  'category',
  'riskScore',
  'commercialUse',
];

const CATEGORY_ORDER: Record<LicenseCategory, number> = {
  Permissive: 0,
  WeakCopyleft: 1,
  StrongCopyleft: 2,
  Unknown: 3,
};

const COMMERCIAL_ORDER: Record<CommercialUse, number> = {
  yes: 0,
  caution: 1,
  restricted: 2,
  review: 3,
};

/**
 * Case-insensitive filter over package name, license, and category, with an
 * optional exact category restriction (used by the TUI quick filters).
 */
export function filterPackages(
  packages: readonly ClassifiedPackage[],
  query: string,
  category: LicenseCategory | null = null,
): ClassifiedPackage[] {
  const needle = query.trim().toLowerCase();
  return packages.filter((pkg) => {
    if (category !== null && pkg.category !== category) {
      return false;
    }
    if (needle.length === 0) {
      return true;
    }
    return (
      pkg.name.toLowerCase().includes(needle) ||
      pkg.license.toLowerCase().includes(needle) ||
      pkg.category.toLowerCase().includes(needle)
    );
  });
}

/** Stable sort by the given column; ties are broken by package name. */
export function sortPackages(
  packages: readonly ClassifiedPackage[],
  column: SortColumn,
  descending: boolean,
): ClassifiedPackage[] {
  const sorted = [...packages].sort((a, b) => {
    let cmp: number;
    switch (column) {
      case 'riskScore':
        cmp = a.riskScore - b.riskScore;
        break;
      case 'category':
        cmp = CATEGORY_ORDER[a.category] - CATEGORY_ORDER[b.category];
        break;
      case 'commercialUse':
        cmp = COMMERCIAL_ORDER[a.commercialUse] - COMMERCIAL_ORDER[b.commercialUse];
        break;
      default:
        cmp = a[column].localeCompare(b[column]);
        break;
    }
    if (descending) {
      cmp = -cmp;
    }
    // Ties always fall back to ascending name order, regardless of direction.
    if (cmp === 0) {
      cmp = a.name.localeCompare(b.name);
    }
    return cmp;
  });
  return sorted;
}

/** Percentage with one decimal, e.g. "42.9". Returns "0.0" for empty sets. */
export function percent(part: number, total: number): string {
  if (total === 0) {
    return '0.0';
  }
  return ((part / total) * 100).toFixed(1);
}

/** Truncate to width, marking cut-off text with a two-dot ASCII ellipsis. */
export function truncate(text: string, width: number): string {
  if (text.length <= width) {
    return text;
  }
  return width <= 2 ? text.slice(0, width) : `${text.slice(0, width - 2)}..`;
}

/** Truncate then right-pad to an exact column width. */
export function pad(text: string, width: number): string {
  return truncate(text, width).padEnd(width);
}

export interface RatioSegment {
  category: LicenseCategory;
  char: string;
  color: string;
  width: number;
}

const SEGMENT_STYLE: ReadonlyArray<{
  category: LicenseCategory;
  char: string;
  color: string;
}> = [
  { category: 'Permissive', char: '#', color: 'green' },
  { category: 'WeakCopyleft', char: '=', color: 'yellow' },
  { category: 'StrongCopyleft', char: '!', color: 'red' },
  { category: 'Unknown', char: '?', color: 'gray' },
];

/**
 * Build a stacked horizontal ratio bar for the summary line. `progress`
 * (0..1) scales the filled portion so the bar can grow as an entrance
 * animation. Widths are distributed by largest remainder so they always sum
 * to the filled width. Distinct fill characters keep the bar readable on
 * monochrome terminals.
 */
export function buildRatioSegments(
  summary: Summary,
  width: number,
  progress = 1,
): RatioSegment[] {
  const counts: Record<LicenseCategory, number> = {
    Permissive: summary.permissive,
    WeakCopyleft: summary.weakCopyleft,
    StrongCopyleft: summary.strongCopyleft,
    Unknown: summary.unknown,
  };
  const filled = Math.round(width * Math.min(Math.max(progress, 0), 1));
  if (summary.total === 0 || filled === 0) {
    return [];
  }
  const exact = SEGMENT_STYLE.map((style) => ({
    style,
    value: (counts[style.category] / summary.total) * filled,
  }));
  const widths = exact.map((entry) => Math.floor(entry.value));
  let remaining = filled - widths.reduce((sum, w) => sum + w, 0);
  const order = exact
    .map((entry, index) => ({ index, frac: entry.value - Math.floor(entry.value) }))
    .sort((a, b) => b.frac - a.frac);
  for (const { index } of order) {
    if (remaining <= 0) {
      break;
    }
    const current = widths[index] ?? 0;
    widths[index] = current + 1;
    remaining -= 1;
  }
  return SEGMENT_STYLE.map((style, index) => ({
    category: style.category,
    char: style.char,
    color: style.color,
    width: widths[index] ?? 0,
  })).filter((segment) => segment.width > 0);
}

/**
 * Packages that violate a --fail-on policy. "gpl" matches every strong
 * copyleft license (GPL-2.0, GPL-3.0, AGPL-3.0); "agpl" matches AGPL only.
 * Dual-licensed packages classified as permissive (e.g. "GPL-2.0 OR MIT")
 * do not violate either policy.
 */
export function violations(report: Report, failOn: FailOn): ClassifiedPackage[] {
  return report.packages.filter((pkg) => {
    if (failOn === 'agpl') {
      return pkg.category === 'StrongCopyleft' && pkg.license.toUpperCase().includes('AGPL');
    }
    return pkg.category === 'StrongCopyleft';
  });
}
