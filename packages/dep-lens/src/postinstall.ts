#!/usr/bin/env node
/**
 * Postinstall setup wizard. Runs once after `npm install`, detects the
 * caller's project ecosystems, asks for a default UI language, and checks
 * whether the global npm bin directory (where the `dep-lens` launcher
 * lands) is on PATH.
 *
 * Skips entirely in non-interactive environments (CI, piped installs, or
 * when npm doesn't attach a TTY to lifecycle scripts) so it never blocks an
 * install.
 */
import { execFile } from 'node:child_process';
import { createInterface } from 'node:readline';
import { promisify } from 'node:util';

import { accent, bad, bold, brand, dim, good } from './ansi.js';
import { detectEcosystems } from './detect.js';
import { readConfig, writeConfig } from './config.js';

const execFileAsync = promisify(execFile);

const PROMPT_TIMEOUT_MS = 15_000;

function isInteractive(): boolean {
  if (process.env['CI'] === 'true' || process.env['CI'] === '1') {
    return false;
  }
  if (process.env['DEP_LENS_SKIP_SETUP'] === '1') {
    return false;
  }
  return process.stdin.isTTY === true && process.stdout.isTTY === true;
}

/** Ask a yes/no question with a default answer if the user just presses enter. */
async function askYesNo(question: string, defaultYes: boolean): Promise<boolean> {
  const rl = createInterface({ input: process.stdin, output: process.stdout });
  const suffix = defaultYes ? 'Y/n' : 'y/N';
  return new Promise((resolve) => {
    const timer = setTimeout(() => {
      rl.close();
      resolve(defaultYes);
    }, PROMPT_TIMEOUT_MS);
    rl.question(`${question} [${suffix}] `, (answer) => {
      clearTimeout(timer);
      rl.close();
      const trimmed = answer.trim().toLowerCase();
      if (trimmed === '') {
        resolve(defaultYes);
        return;
      }
      resolve(trimmed === 'y' || trimmed === 'yes' || trimmed === 'e' || trimmed === 'evet');
    });
  });
}

async function npmGlobalBinDir(): Promise<string | null> {
  try {
    const { stdout } = await execFileAsync('npm', ['bin', '-g']);
    const dir = stdout.trim();
    return dir.length > 0 ? dir : null;
  } catch {
    try {
      const { stdout } = await execFileAsync('npm', ['prefix', '-g']);
      const prefix = stdout.trim();
      return prefix.length > 0 ? `${prefix}/bin` : null;
    } catch {
      return null;
    }
  }
}

function pathContains(dir: string): boolean {
  const pathEnv = process.env['PATH'] ?? '';
  const sep = process.platform === 'win32' ? ';' : ':';
  return pathEnv.split(sep).some((entry) => entry.replace(/[/\\]+$/, '') === dir.replace(/[/\\]+$/, ''));
}

/**
 * Run the setup wizard. `force` skips the TTY/CI checks (used by
 * `dep-lens --setup`, where the user explicitly asked for it).
 */
export async function main(force = false): Promise<void> {
  // Always print a one-line banner so even non-interactive installs show
  // something useful, but never prompt unless we have a real TTY.
  const cwd = process.env['INIT_CWD'] ?? process.cwd();
  const detected = detectEcosystems(cwd);

  process.stdout.write(`\n${bold(brand('dep-lens'))} installed.\n`);

  if (detected.length > 0) {
    const labels = detected.map((eco) => eco.label).join(', ');
    process.stdout.write(`${dim('detected in this project:')} ${good(labels)}\n`);
  }

  if (!force && !isInteractive()) {
    process.stdout.write(`${dim('run')} dep-lens ${dim('to scan, or')} dep-lens --help\n\n`);
    return;
  }

  process.stdout.write(`\n${bold('Quick setup')} ${dim('(press enter to accept defaults)')}\n`);

  // --- default language -----------------------------------------------
  const config = await readConfig();
  const wantsTurkish = await askYesNo(
    `${accent('?')} Use Turkish UI by default (--tr)?`,
    config.locale === 'tr',
  );
  config.locale = wantsTurkish ? 'tr' : 'en';
  await writeConfig(config);

  // --- PATH check --------------------------------------------------------
  const binDir = await npmGlobalBinDir();
  if (binDir !== null && !pathContains(binDir)) {
    process.stdout.write(
      `\n${bad('!')} ${binDir} is not on your PATH, so ${bold('dep-lens')} may not run yet.\n`,
    );
    const addPath = await askYesNo(`${accent('?')} Add it to your shell profile now?`, true);
    if (addPath) {
      await appendToShellProfile(binDir);
    } else {
      process.stdout.write(`${dim('add manually:')} export PATH="${binDir}:$PATH"\n`);
    }
  }

  process.stdout.write(`\n${good('done.')} ${dim('run')} dep-lens ${dim('to get started.')}\n\n`);
}

async function appendToShellProfile(binDir: string): Promise<void> {
  if (process.platform === 'win32') {
    process.stdout.write(
      `${dim('on Windows, add this to your PATH via System Properties > Environment Variables:')}\n${binDir}\n`,
    );
    return;
  }
  const { homedir } = await import('node:os');
  const { appendFile, readFile } = await import('node:fs/promises');
  const path = await import('node:path');

  const shell = (process.env['SHELL'] ?? '').split('/').pop() ?? '';
  const rcFile =
    shell === 'fish'
      ? path.join(homedir(), '.config', 'fish', 'config.fish')
      : shell === 'zsh'
        ? path.join(homedir(), '.zshrc')
        : path.join(homedir(), '.bashrc');

  const line =
    shell === 'fish'
      ? `fish_add_path "${binDir}"`
      : `export PATH="${binDir}:$PATH"`;

  try {
    const existing = await readFile(rcFile, 'utf8').catch(() => '');
    if (existing.includes(binDir)) {
      process.stdout.write(`${dim('already present in')} ${rcFile}\n`);
      return;
    }
    await appendFile(rcFile, `\n# added by dep-lens postinstall\n${line}\n`);
    process.stdout.write(`${good('added to')} ${rcFile}${dim(' - open a new terminal to apply.')}\n`);
  } catch {
    process.stdout.write(`${dim('could not update shell profile, add manually:')} ${line}\n`);
  }
}

// Only auto-run when invoked directly as the postinstall script, not when
// imported by `dep-lens --setup`.
if (process.argv[1]?.endsWith('postinstall.js')) {
  main().catch(() => {
    // Never fail the install over the setup wizard.
  });
}
