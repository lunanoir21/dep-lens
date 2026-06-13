import { execFile } from 'node:child_process';
import path from 'node:path';
import { promisify } from 'node:util';

import { bad, bold, brand, dim, good, ok } from './ansi.js';
import { resolveBinaryPath, runScan, type ScanOptions } from './bridge.js';
import { detectEcosystems } from './detect.js';
import type { Ecosystem } from './types.js';

const execFileAsync = promisify(execFile);

interface Check {
  label: string;
  ok: boolean;
  detail: string;
}

function statusGlyph(passed: boolean): string {
  return passed ? good('PASS') : bad('FAIL');
}

/**
 * `dep-lens --test`: verify the native scanner binary runs, scan
 * `options.path`, and report which of the ecosystems detected by their
 * manifest files actually produced packages. Returns the process exit code.
 */
export async function runSelfTest(options: ScanOptions): Promise<number> {
  const checks: Check[] = [];
  let exitCode = 0;

  process.stdout.write(`${bold(brand('dep-lens --test'))} ${dim(`(${path.resolve(options.path)})`)}\n\n`);

  // 1. Binary resolves and runs.
  let binaryPath: string;
  try {
    binaryPath = resolveBinaryPath();
  } catch (error: unknown) {
    checks.push({
      label: 'native scanner binary',
      ok: false,
      detail: error instanceof Error ? error.message : String(error),
    });
    printChecks(checks);
    return 2;
  }

  try {
    const { stdout } = await execFileAsync(binaryPath, ['--version']);
    checks.push({ label: 'native scanner binary', ok: true, detail: stdout.trim() });
  } catch (error: unknown) {
    checks.push({
      label: 'native scanner binary',
      ok: false,
      detail: error instanceof Error ? error.message : String(error),
    });
    printChecks(checks);
    return 2;
  }

  // 2. Run a scan.
  let report;
  try {
    report = await runScan(options);
    checks.push({
      label: 'scan',
      ok: true,
      detail: `${report.summary.total} package(s) found`,
    });
  } catch (error: unknown) {
    checks.push({
      label: 'scan',
      ok: false,
      detail: error instanceof Error ? error.message : String(error),
    });
    printChecks(checks);
    return 2;
  }

  // 3. Per-ecosystem: every manifest dep-lens recognizes in this project
  //    should have produced at least one package.
  const detected = detectEcosystems(options.path);
  const foundEcosystems = new Set<Ecosystem>(report.packages.map((pkg) => pkg.ecosystem));

  for (const eco of detected) {
    const found = foundEcosystems.has(eco.id as Ecosystem);
    const count = report.packages.filter((pkg) => pkg.ecosystem === eco.id).length;
    if (!found) {
      exitCode = 1;
    }
    checks.push({
      label: eco.label,
      ok: found,
      detail: found ? `${count} package(s)` : 'manifest found but no packages reported',
    });
  }

  if (detected.length === 0) {
    checks.push({
      label: 'ecosystems',
      ok: true,
      detail: 'no recognized manifests in this directory',
    });
  }

  // 4. Unknown licenses are a warning, not a failure.
  if (report.summary.unknown > 0) {
    checks.push({
      label: 'license coverage',
      ok: true,
      detail: `${ok(String(report.summary.unknown))} package(s) with unknown license`,
    });
  }

  printChecks(checks);

  process.stdout.write(
    `\n${exitCode === 0 ? good('all checks passed') : bad('some checks failed')}\n`,
  );
  return exitCode;
}

function printChecks(checks: Check[]): void {
  const width = Math.max(...checks.map((c) => c.label.length), 8);
  for (const check of checks) {
    process.stdout.write(`  ${statusGlyph(check.ok)}  ${check.label.padEnd(width)}  ${dim(check.detail)}\n`);
  }
}
