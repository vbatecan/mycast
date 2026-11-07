import test from 'node:test';
import assert from 'node:assert/strict';
import { conversationHistory } from '../src/lib/ai/conversation';

const turn = (
  prompt: string,
  text: string,
  status = 'Complete',
  error = '',
) => ({ prompt, text, status, error });
test('follow-ups include completed user/assistant pairs in order', () => {
  assert.deepEqual(
    conversationHistory([
      turn('My project is Mycast', 'I will use that name'),
      turn('Which language?', 'Rust and TypeScript'),
    ]),
    [
      { role: 'user', content: 'My project is Mycast' },
      { role: 'assistant', content: 'I will use that name' },
      { role: 'user', content: 'Which language?' },
      { role: 'assistant', content: 'Rust and TypeScript' },
    ],
  );
});
test('failed, empty, and stopped answers are not sent as completed context', () => {
  assert.deepEqual(
    conversationHistory([
      turn('good', 'answer'),
      turn('failed', 'partial', 'Request failed', 'error'),
      turn('stopped', 'partial', 'Stopped'),
      turn('empty', ''),
    ]),
    [
      { role: 'user', content: 'good' },
      { role: 'assistant', content: 'answer' },
    ],
  );
});
test('context limits preserve whole recent exchanges and UTF-8 byte bounds', () => {
  const history = conversationHistory(
    Array.from({ length: 20 }, (_, index) =>
      turn(String(index), '答'.repeat(3000)),
    ),
  );
  assert.equal(history.length, 32);
  assert.equal(history[0].content, '4');
  assert.ok(
    history.reduce(
      (bytes, message) => bytes + Buffer.byteLength(message.content),
      0,
    ) <= 256_000,
  );
  assert.deepEqual(conversationHistory([]), []);
});
