import test from 'node:test';
import assert from 'node:assert/strict';
import { calculateExpression } from '../src/lib/commands/calculator';

function answer(expression: string): string {
  const result = calculateExpression(expression);
  assert.equal(
    result.kind,
    'success',
    `${expression}: ${JSON.stringify(result)}`,
  );
  if (result.kind !== 'success') throw new Error('Expected calculator result');
  return result.formatted;
}

test('arithmetic respects precedence and parentheses', () => {
  assert.equal(answer('2 + 2'), '4');
  assert.equal(answer('120 * 0.15'), '18');
  assert.equal(answer('(50 + 20) / 2'), '35');
  assert.equal(answer('1 + 2 * 3 - 4 / 2'), '5');
  assert.equal(answer('8 / 2 / 2'), '2');
});

test('decimals, unary operators, and common math glyphs work', () => {
  assert.equal(answer('-.5 * (-2 + .25)'), '0.875');
  assert.equal(answer('2 × (6 − 2) ÷ 4'), '2');
  assert.equal(answer('0.1 + 0.2'), '0.3');
  assert.equal(answer('-0'), '0');
});

test('invalid and unfinished expressions do not produce answers', () => {
  for (const expression of [
    '2 +',
    '(2 + 3',
    '2..4',
    '2 3',
    '2 ** 4',
    '()',
    '2)',
  ]) {
    assert.equal(calculateExpression(expression).kind, 'error', expression);
  }
  const zero = calculateExpression('1 / (2 - 2)');
  assert.deepEqual(zero, { kind: 'error', message: 'Cannot divide by zero' });
});

test('text is search input and JavaScript is never evaluated', () => {
  for (const input of [
    '',
    'notepad',
    'Math.random()',
    'alert(1)',
    '1; globalThis.compromised = true',
  ]) {
    assert.deepEqual(calculateExpression(input), { kind: 'none' });
  }
});

test('input and recursion limits contain expensive expressions', () => {
  assert.equal(calculateExpression('1+'.repeat(130) + '1').kind, 'error');
  assert.equal(
    calculateExpression('('.repeat(40) + '1' + ')'.repeat(40)).kind,
    'error',
  );
  assert.equal(calculateExpression('-'.repeat(40) + '1').kind, 'error');
  assert.equal(
    calculateExpression('9'.repeat(200) + '* ' + '9'.repeat(50)).kind,
    'success',
  );
  assert.equal(
    calculateExpression('9'.repeat(150) + '* 9'.repeat(40)).kind,
    'error',
  );
});
