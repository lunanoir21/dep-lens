import assert from 'node:assert/strict';
import test from 'node:test';

import { EN, format, getMessages, TR } from '../i18n.js';

function shapeOf(value: unknown): unknown {
  if (Array.isArray(value)) {
    return 'array';
  }
  if (typeof value === 'object' && value !== null) {
    const out: Record<string, unknown> = {};
    for (const key of Object.keys(value).sort()) {
      out[key] = shapeOf((value as Record<string, unknown>)[key]);
    }
    return out;
  }
  return typeof value;
}

test('EN and TR catalogs have identical structure', () => {
  assert.deepEqual(shapeOf(TR), shapeOf(EN));
});

test('help bindings cover the same keys in both languages', () => {
  assert.equal(TR.help.bindings.length, EN.help.bindings.length);
});

test('placeholders survive translation', () => {
  for (const messages of [EN, TR]) {
    assert.ok(messages.table.rows.includes('{start}'));
    assert.ok(messages.table.rows.includes('{end}'));
    assert.ok(messages.table.rows.includes('{total}'));
    assert.ok(messages.exportMenu.writing.includes('{file}'));
    assert.ok(messages.exportMenu.wrote.includes('{file}'));
    assert.ok(messages.scanning.elapsed.includes('{seconds}'));
    assert.ok(messages.detail.riskValue.includes('{score}'));
  }
});

test('format replaces placeholders and leaves unknown ones intact', () => {
  assert.equal(format('rows {start}-{end} of {total}', { start: 1, end: 5, total: 9 }), 'rows 1-5 of 9');
  assert.equal(format('{missing} stays', {}), '{missing} stays');
});

test('getMessages selects the catalog', () => {
  assert.equal(getMessages('en'), EN);
  assert.equal(getMessages('tr'), TR);
});
