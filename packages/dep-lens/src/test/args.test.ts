import assert from 'node:assert/strict';
import test from 'node:test';

import { parseArgs } from '../args.js';

test('defaults', () => {
  const options = parseArgs([]);
  assert.deepEqual(options, {
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
  });
});

test('--tr switches the TUI locale to Turkish', () => {
  assert.equal(parseArgs(['--tr']).locale, 'tr');
  assert.equal(parseArgs([]).locale, 'en');
});

test('--json sets json output', () => {
  assert.equal(parseArgs(['--json']).json, true);
});

test('--html captures the output file', () => {
  assert.equal(parseArgs(['--html', 'out.html']).html, 'out.html');
});

test('--html without a value throws', () => {
  assert.throws(() => parseArgs(['--html']), /--html requires a value/);
});

test('--fail-on accepts gpl and agpl', () => {
  assert.equal(parseArgs(['--fail-on', 'gpl']).failOn, 'gpl');
  assert.equal(parseArgs(['--fail-on', 'agpl']).failOn, 'agpl');
});

test('--fail-on rejects other values', () => {
  assert.throws(() => parseArgs(['--fail-on', 'mit']), /must be "gpl" or "agpl"/);
});

test('--path overrides the scan directory', () => {
  assert.equal(parseArgs(['--path', '/tmp/project']).path, '/tmp/project');
});

test('--ignore is repeatable and comma-separated', () => {
  const options = parseArgs(['--ignore', 'left-pad', '--ignore', 'a, b ,', '--ignore', 'c']);
  assert.deepEqual(options.ignore, ['left-pad', 'a', 'b', 'c']);
});

test('unknown flags throw', () => {
  assert.throws(() => parseArgs(['--bogus']), /unknown argument: --bogus/);
});

test('--help and --version are recognized', () => {
  assert.equal(parseArgs(['--help']).help, true);
  assert.equal(parseArgs(['-h']).help, true);
  assert.equal(parseArgs(['--version']).version, true);
  assert.equal(parseArgs(['-V']).version, true);
});
