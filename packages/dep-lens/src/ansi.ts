/**
 * Minimal truecolor ANSI helpers for plain-console output (postinstall
 * wizard, --test report) where rendering a full Ink tree would be overkill.
 * Colors mirror `ui/theme.ts` PALETTE. No-ops when stdout is not a TTY.
 */

function hexToRgb(hex: string): [number, number, number] {
  const value = hex.replace('#', '');
  return [
    parseInt(value.slice(0, 2), 16),
    parseInt(value.slice(2, 4), 16),
    parseInt(value.slice(4, 6), 16),
  ];
}

function colorize(text: string, hex: string): string {
  if (!process.stdout.isTTY) {
    return text;
  }
  const [r, g, b] = hexToRgb(hex);
  return `\x1b[38;2;${r};${g};${b}m${text}\x1b[39m`;
}

export const good = (text: string): string => colorize(text, '#4ade80');
export const ok = (text: string): string => colorize(text, '#fbbf24');
export const bad = (text: string): string => colorize(text, '#fb7185');
export const unknown = (text: string): string => colorize(text, '#94a3b8');
export const brand = (text: string): string => colorize(text, '#38bdf8');
export const accent = (text: string): string => colorize(text, '#a78bfa');
export const dim = (text: string): string => colorize(text, '#64748b');

export function bold(text: string): string {
  if (!process.stdout.isTTY) {
    return text;
  }
  return `\x1b[1m${text}\x1b[22m`;
}
