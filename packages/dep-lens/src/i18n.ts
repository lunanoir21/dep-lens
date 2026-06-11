import type {
  CommercialUse,
  LicenseCategory,
  LicenseSource,
  RiskLevel,
} from './types.js';

export type Locale = 'en' | 'tr';

/**
 * Complete message catalog for one UI language. Both catalogs must satisfy
 * this interface, so a missing translation is a compile error.
 */
export interface Messages {
  header: {
    scanned: string;
    packages: string;
  };
  categories: Record<LicenseCategory, string>;
  summaryShort: Record<LicenseCategory, string>;
  riskLevels: Record<RiskLevel, string>;
  commercial: Record<CommercialUse, string>;
  sources: Record<LicenseSource, string>;
  advice: Record<LicenseCategory, string>;
  table: {
    package: string;
    version: string;
    license: string;
    category: string;
    risk: string;
    commercial: string;
    empty: string;
    rows: string; // placeholders: {start} {end} {total}
  };
  footer: string;
  filtersLabel: string; // {list}
  filterText: string; // {query}
  filterCategory: string; // {category}
  filterBar: {
    label: string;
    hint: string;
  };
  exportMenu: {
    title: string;
    json: string;
    html: string;
    cancel: string;
    writing: string; // {file}
    wrote: string; // {file}
    failed: string; // {error}
  };
  detail: {
    hint: string;
    ecosystem: string;
    license: string;
    category: string;
    risk: string;
    commercial: string;
    advice: string;
    riskValue: string; // {score} {level}
  };
  help: {
    title: string;
    bindings: ReadonlyArray<readonly [string, string]>;
  };
  scanning: {
    title: string;
    elapsed: string; // {seconds}
  };
  error: {
    title: string;
    hint: string;
  };
}

/** Replace `{name}` placeholders in a catalog template. */
export function format(
  template: string,
  values: Record<string, string | number>,
): string {
  return template.replace(/\{(\w+)\}/g, (match, key: string) =>
    key in values ? String(values[key]) : match,
  );
}

export const EN: Messages = {
  header: {
    scanned: 'scanned',
    packages: 'packages',
  },
  categories: {
    Permissive: 'Permissive',
    WeakCopyleft: 'Weak Copyleft',
    StrongCopyleft: 'Strong Copyleft',
    Unknown: 'Unknown',
  },
  summaryShort: {
    Permissive: 'Permissive',
    WeakCopyleft: 'Weak',
    StrongCopyleft: 'Strong',
    Unknown: 'Unknown',
  },
  riskLevels: {
    low: 'low',
    medium: 'medium',
    high: 'high',
  },
  commercial: {
    yes: 'yes',
    caution: 'caution',
    restricted: 'restricted',
    review: 'review',
  },
  sources: {
    declared: 'declared in manifest',
    licenseFile: 'detected from LICENSE file',
    none: 'not found',
  },
  advice: {
    Permissive: 'Safe for commercial use. Keep the copyright notice.',
    WeakCopyleft:
      'Commercial use is fine; modifications to this library itself must stay open.',
    StrongCopyleft:
      'Copyleft terms can extend to derivative works. Legal review required.',
    Unknown: 'License could not be identified. Review this package manually.',
  },
  table: {
    package: 'PACKAGE',
    version: 'VERSION',
    license: 'LICENSE',
    category: 'CATEGORY',
    risk: 'RISK',
    commercial: 'COMMERCIAL',
    empty: 'No packages match the current filter.',
    rows: 'rows {start}-{end} of {total}',
  },
  footer: 'arrows  enter detail  f filter  1-4 cat  s sort  e export  h help  q quit',
  filtersLabel: 'filters: {list}',
  filterText: 'text "{query}"',
  filterCategory: 'category {category}',
  filterBar: {
    label: 'filter: ',
    hint: '  (enter apply, esc clear)',
  },
  exportMenu: {
    title: 'export (enter select, esc cancel)',
    json: 'Export JSON  (dep-lens-report.json)',
    html: 'Export HTML  (dep-lens-report.html)',
    cancel: 'Cancel',
    writing: 'writing {file} ...',
    wrote: 'wrote {file}',
    failed: 'export failed: {error}',
  },
  detail: {
    hint: '(enter/esc close)',
    ecosystem: 'ecosystem',
    license: 'license',
    category: 'category',
    risk: 'risk',
    commercial: 'commercial',
    advice: 'advice',
    riskValue: '{score}/100 ({level})',
  },
  help: {
    title: 'keys (h or esc to close)',
    bindings: [
      ['up/down', 'move selection'],
      ['pgup/pgdn', 'jump 10 rows'],
      ['g / G', 'jump to top / bottom'],
      ['enter', 'open package detail'],
      ['f', 'free-text filter (name, license, category)'],
      ['1 / 2 / 3 / 4', 'quick filter: Permissive / Weak / Strong / Unknown'],
      ['0', 'clear all filters'],
      ['s', 'cycle sort column'],
      ['r', 'reverse sort direction'],
      ['e', 'export menu (JSON / HTML)'],
      ['h', 'toggle this help'],
      ['q', 'quit'],
    ],
  },
  scanning: {
    title: 'scanning dependencies',
    elapsed: '{seconds}s elapsed',
  },
  error: {
    title: 'scan failed',
    hint: 'press q to quit',
  },
};

export const TR: Messages = {
  header: {
    scanned: 'tarandı',
    packages: 'paket',
  },
  categories: {
    Permissive: 'İzinli',
    WeakCopyleft: 'Zayıf Copyleft',
    StrongCopyleft: 'Güçlü Copyleft',
    Unknown: 'Bilinmeyen',
  },
  summaryShort: {
    Permissive: 'İzinli',
    WeakCopyleft: 'Zayıf',
    StrongCopyleft: 'Güçlü',
    Unknown: 'Bilinmeyen',
  },
  riskLevels: {
    low: 'düşük',
    medium: 'orta',
    high: 'yüksek',
  },
  commercial: {
    yes: 'evet',
    caution: 'dikkat',
    restricted: 'kısıtlı',
    review: 'incele',
  },
  sources: {
    declared: 'manifestte bildirilmiş',
    licenseFile: 'LICENSE dosyasından tespit edildi',
    none: 'bulunamadı',
  },
  advice: {
    Permissive: 'Ticari kullanım için güvenli. Telif bildirimini koruyun.',
    WeakCopyleft:
      'Ticari kullanım uygundur; bu kütüphanede yapılan değişiklikler açık kalmalıdır.',
    StrongCopyleft:
      'Copyleft şartları türev çalışmalara yayılabilir. Hukuki inceleme gerekir.',
    Unknown: 'Lisans tespit edilemedi. Bu paketi elle inceleyin.',
  },
  table: {
    package: 'PAKET',
    version: 'SÜRÜM',
    license: 'LİSANS',
    category: 'KATEGORİ',
    risk: 'RİSK',
    commercial: 'TİCARİ',
    empty: 'Filtreyle eşleşen paket yok.',
    rows: 'satır {start}-{end} / {total}',
  },
  footer: 'oklar  enter detay  f filtre  1-4 kategori  s sırala  e aktar  h yardım  q çıkış',
  filtersLabel: 'filtreler: {list}',
  filterText: 'metin "{query}"',
  filterCategory: 'kategori {category}',
  filterBar: {
    label: 'filtre: ',
    hint: '  (enter uygula, esc temizle)',
  },
  exportMenu: {
    title: 'dışa aktar (enter seç, esc iptal)',
    json: 'JSON aktar  (dep-lens-report.json)',
    html: 'HTML aktar  (dep-lens-report.html)',
    cancel: 'İptal',
    writing: '{file} yazılıyor ...',
    wrote: '{file} yazıldı',
    failed: 'dışa aktarma başarısız: {error}',
  },
  detail: {
    hint: '(enter/esc kapat)',
    ecosystem: 'ekosistem',
    license: 'lisans',
    category: 'kategori',
    risk: 'risk',
    commercial: 'ticari',
    advice: 'tavsiye',
    riskValue: '{score}/100 ({level})',
  },
  help: {
    title: 'tuşlar (kapatmak için h veya esc)',
    bindings: [
      ['yukarı/aşağı', 'seçimi taşı'],
      ['pgup/pgdn', '10 satır atla'],
      ['g / G', 'başa / sona git'],
      ['enter', 'paket detayını aç'],
      ['f', 'serbest metin filtresi (ad, lisans, kategori)'],
      ['1 / 2 / 3 / 4', 'hızlı filtre: İzinli / Zayıf / Güçlü / Bilinmeyen'],
      ['0', 'tüm filtreleri temizle'],
      ['s', 'sıralama sütununu değiştir'],
      ['r', 'sıralama yönünü ters çevir'],
      ['e', 'dışa aktarma menüsü (JSON / HTML)'],
      ['h', 'bu yardımı aç/kapat'],
      ['q', 'çıkış'],
    ],
  },
  scanning: {
    title: 'bağımlılıklar taranıyor',
    elapsed: '{seconds}sn geçti',
  },
  error: {
    title: 'tarama başarısız',
    hint: 'çıkmak için q',
  },
};

export function getMessages(locale: Locale): Messages {
  return locale === 'tr' ? TR : EN;
}
