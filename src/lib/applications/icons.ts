import { writable } from 'svelte/store';
import { getApplicationIcon, isNative } from '../native';

const maximumCached = 128;
const maximumConcurrent = 3;
const maximumQueued = 64;
const cache = new Map<string, Promise<string | null>>();
const generation = writable(0);
export const iconGeneration = { subscribe: generation.subscribe };
interface Request {
  id: string;
  signal: AbortSignal;
  promise: Promise<string | null>;
  resolve: (value: string | null) => void;
}
const queue: Request[] = [];
let active = 0;

function forget(request: Request): void {
  if (cache.get(request.id) === request.promise) cache.delete(request.id);
}

function drain(): void {
  while (active < maximumConcurrent && queue.length) {
    const request = queue.shift()!;
    if (request.signal.aborted) {
      forget(request);
      request.resolve(null);
      continue;
    }
    active++;
    void getApplicationIcon(request.id)
      .then((value) =>
        request.resolve(
          value &&
            value.length <= 98304 &&
            /^data:image\/png;base64,[a-z0-9+/]+={0,2}$/i.test(value)
            ? value
            : null,
        ),
      )
      .catch(() => {
        forget(request);
        request.resolve(null);
      })
      .finally(() => {
        active--;
        drain();
      });
  }
}

/** Only visible rows ask for icons; offscreen requests are cancelled before work starts. */
export function requestApplicationIcon(
  id: string,
  signal: AbortSignal,
): Promise<string | null> {
  if (!isNative() || signal.aborted) return Promise.resolve(null);
  const existing = cache.get(id);
  if (existing) return existing;
  if (queue.length >= maximumQueued) return Promise.resolve(null);
  let resolve!: (value: string | null) => void;
  const promise = new Promise<string | null>((done) => {
    resolve = done;
  });
  cache.set(id, promise);
  if (cache.size > maximumCached) cache.delete(cache.keys().next().value!);
  queue.push({ id, signal, promise, resolve });
  drain();
  return promise;
}

export function invalidateApplicationIcons(): void {
  cache.clear();
  for (const request of queue.splice(0)) request.resolve(null);
  generation.update((value) => value + 1);
}
