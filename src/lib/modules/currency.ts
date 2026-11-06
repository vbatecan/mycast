export async function convert(
  amount: string,
  base: string,
  quote: string,
  signal: AbortSignal,
): Promise<{ formatted: string; description: string }> {
  if (!Number.isFinite(Number(amount)))
    throw new Error('Amount is outside the supported range.');
  if (base === quote)
    return {
      formatted: `${Number(amount)} ${quote}`,
      description: 'Same currency · no conversion needed',
    };
  const key = `mycast.rate.${base}.${quote}`;
  let cached: { rate: number; date: string; fetched: number } | null = null;
  try {
    cached = JSON.parse(localStorage.getItem(key) ?? 'null');
  } catch {}
  if (
    cached &&
    (!(cached.rate > 0) ||
      !Number.isFinite(cached.rate) ||
      !Number.isFinite(cached.fetched) ||
      !/^\d{4}-\d{2}-\d{2}$/.test(cached.date))
  )
    cached = null;
  let offline = false;
  const request = new AbortController();
  const abort = () => request.abort();
  if (signal.aborted) throw new Error('Conversion cancelled.');
  signal.addEventListener('abort', abort, { once: true });
  const timeout = setTimeout(abort, 8000);
  try {
    if (!cached || Date.now() - cached.fetched > 6 * 60 * 60 * 1000) {
      try {
        const response = await fetch(
          `https://api.frankfurter.dev/v2/rate/${base}/${quote}`,
          { signal: request.signal },
        );
        if (!response.ok)
          throw new Error(`No exchange rate available for ${base}/${quote}.`);
        const data = await response.json();
        if (
          !(data.rate > 0) ||
          !Number.isFinite(data.rate) ||
          !/^\d{4}-\d{2}-\d{2}$/.test(data.date)
        )
          throw new Error('Invalid exchange rate response.');
        cached = { rate: data.rate, date: data.date, fetched: Date.now() };
        try {
          localStorage.setItem(key, JSON.stringify(cached));
        } catch {}
      } catch (error) {
        if (signal.aborted) throw error;
        if (!cached) throw error;
        offline = true;
      }
    }
    if (!cached)
      throw new Error(
        'No cached exchange rate available. Connect to the internet to refresh.',
      );
    const value = Number(amount) * cached.rate;
    if (!Number.isFinite(value))
      throw new Error('Amount is outside the supported range.');
    return {
      formatted: `${new Intl.NumberFormat(undefined, { maximumSignificantDigits: 12 }).format(value)} ${quote}`,
      description: `Frankfurter reference rate · ${cached.date}${offline ? ' · offline cache' : ' · cached locally'} · not a live quote`,
    };
  } finally {
    clearTimeout(timeout);
    signal.removeEventListener('abort', abort);
  }
}
