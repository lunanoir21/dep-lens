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

  // Transient status messages fade out on their own.
  useEffect(() => {
    if (status.length === 0) {
      return undefined;
    }
    const id = setTimeout(() => {
      setStatus('');
    }, STATUS_CLEAR_MS);
    return () => {
      clearTimeout(id);
    };
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
    if (index === 0) {
      setStatus(format(messages.exportMenu.writing, { file: 'dep-lens-report.json' }));
      void writeFile('dep-lens-report.json', `${JSON.stringify(report, null, 2)}\n`)
        .then(() => {
          setStatus(format(messages.exportMenu.wrote, { file: 'dep-lens-report.json' }));
        })
        .catch((error: unknown) => {
          setStatus(format(messages.exportMenu.failed, { error: String(error) }));
        });
    } else if (index === 1) {
      setStatus(format(messages.exportMenu.writing, { file: 'dep-lens-report.html' }));
      void getHtml()
        .then(async (html) => {
          await writeFile('dep-lens-report.html', html);
          setStatus(format(messages.exportMenu.wrote, { file: 'dep-lens-report.html' }));
        })
        .catch((error: unknown) => {
          setStatus(format(messages.exportMenu.failed, { error: String(error) }));
        });
    }
  }

  useInput((input, key) => {
    if (mode === 'filter') {
      if (key.return) {
        setMode('list');
      } else if (key.escape) {
        setQuery('');
        setMode('list');
      } else if (key.backspace || key.delete) {
        setQuery((current) => current.slice(0, -1));
        setCursor(0);
      } else if (input.length > 0 && !key.ctrl && !key.meta) {
        setQuery((current) => current + input);
        setCursor(0);
      }
      return;
    }

    if (mode === 'export') {
      if (key.escape) {
        setMode('list');
      } else if (key.upArrow) {
        setExportCursor((current) => Math.max(0, current - 1));
      } else if (key.downArrow) {
        setExportCursor((current) => Math.min(EXPORT_OPTION_COUNT - 1, current + 1));
      } else if (key.return) {
        exportSelection(exportCursor);
        setMode('list');
      }
      return;
    }

    if (mode === 'detail') {
      if (key.escape || key.return || input === 'q') {
        setMode('list');
      }
      return;
    }

    if (mode === 'help') {
      if (key.escape || input === 'h' || input === 'q') {
        setMode('list');
      }
      return;
    }

    // list mode
    const quickFilter = QUICK_FILTERS[input];
    if (quickFilter !== undefined) {
      // Pressing the active category again toggles it off.
      setCategoryFilter((current) => (current === quickFilter ? null : quickFilter));
      setCursor(0);
    } else if (input === '0') {
      setQuery('');
      setCategoryFilter(null);
      setCursor(0);
    } else if (input === 'q') {
      exit();
    } else if (input === 'f') {
      setStatus('');
      setMode('filter');
    } else if (input === 's') {
      setSortIndex((current) => (current + 1) % SORT_COLUMNS.length);
    } else if (input === 'r') {
      setDescending((current) => !current);
    } else if (input === 'e') {
      setStatus('');
      setExportCursor(0);
      setMode('export');
    } else if (input === 'h' || input === '?') {
      setMode('help');
    } else if (input === 'g') {
      setCursor(0);
    } else if (input === 'G') {
      setCursor(maxCursor);
    } else if (key.return) {
      if (selectedPackage !== undefined) {
        setMode('detail');
      }
    } else if (key.upArrow) {
      setCursor((current) => Math.max(0, current - 1));
    } else if (key.downArrow) {
      setCursor((current) => Math.min(maxCursor, current + 1));
    } else if (key.pageUp) {
      setCursor((current) => Math.max(0, current - 10));
    } else if (key.pageDown) {
      setCursor((current) => Math.min(maxCursor, current + 10));
    }
  });

  const activeFilters: string[] = [];
  if (query.length > 0) {
    activeFilters.push(format(messages.filterText, { query }));
  }
  if (categoryFilter !== null) {
    activeFilters.push(
      format(messages.filterCategory, { category: messages.categories[categoryFilter] }),
    );
  }

  return (
    <Box flexDirection="column">
      <Header
        project={report.project}
        scannedAt={report.scannedAt}
        total={report.summary.total}
      />
      <SummaryBar summary={report.summary} />
      <PackageTable
        packages={visible}
        cursor={clampedCursor}
        sortColumn={sortColumn}
        descending={descending}
      />
      {mode === 'filter' ? <FilterBar query={query} /> : null}
      {mode === 'export' ? <ExportMenu cursor={exportCursor} /> : null}
      {mode === 'detail' && selectedPackage !== undefined ? (
        <DetailPane pkg={selectedPackage} />
      ) : null}
      {mode === 'help' ? <HelpOverlay /> : null}
      <Box paddingX={1}>
        <Text color="gray">
          {activeFilters.length > 0 && mode !== 'filter'
            ? `${format(messages.filtersLabel, { list: activeFilters.join(', ') })}  `
            : ''}
          {status.length > 0 ? `${status}  ` : ''}
          {messages.footer}
        </Text>
      </Box>
    </Box>
  );
}
