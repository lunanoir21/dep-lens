import assert from 'node:assert/strict';
import test from 'node:test';

import {
  filterPackages,
  pad,
  percent,
  sortPackages,
  truncate,
  violations,
} from '../utils.js';
import { pkg, report } from './fixtures.js';

test('filterPackages matches name, license, and category case-insensitively', () => {
  const packages = [
    pkg({ name: 'alpha', license: 'MIT', category: 'Permissive' }),
    pkg({ name: 'beta', license: 'GPL-3.0', category: 'StrongCopyleft' }),
    pkg({ name: 'gamma', license: 'UNKNOWN', category: 'Unknown' }),
  ];
  assert.deepEqual(
    filterPackages(packages, 'ALPHA').map((p) => p.name),
    ['alpha'],
  );
  assert.deepEqual(
    filterPackages(packages, 'gpl').map((p) => p.name),
    ['beta'],
  );
  assert.deepEqual(
    filterPackages(packages, 'strongcopyleft').map((p) => p.name),
    ['beta'],
  );
  assert.equal(filterPackages(packages, '  ').length, 3);
  assert.equal(filterPackages(packages, 'nope').length, 0);
});

test('filterPackages restricts by category and combines with text query', () => {
  const packages = [
    pkg({ name: 'alpha', license: 'MIT', category: 'Permissive' }),
    pkg({ name: 'alpine', license: 'GPL-3.0', category: 'StrongCopyleft' }),
    pkg({ name: 'beta', license: 'AGPL-3.0', category: 'StrongCopyleft' }),
  ];
  assert.deepEqual(
    filterPackages(packages, '', 'StrongCopyleft').map((p) => p.name),
    ['alpine', 'beta'],
  );
  assert.deepEqual(
    filterPackages(packages, 'alp', 'StrongCopyleft').map((p) => p.name),
    ['alpine'],
  );
  assert.equal(filterPackages(packages, '', null).length, 3);
});

test('sortPackages sorts by name ascending and descending', () => {
  const packages = [pkg({ name: 'b' }), pkg({ name: 'a' }), pkg({ name: 'c' })];
  assert.deepEqual(
    sortPackages(packages, 'name', false).map((p) => p.name),
    ['a', 'b', 'c'],
  );
  assert.deepEqual(
    sortPackages(packages, 'name', true).map((p) => p.name),
    ['c', 'b', 'a'],
  );
});

test('sortPackages sorts riskScore numerically with name tiebreak', () => {
  const packages = [
    pkg({ name: 'z', riskScore: 0 }),
    pkg({ name: 'a', riskScore: 100 }),
    pkg({ name: 'm', riskScore: 0 }),
  ];
  assert.deepEqual(
    sortPackages(packages, 'riskScore', true).map((p) => p.name),
    ['a', 'm', 'z'],
  );
});

test('sortPackages orders categories by restrictiveness', () => {
  const packages = [
    pkg({ name: 'u', category: 'Unknown' }),
    pkg({ name: 's', category: 'StrongCopyleft' }),
    pkg({ name: 'p', category: 'Permissive' }),
    pkg({ name: 'w', category: 'WeakCopyleft' }),
  ];
  assert.deepEqual(
    sortPackages(packages, 'category', false).map((p) => p.name),
    ['p', 'w', 's', 'u'],
  );
});

test('sortPackages does not mutate its input', () => {
  const packages = [pkg({ name: 'b' }), pkg({ name: 'a' })];
  sortPackages(packages, 'name', false);
  assert.deepEqual(
    packages.map((p) => p.name),
    ['b', 'a'],
  );
});

test('percent formats one decimal and handles zero totals', () => {
  assert.equal(percent(1, 3), '33.3');
  assert.equal(percent(0, 0), '0.0');
  assert.equal(percent(2, 2), '100.0');
});

test('truncate and pad produce exact widths with ASCII ellipsis', () => {
  assert.equal(truncate('short', 10), 'short');
  assert.equal(truncate('a-very-long-name', 10), 'a-very-l..');
  assert.equal(pad('ab', 5), 'ab   ');
  assert.equal(pad('abcdef', 5).length, 5);
});

test('violations for gpl matches all strong copyleft, not LGPL', () => {
  const r = report([
    pkg({ name: 'mit', license: 'MIT', category: 'Permissive' }),
    pkg({ name: 'lgpl', license: 'LGPL-3.0', category: 'WeakCopyleft' }),
    pkg({ name: 'gpl', license: 'GPL-3.0', category: 'StrongCopyleft' }),
    pkg({ name: 'agpl', license: 'AGPL-3.0', category: 'StrongCopyleft' }),
  ]);
  assert.deepEqual(
    violations(r, 'gpl').map((p) => p.name),
    ['gpl', 'agpl'],
  );
});

test('violations for agpl matches only AGPL', () => {
  const r = report([
    pkg({ name: 'gpl', license: 'GPL-2.0', category: 'StrongCopyleft' }),
    pkg({ name: 'agpl', license: 'AGPL-3.0', category: 'StrongCopyleft' }),
  ]);
  assert.deepEqual(
    violations(r, 'agpl').map((p) => p.name),
    ['agpl'],
  );
});

test('violations ignores dual-licensed permissive packages', () => {
  const r = report([
    pkg({ name: 'dual', license: 'GPL-2.0 OR MIT', category: 'Permissive' }),
  ]);
  assert.equal(violations(r, 'gpl').length, 0);
  assert.equal(violations(r, 'agpl').length, 0);
});
