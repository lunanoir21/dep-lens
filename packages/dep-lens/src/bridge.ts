import { execFile } from 'node:child_process';
import { existsSync } from 'node:fs';
import { createRequire } from 'node:module';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { promisify } from 'node:util';

import type { Report } from './types.js';

const execFileAsync = promisify(execFile);

export interface ScanOptions {
  path: string;
  ignore: readonly string[];
  locale?: string;
}

const PLATFORM_PACKAGES: Record<string, string> = {
  'linux-x64': '@lunanoir/dep-lens-linux-x64',
  'darwin-arm64': '@lunanoir/dep-lens-darwin-arm64',
  'darwin-x64': '@lunanoir/dep-lens-darwin-x64',
  'win32-x64': '@lunanoir/dep-lens-win32-x64',
};

function binaryName(): string {
  return process.platform === 'win32' ? 'dep-lens-core.exe' : 'dep-lens-core';
}

/**
 * Locate the dep-lens-core binary. Resolution order:
 * 1. DEP_LENS_BINARY environment variable (explicit override)
 * 2. The platform-specific optional dependency (normal installs)
 * 3. The Cargo target directory of this repository (development)
 */
export function resolveBinaryPath(): string {
  const override = process.env['DEP_LENS_BINARY'];
  if (override !== undefined && override.length > 0) {
    return override;
  }

  const platformKey = `${process.platform}-${process.arch}`;
  const packageName = PLATFORM_PACKAGES[platformKey];
  if (packageName !== undefined) {
    try {
      const require = createRequire(import.meta.url);
      return require.resolve(`${packageName}/bin/${binaryName()}`);
    } catch {
      // Platform package not installed; fall through to development paths.
    }
  }

  const here = path.dirname(fileURLToPath(import.meta.url));
  for (const profile of ['release', 'debug']) {
    const candidate = path.resolve(here, '..', '..', '..', 'target', profile, binaryName());
    if (existsSync(candidate)) {
      return candidate;
    }
  }

  throw new Error(
    `no native binary available for ${platformKey}. ` +
      'Reinstall dep-lens so the platform package is picked up, or set DEP_LENS_BINARY.',
  );
}

function scanArgs(options: ScanOptions): string[] {
  const args = ['--path', options.path];
  for (const name of options.ignore) {
    args.push('--ignore', name);
  }
  if (options.locale) {
    args.push('--lang', options.locale);
  }
  return args;
}

async function runCore(args: readonly string[]): Promise<string> {
  const binary = resolveBinaryPath();
  try {
    const { stdout } = await execFileAsync(binary, [...args], {
      maxBuffer: 64 * 1024 * 1024,
    });
    return stdout;
  } catch (error: unknown) {
    const stderr =
      typeof error === 'object' && error !== null && 'stderr' in error
        ? String((error as { stderr: unknown }).stderr).trim()
        : '';
    throw new Error(`core scanner failed${stderr.length > 0 ? `: ${stderr}` : ''}`);
  }
}

/** Structural validation of the core binary's JSON output. */
export function validateReport(value: unknown): Report {
  if (typeof value !== 'object' || value === null) {
    throw new Error('core output is not a JSON object');
  }
  const record = value as Record<string, unknown>;
  for (const field of ['project', 'scannedAt', 'path']) {
    if (typeof record[field] !== 'string') {
      throw new Error(`core output is missing string field "${field}"`);
    }
  }
  const summary = record['summary'];
  if (typeof summary !== 'object' || summary === null) {
    throw new Error('core output is missing "summary"');
  }
  if (!Array.isArray(record['packages'])) {
    throw new Error('core output is missing "packages" array');
  }
  return value as Report;
}

/** Run a scan and return the parsed report. */
export async function runScan(options: ScanOptions): Promise<Report> {
  const stdout = await runCore([...scanArgs(options), '--format', 'json']);
  let parsed: unknown;
  try {
    parsed = JSON.parse(stdout);
  } catch {
    throw new Error('core output is not valid JSON');
  }
  return validateReport(parsed);
}

/** Run a scan and return the standalone HTML report. */
export async function renderHtml(options: ScanOptions): Promise<string> {
  return runCore([...scanArgs(options), '--format', 'html']);
}

/** Run a scan and return the CSV report. */
export async function renderCsv(options: ScanOptions): Promise<string> {
  return runCore([...scanArgs(options), '--format', 'csv']);
}

/** Run a scan and return the Markdown report. */
export async function renderMarkdown(options: ScanOptions): Promise<string> {
  return runCore([...scanArgs(options), '--format', 'markdown']);
}
