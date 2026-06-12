import type { Locale } from './i18n.js';

export type FailOn = 'gpl' | 'agpl';

export interface CliOptions {
  json: boolean;
  html: string | null;
  csv: string | null;
  markdown: string | null;
  failOn: FailOn | null;
  path: string;
  ignore: string[];
  locale: Locale;
  help: boolean;
  version: boolean;
}

export const USAGE = `dep-lens: scan dependencies for license risk

USAGE:
    dep-lens [OPTIONS]

OPTIONS:
    --json             Print the raw JSON report to stdout (no TUI)
    --html <FILE>      Write an HTML report to FILE (no TUI)
    --csv <FILE>       Write a CSV report to FILE (no TUI)
    --md <FILE>        Write a Markdown report to FILE (no TUI)
    --fail-on <KIND>   Exit with code 1 when matching licenses are found.
                       KIND is "gpl" (any strong copyleft: GPL, AGPL) or
                       "agpl" (AGPL only). Intended for CI/CD pipelines.
    --path <DIR>       Project directory to scan (default: current directory)
    --ignore <NAMES>   Comma-separated package names to exclude (repeatable)
    --tr               Turkish UI (Turkce arayuz)
    --help             Show this help
    --version          Show version

KEYS (interactive TUI):
    up/down            Move selection (pgup/pgdn jump 10, g/G top/bottom)
    enter              Open package detail pane
    f                  Filter by package name, license, or category
    1 / 2 / 3 / 4      Quick filter: Permissive / Weak / Strong / Unknown
    0                  Clear all filters
    s                  Cycle sort column
    r                  Reverse sort direction
    e                  Export menu (JSON / HTML)
    h                  Help overlay
    q                  Quit
`;

/**
 * Parse CLI arguments. Throws an Error with a user-facing message on
 * unknown flags, missing values, or invalid --fail-on kinds.
 */
export function parseArgs(argv: readonly string[]): CliOptions {
  const options: CliOptions = {
    json: false,
    html: null,
    csv: null,
    markdown: null,
    failOn: null,
    path: '.',
    ignore: [],
    locale: 'en',
    help: false,
    version: false,
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    switch (arg) {
      case '--json':
        options.json = true;
        break;
      case '--html':
        options.html = requireValue(argv, i, '--html');
        i += 1;
        break;
      case '--csv':
        options.csv = requireValue(argv, i, '--csv');
        i += 1;
        break;
      case '--md':
      case '--markdown':
        options.markdown = requireValue(argv, i, '--md');
        i += 1;
        break;
      case '--fail-on': {
        const value = requireValue(argv, i, '--fail-on');
        i += 1;
        if (value !== 'gpl' && value !== 'agpl') {
          throw new Error(`--fail-on must be "gpl" or "agpl", got "${value}"`);
        }
        options.failOn = value;
        break;
      }
      case '--path':
        options.path = requireValue(argv, i, '--path');
        i += 1;
        break;
      case '--ignore': {
        const value = requireValue(argv, i, '--ignore');
        i += 1;
        options.ignore.push(
          ...value
            .split(',')
            .map((name) => name.trim())
            .filter((name) => name.length > 0),
        );
        break;
      }
      case '--tr':
        options.locale = 'tr';
        break;
      case '--help':
      case '-h':
        options.help = true;
        break;
      case '--version':
      case '-V':
        options.version = true;
        break;
      default:
        throw new Error(`unknown argument: ${String(arg)}`);
    }
  }
  return options;
}

function requireValue(argv: readonly string[], index: number, flag: string): string {
  const value = argv[index + 1];
  if (value === undefined || value.startsWith('--')) {
    throw new Error(`${flag} requires a value`);
  }
  return value;
}
