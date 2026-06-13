import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { homedir } from 'node:os';
import path from 'node:path';

import type { Locale } from './i18n.js';

export interface DepLensConfig {
  locale?: Locale;
  setupDone?: boolean;
}

function configDir(): string {
  const xdg = process.env['XDG_CONFIG_HOME'];
  const base = xdg && xdg.length > 0 ? xdg : path.join(homedir(), '.config');
  return path.join(base, 'dep-lens');
}

export function configPath(): string {
  return path.join(configDir(), 'config.json');
}

/** Read the user config; returns `{}` if missing or unreadable/invalid. */
export async function readConfig(): Promise<DepLensConfig> {
  try {
    const raw = await readFile(configPath(), 'utf8');
    const parsed = JSON.parse(raw) as unknown;
    if (typeof parsed !== 'object' || parsed === null) {
      return {};
    }
    const record = parsed as Record<string, unknown>;
    const config: DepLensConfig = {};
    if (record['locale'] === 'en' || record['locale'] === 'tr') {
      config.locale = record['locale'];
    }
    if (typeof record['setupDone'] === 'boolean') {
      config.setupDone = record['setupDone'];
    }
    return config;
  } catch {
    return {};
  }
}

/** Write the user config, creating `~/.config/dep-lens/` if needed. */
export async function writeConfig(config: DepLensConfig): Promise<void> {
  await mkdir(configDir(), { recursive: true });
  await writeFile(configPath(), `${JSON.stringify(config, null, 2)}\n`, 'utf8');
}
