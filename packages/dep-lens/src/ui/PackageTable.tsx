import React from 'react';
import { Box, Text, useStdout } from 'ink';

import type { ClassifiedPackage } from '../types.js';
import { format, type Messages } from '../i18n.js';
import { pad, type SortColumn } from '../utils.js';
import { useRevealOnce } from './hooks.js';
import { useMessages } from './i18n-context.js';
import { categoryColor } from './theme.js';

export interface PackageTableProps {
  packages: readonly ClassifiedPackage[];
  cursor: number;
  sortColumn: SortColumn;
  descending: boolean;
  viewport?: number;
}

interface Column {
  key: SortColumn;
  label: string;
  width: number;
}

/**
 * Column widths adapt to the terminal: version/category/risk/commercial are
 * fixed, the remainder is split between name (60%) and license (40%).
 */
export function computeColumns(terminalWidth: number, messages: Messages): Column[] {
  const fixed: Column[] = [
    { key: 'version', label: messages.table.version, width: 9 },
    { key: 'category', label: messages.table.category, width: 16 },
    { key: 'riskScore', label: messages.table.risk, width: 7 },
    { key: 'commercialUse', label: messages.table.commercial, width: 12 },
  ];
  const fixedTotal = fixed.reduce((sum, column) => sum + column.width, 0);
  const separators = 5;
  const chrome = 4; // table border plus horizontal padding
  const flexible = Math.max(terminalWidth - chrome - separators - fixedTotal, 26);
  const nameWidth = Math.max(Math.floor(flexible * 0.6), 16);
  const licenseWidth = Math.max(flexible - nameWidth, 10);
  const columns: Column[] = [
    { key: 'name', label: messages.table.package, width: nameWidth },
    { key: 'license', label: messages.table.license, width: licenseWidth },
    ...fixed,
  ];
  return columns.sort((a, b) => ORDER.indexOf(a.key) - ORDER.indexOf(b.key));
}

const ORDER: SortColumn[] = [
  'name',
  'version',
  'license',
  'category',
  'riskScore',
  'commercialUse',
];

function cellValue(
  pkg: ClassifiedPackage,
  key: SortColumn,
  messages: Messages,
): string {
  switch (key) {
    case 'name':
      return pkg.name;
    case 'version':
      return pkg.version;
    case 'license':
      return pkg.license;
    case 'category':
      return messages.categories[pkg.category];
    case 'riskScore':
      return String(pkg.riskScore);
    case 'commercialUse':
      return messages.commercial[pkg.commercialUse];
  }
}

export function PackageTable({
  packages,
  cursor,
  sortColumn,
  descending,
  viewport = 12,
}: PackageTableProps): React.JSX.Element {
  const messages = useMessages();
  const { stdout } = useStdout();
  // ink assumes an 80-column terminal when the size is unknown; match it.
  const columns = computeColumns(stdout.columns > 0 ? stdout.columns : 80, messages);
  // Entrance animation: rows appear one by one on the first render only.
  const revealed = useRevealOnce(viewport);

  const start = Math.max(
    0,
    Math.min(cursor - Math.floor(viewport / 2), packages.length - viewport),
  );
  const rows = packages.slice(start, start + Math.min(viewport, revealed));

  const header = columns
    .map((column) => {
      const marker = column.key === sortColumn ? (descending ? ' v' : ' ^') : '';
      return pad(`${column.label}${marker}`, column.width);
    })
    .join(' ');

  return (
    <Box flexDirection="column" borderStyle="classic" paddingX={1}>
      <Text bold underline>
        {header}
      </Text>
      {rows.length === 0 ? (
        <Text color="gray">{messages.table.empty}</Text>
      ) : (
        rows.map((pkg, index) => {
          const absoluteIndex = start + index;
          const selected = absoluteIndex === cursor;
          const line = columns
            .map((column) => pad(cellValue(pkg, column.key, messages), column.width))
            .join(' ');
          return (
            <Text
              key={`${pkg.ecosystem}:${pkg.name}@${pkg.version}`}
              color={categoryColor(pkg.category)}
              inverse={selected}
            >
              {line}
            </Text>
          );
        })
      )}
      <Text color="gray">
        {packages.length === 0
          ? format(messages.table.rows, { start: 0, end: 0, total: 0 })
          : format(messages.table.rows, {
              start: start + 1,
              end: start + rows.length,
              total: packages.length,
            })}
      </Text>
    </Box>
  );
}
