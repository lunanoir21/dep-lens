import React, { useEffect, useState } from 'react';

import { getMessages, type Locale } from '../i18n.js';
import type { Report } from '../types.js';
import { App } from './App.js';
import { ErrorScreen } from './ErrorScreen.js';
import { I18nContext } from './i18n-context.js';
import { ScanningScreen } from './ScanningScreen.js';

export interface RootProps {
  locale: Locale;
  scan: () => Promise<Report>;
  getHtml: () => Promise<string>;
  onReport: (report: Report) => void;
  onError: () => void;
}

type Phase =
  | { name: 'scanning' }
  | { name: 'ready'; report: Report }
  | { name: 'error'; message: string };

/**
 * Top-level TUI component: runs the scan with an animated progress screen,
 * then hands over to the interactive report, all under the chosen locale.
 */
export function Root({
  locale,
  scan,
  getHtml,
  onReport,
  onError,
}: RootProps): React.JSX.Element {
  const [phase, setPhase] = useState<Phase>({ name: 'scanning' });

  useEffect(() => {
    let cancelled = false;
    scan()
      .then((report) => {
        if (!cancelled) {
          onReport(report);
          setPhase({ name: 'ready', report });
        }
      })
      .catch((error: unknown) => {
        if (!cancelled) {
          onError();
          setPhase({
            name: 'error',
            message: error instanceof Error ? error.message : String(error),
          });
        }
      });
    return () => {
      cancelled = true;
    };
    // The scan runs exactly once for the lifetime of the TUI.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <I18nContext.Provider value={getMessages(locale)}>
      {phase.name === 'scanning' ? <ScanningScreen /> : null}
      {phase.name === 'error' ? <ErrorScreen message={phase.message} /> : null}
      {phase.name === 'ready' ? <App report={phase.report} getHtml={getHtml} /> : null}
    </I18nContext.Provider>
  );
}
