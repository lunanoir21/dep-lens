import assert from 'node:assert/strict';
import test from 'node:test';

import { easeOutCubic } from '../ui/hooks.js';
import { buildRatioSegments } from '../utils.js';

const SUMMARY = {
  total: 22,
  permissive: 11,
  weakCopyleft: 4,
  strongCopyleft: 5,
  unknown: 2,
};

test('easeOutCubic is anchored and monotonic', () => {
  assert.equal(easeOutCubic(0), 0);
  assert.equal(easeOutCubic(1), 1);
  let previous = -1;
  for (let i = 0; i <= 10; i += 1) {
    const value = easeOutCubic(i / 10);
    assert.ok(value >= previous);
    previous = value;
  }
});

test('ratio segments fill the requested width exactly', () => {
  for (const width of [20, 40, 76]) {
    const segments = buildRatioSegments(SUMMARY, width, 1);
    const total = segments.reduce((sum, segment) => sum + segment.width, 0);
    assert.equal(total, width);
  }
});

test('ratio segments scale with progress', () => {
  const half = buildRatioSegments(SUMMARY, 40, 0.5);
  const total = half.reduce((sum, segment) => sum + segment.width, 0);
  assert.equal(total, 20);
  assert.deepEqual(buildRatioSegments(SUMMARY, 40, 0), []);
});

test('ratio segments use distinct ASCII characters per category', () => {
  const segments = buildRatioSegments(SUMMARY, 40, 1);
  const chars = segments.map((segment) => segment.char);
  assert.deepEqual(chars, ['#', '=', '!', '?']);
  for (const segment of segments) {
    assert.ok(segment.char.charCodeAt(0) < 128);
  }
});

test('empty summaries produce an empty bar', () => {
  assert.deepEqual(
    buildRatioSegments(
      { total: 0, permissive: 0, weakCopyleft: 0, strongCopyleft: 0, unknown: 0 },
      40,
      1,
    ),
    [],
  );
});
