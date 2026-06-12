import assert from 'node:assert/strict';
import test from 'node:test';

import { EN, TR } from '../i18n.js';
import { computeColumns } from '../ui/PackageTable.js';

test('computeColumns keeps the canonical column order', () => {
  const keys = computeColumns(120, EN).map((column) => column.key);
  assert.deepEqual(keys, [
    'name',
    'version',
    'license',
    'category',
    'dependencyType',
    'riskScore',
    'commercialUse',
  ]);
});

test('computeColumns fits within the terminal width', () => {
  for (const width of [80, 100, 132, 200]) {
    for (const messages of [EN, TR]) {
      const columns = computeColumns(width, messages);
      const separators = columns.length - 1;
      const total = columns.reduce((sum, column) => sum + column.width, 0) + separators;
      const chrome = 4;
      assert.ok(
        total <= width - chrome,
        `width ${width}: columns occupy ${total}, budget ${width - chrome}`,
      );
    }
  }
});

test('computeColumns grows the name column on wide terminals', () => {
  const narrow = computeColumns(80, EN).find((column) => column.key === 'name');
  const wide = computeColumns(200, EN).find((column) => column.key === 'name');
  assert.ok(narrow !== undefined && wide !== undefined);
  assert.ok(wide.width > narrow.width);
});

test('computeColumns enforces minimum widths on tiny terminals', () => {
  const columns = computeColumns(40, EN);
  const name = columns.find((column) => column.key === 'name');
  const license = columns.find((column) => column.key === 'license');
  assert.ok(name !== undefined && license !== undefined);
  assert.ok(name.width >= 10);
  assert.ok(license.width >= 6);
});

test('localized header labels fit their fixed column widths', () => {
  for (const messages of [EN, TR]) {
    const columns = computeColumns(100, messages);
    for (const column of columns) {
      // +2 leaves room for the sort marker (" ^" / " v").
      assert.ok(
        column.label.length + 2 <= column.width || column.key === 'name',
        `label "${column.label}" overflows width ${column.width}`,
      );
    }
  }
});
