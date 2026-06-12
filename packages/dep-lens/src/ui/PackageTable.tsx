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
  query?: string;
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
    { key: 'dependencyType', label: messages.table.type, width: 10 },
    { key: 'riskScore', label: messages.table.risk, width: 7 },
    { key: 'commercialUse', label: messages.table.commercial, width: 12 },
  ];
  const fixedTotal = fixed.reduce((sum, column) => sum + column.width, 0);
  const separators = 6; // 7 columns
  const chrome = 4; // table border plus horizontal padding
  const flexible = Math.max(terminalWidth - chrome - separators - fixedTotal, 16);
  const nameWidth = Math.max(Math.floor(flexible * 0.6), 10);
  const licenseWidth = Math.max(flexible - nameWidth, 6);
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
  'dependencyType',
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
    case 'dependencyType':
      return messages.types[pkg.dependencyType];
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
  query = '',
}: PackageTableProps): React.JSX.Element {
  const messages = useMessages();
  const { stdout } = useStdout();
  const columns = computeColumns(stdout.columns > 0 ? stdout.columns : 80, messages);
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

  const scrollbarHeight = Math.min(viewport, packages.length);
  const scrollbarThumbHeight = Math.max(
    1,
    Math.floor((scrollbarHeight / Math.max(1, packages.length)) * scrollbarHeight),
  );
  const scrollbarThumbTop =
    packages.length > viewport
      ? Math.floor((start / Math.max(1, packages.length - viewport)) * (scrollbarHeight - scrollbarThumbHeight))
      : 0;

  return (
    <Box flexDirection="column" borderStyle="round" borderColor="gray" paddingX={1}>
      <Text bold color="white" underline>
        {header}
      </Text>
      <Box flexDirection="row">
        <Box flexDirection="column" flexGrow={1}>
          {rows.length === 0 ? (
            <Text color="gray">{messages.table.empty}</Text>
          ) : (
            rows.map((pkg, index) => {
              const absoluteIndex = start + index;
              const selected = absoluteIndex === cursor;
              const line = columns
                .map((column) => {
                  const val = cellValue(pkg, column.key, messages);
                  return pad(val, column.width);
                })
                .join(' ');

              if (selected) {
                return (
                  <Text
                    key={`${pkg.ecosystem}:${pkg.name}@${pkg.version}`}
                    color={categoryColor(pkg.category)}
                    inverse
                  >
                    {line}
                  </Text>
                );
              }

              // Search highlighting for unselected rows
              if (query.length > 0) {
                const needle = query.toLowerCase();
                const parts: React.ReactNode[] = [];
                let lastIndex = 0;
                const lowerLine = line.toLowerCase();
                let matchIndex = lowerLine.indexOf(needle);
                while (matchIndex !== -1) {
                  parts.push(line.slice(lastIndex, matchIndex));
                  parts.push(
                    <Text key={matchIndex} underline bold color="cyan">
                      {line.slice(matchIndex, matchIndex + needle.length)}
                    </Text>,
                  );
                  lastIndex = matchIndex + needle.length;
                  matchIndex = lowerLine.indexOf(needle, lastIndex);
                }
                parts.push(line.slice(lastIndex));
                return (
                  <Text
                    key={`${pkg.ecosystem}:${pkg.name}@${pkg.version}`}
                    color={categoryColor(pkg.category)}
                  >
                    {parts}
                  </Text>
                );
              }

              return (
                <Text
                  key={`${pkg.ecosystem}:${pkg.name}@${pkg.version}`}
                  color={categoryColor(pkg.category)}
                >
                  {line}
                </Text>
              );
            })
          )}
        </Box>
        {packages.length > viewport && (
          <Box flexDirection="column" width={1} marginLeft={1}>
            {Array.from({ length: scrollbarHeight }).map((_, i) => (
              <Text key={i} color="gray">
                {i >= scrollbarThumbTop && i < scrollbarThumbTop + scrollbarThumbHeight
                  ? '┃'
                  : '│'}
              </Text>
            ))}
          </Box>
        )}
      </Box>
      <Box justifyContent="space-between">
        <Text color="gray">
          {packages.length === 0
            ? format(messages.table.rows, { start: 0, end: 0, total: 0 })
            : format(messages.table.rows, {
                start: start + 1,
                end: start + rows.length,
                total: packages.length,
              })}
        </Text>
        <Text color="gray">{messages.table.rows.includes(' / ') ? `sıralama: ${sortColumn}` : `sort: ${sortColumn}`}</Text>
      </Box>
    </Box>
  );
}
