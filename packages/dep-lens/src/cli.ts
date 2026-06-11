#!/usr/bin/env node
import { writeFile } from 'node:fs/promises';
import { createRequire } from 'node:module';

import React from 'react';
import { render } from 'ink';

import { parseArgs, USAGE, type CliOptions } from './args.js';
import { renderHtml, runScan } from './bridge.js';
import type { Report } from './types.js';
import { violations } from './utils.js';
import { Root } from './ui/Root.js';

function packageVersion(): string {
  const require = createRequire(import.meta.url);
  const manifest = require('../package.json') as { version: string };
  return manifest.version;
}

function applyFailOn(report: Report, options: CliOptions): void {
  if (options.failOn === null) {
    return;
  }
  const offenders = violations(report, options.failOn);
  if (offenders.length > 0) {
    process.stderr.write(
      `dep-lens: --fail-on ${options.failOn} matched ${offenders.length} package(s):\n`,
    );
    for (const pkg of offenders) {
      process.stderr.write(`  ${pkg.name}@${pkg.version} (${pkg.license})\n`);
    }
    process.exitCode = 1;
  }
}

async function main(): Promise<void> {
  let options: CliOptions;
  try {
    options = parseArgs(process.argv.slice(2));
  } catch (error: unknown) {
    process.stderr.write(`dep-lens: ${error instanceof Error ? error.message : String(error)}\n`);
    process.stderr.write('Run dep-lens --help for usage.\n');
    process.exitCode = 2;
    return;
  }

  if (options.help) {
    process.stdout.write(USAGE);
    return;
  }
  if (options.version) {
    process.stdout.write(`dep-lens ${packageVersion()}\n`);
    return;
  }

  const scanOptions = { path: options.path, ignore: options.ignore };

  if (options.json) {
    const report = await runScan(scanOptions);
    process.stdout.write(`${JSON.stringify(report, null, 2)}\n`);
    applyFailOn(report, options);
    return;
  }
  if (options.html !== null) {
    const report = await runScan(scanOptions);
    const html = await renderHtml(scanOptions);
    await writeFile(options.html, html);
    process.stderr.write(`dep-lens: HTML report written to ${options.html}\n`);
    applyFailOn(report, options);
    return;
  }
  if (process.stdout.isTTY !== true) {
    // Piped or redirected output: emit JSON instead of starting the TUI.
    const report = await runScan(scanOptions);
    process.stdout.write(`${JSON.stringify(report, null, 2)}\n`);
    applyFailOn(report, options);
    return;
  }

  // Interactive TUI: the scan runs inside the UI behind an animated
  // progress screen; the report is captured for the --fail-on gate.
  const captured: { report: Report | null } = { report: null };
  const app = render(
    React.createElement(Root, {
      locale: options.locale,
      scan: () => runScan(scanOptions),
      getHtml: () => renderHtml(scanOptions),
      onReport: (report: Report) => {
        captured.report = report;
      },
      onError: () => {
        process.exitCode = 2;
      },
    }),
  );
  await app.waitUntilExit();
  if (captured.report !== null) {
    applyFailOn(captured.report, options);
  }
}

main().catch((error: unknown) => {
  process.stderr.write(`dep-lens: ${error instanceof Error ? error.message : String(error)}\n`);
  process.exitCode = 2;
});
