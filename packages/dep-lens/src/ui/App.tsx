import { writeFile } from 'node:fs/promises';
import React, { useEffect, useMemo, useState } from 'react';
import { Box, Text, useApp, useInput } from 'ink';

import type { LicenseCategory, Report } from '../types.js';
import { format } from '../i18n.js';
import { filterPackages, SORT_COLUMNS, sortPackages } from '../utils.js';
import { DetailPane } from './DetailPane.js';
import { ExportMenu, EXPORT_OPTION_COUNT } from './ExportMenu.js';
import { FilterBar } from './FilterBar.js';
import { Header } from './Header.js';
import { HelpOverlay } from './HelpOverlay.js';
import { useMessages } from './i18n-context.js';
import { PackageTable } from './PackageTable.js';
import { SummaryBar } from './SummaryBar.js';

export interface AppProps {
  report: Report;
  getHtml: () => Promise<string>;
}

type Mode = 'list' | 'filter' | 'export' | 'detail' | 'help';

const QUICK_FILTERS: Record<string, LicenseCategory> = {
  '1': 'Permissive',
  '2': 'WeakCopyleft',
  '3': 'StrongCopyleft',
  '4': 'Unknown',
};

const STATUS_CLEAR_MS = 4000;

export function App({ report, getHtml }: AppProps): React.JSX.Element {
  const messages = useMessages();
  const { exit } = useApp();
  const [mode, setMode] = useState<Mode>('list');
  const [query, setQuery] = useState('');
  const [categoryFilter, setCategoryFilter] = useState<LicenseCategory | null>(null);
  const [sortIndex, setSortIndex] = useState(0);
  const [descending, setDescending] = useState(false);
  const [cursor, setCursor] = useState(0);
  const [exportCursor, setExportCursor] = useState(0);
  const [status, setStatus] = useState('');

  useEffect(() => {
    if (status.length === 0) return;
    const id = setTimeout(() => setStatus(''), STATUS_CLEAR_MS);
    return () => clearTimeout(id);
  }, [status]);

  const sortColumn = SORT_COLUMNS[sortIndex] ?? 'name';

  const visible = useMemo(
    () =>
      sortPackages(
        filterPackages(report.packages, query, categoryFilter),
        sortColumn,
        descending,
      ),
    [report.packages, query, categoryFilter, sortColumn, descending],
  );

  const maxCursor = Math.max(0, visible.length - 1);
  const clampedCursor = Math.min(cursor, maxCursor);
  const selectedPackage = visible[clampedCursor];

  function exportSelection(index: number): void {
    const filename = index === 0 ? 'dep-lens-report.json' : 'dep-lens-report.html';
    setStatus(format(messages.exportMenu.writing, { file: filename }));
    
    const promise = index === 0 
      ? Promise.resolve(JSON.stringify(report, null, 2))
      : getHtml();

    promise
      .then(async (content) => {
        await writeFile(filename, content);
        setStatus(format(messages.exportMenu.wrote, { file: filename }));
      })
      .catch((error: unknown) => {
        setStatus(format(messages.exportMenu.failed, { error: String(error) }));
      });
  }

  useInput((input, key) => {
    if (mode === 'filter') {
      if (key.return) setMode('list');
      else if (key.escape) { setQuery(''); setMode('list'); }
      else if (key.backspace || key.delete) { setQuery(c => c.slice(0, -1)); setCursor(0); }
      else if (input.length > 0 && !key.ctrl && !key.meta) { setQuery(c => c + input); setCursor(0); }
      return;
    }

    if (mode === 'export') {
      if (key.escape) setMode('list');
      else if (key.upArrow) setExportCursor(c => Math.max(0, c - 1));
      else if (key.downArrow) setExportCursor(c => Math.min(EXPORT_OPTION_COUNT - 1, c + 1));
      else if (key.return) { exportSelection(exportCursor); setMode('list'); }
      return;
    }

    if (mode === 'detail' || mode === 'help') {
      if (key.escape || key.return || input === 'q' || (mode === 'help' && input === 'h')) {
        setMode('list');
      }
      return;
    }

    const quickFilter = QUICK_FILTERS[input];
    if (quickFilter !== undefined) { setCategoryFilter(c => c === quickFilter ? null : quickFilter); setCursor(0); }
    else if (input === '0') { setQuery(''); setCategoryFilter(null); setCursor(0); }
    else if (input === 'q') exit();
    else if (input === 'f' || input === '/') { setStatus(''); setMode('filter'); }
    else if (input === 's') setSortIndex(c => (c + 1) % SORT_COLUMNS.length);
    else if (input === 'r') setDescending(c => !c);
    else if (input === 'e') { setStatus(''); setExportCursor(0); setMode('export'); }
    else if (input === 'h' || input === '?') setMode('help');
    else if (input === 'g') setCursor(0);
    else if (input === 'G') setCursor(maxCursor);
    else if (key.return && selectedPackage !== undefined) setMode('detail');
    else if (key.upArrow) setCursor(c => Math.max(0, c - 1));
    else if (key.downArrow) setCursor(c => Math.min(maxCursor, c + 1));
    else if (key.pageUp) setCursor(c => Math.max(0, c - 10));
    else if (key.pageDown) setCursor(c => Math.min(maxCursor, c + 10));
  });

  const activeFilters: string[] = [];
  if (query.length > 0) activeFilters.push(format(messages.filterText, { query }));
  if (categoryFilter !== null) activeFilters.push(format(messages.filterCategory, { category: messages.categories[categoryFilter] }));

  const modeLabel = mode !== 'list' ? ` [${mode.toUpperCase()}] ` : '';

  return (
    <Box flexDirection="column" paddingX={2} paddingY={1}>
      <Header project={report.project} scannedAt={report.scannedAt} summary={report.summary} />
      
      <Box marginTop={1} marginBottom={1}>
        <SummaryBar summary={report.summary} />
      </Box>

      <PackageTable
        packages={visible}
        cursor={clampedCursor}
        sortColumn={sortColumn}
        descending={descending}
        query={query}
      />

      <Box marginTop={1} flexDirection="column">
        {mode === 'filter' && <FilterBar query={query} />}
        {mode === 'export' && <ExportMenu cursor={exportCursor} />}
        {mode === 'detail' && selectedPackage !== undefined && <DetailPane pkg={selectedPackage} />}
        {mode === 'help' && <HelpOverlay />}
        
        <Box paddingX={1} marginTop={1} borderStyle="round" borderColor="gray">
          <Text color="gray">
            <Text bold color="cyan">{modeLabel}</Text>
            {activeFilters.length > 0 && mode !== 'filter' && <Text color="yellow">({activeFilters.join(', ')}) </Text>}
            {status.length > 0 ? <Text color="green">{status} </Text> : messages.footer}
          </Text>
        </Box>
      </Box>
    </Box>
  );
}
