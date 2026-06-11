import assert from 'node:assert/strict';
import test from 'node:test';

import { resolveBinaryPath, validateReport } from '../bridge.js';
import { pkg, report } from './fixtures.js';

test('validateReport accepts a well-formed report', () => {
  const input = report([pkg()]);
  assert.deepEqual(validateReport(JSON.parse(JSON.stringify(input))), input);
});

test('validateReport rejects non-objects', () => {
  assert.throws(() => validateReport(null), /not a JSON object/);
  assert.throws(() => validateReport('text'), /not a JSON object/);
  assert.throws(() => validateReport(42), /not a JSON object/);
});

test('validateReport rejects missing fields', () => {
  assert.throws(() => validateReport({}), /missing string field "project"/);
  assert.throws(
    () =>
      validateReport({
        project: 'x',
        scannedAt: 'now',
        path: '/tmp',
        packages: [],
      }),
    /missing "summary"/,
  );
  assert.throws(
    () =>
      validateReport({
        project: 'x',
        scannedAt: 'now',
        path: '/tmp',
        summary: {},
        packages: 'nope',
      }),
    /missing "packages" array/,
  );
});

test('resolveBinaryPath honors the DEP_LENS_BINARY override', () => {
  const previous = process.env['DEP_LENS_BINARY'];
  process.env['DEP_LENS_BINARY'] = '/opt/custom/dep-lens-core';
  try {
    assert.equal(resolveBinaryPath(), '/opt/custom/dep-lens-core');
  } finally {
    if (previous === undefined) {
      delete process.env['DEP_LENS_BINARY'];
    } else {
      process.env['DEP_LENS_BINARY'] = previous;
    }
  }
});
