import { test, expect } from '@playwright/test';
import { chooseProvider } from './helpers/pickers';
test('AI prefix has an empty state and never opens web search', async ({
  page,
}) => {
  await page.goto('http://127.0.0.1:1422');
  await page.getByRole('combobox').fill('??');
  await expect(
    page.getByRole('heading', { name: 'Ask anything' }),
  ).toBeVisible();
  await expect(page.getByRole('option')).toHaveCount(0);
  await page
    .getByRole('textbox', { name: 'Ask AI', exact: true })
    .fill('explain streaming');
  await page
    .getByRole('textbox', { name: 'Ask AI', exact: true })
    .press('Enter');
  await expect(page.getByRole('alert')).toContainText('desktop app');
});
test('animation speed persists and reduced motion wins', async ({ page }) => {
  await page.goto('http://127.0.0.1:1422');
  await page.getByRole('button', { name: 'Open Mycast settings' }).click();
  await page.getByRole('tab', { name: 'Appearance', exact: true }).click();
  await page.getByLabel(/Animation speed/).fill('2');
  await expect(page.locator('html')).toHaveCSS('--motion-scale', '0.5');
  await page.reload();
  await expect(page.locator('html')).toHaveCSS('--motion-scale', '0.5');
  await page.emulateMedia({ reducedMotion: 'reduce' });
  await expect(
    page.getByRole('region', { name: 'Mycast command launcher' }),
  ).toHaveCSS('animation-name', 'none');
});
test('AI streams Markdown safely and stops on request', async ({ page }) => {
  await page.goto('http://127.0.0.1:1422');
  await page.getByRole('combobox').fill('?? test');
  await page.evaluate(() => {
    const callbacks = new Map<number, (value: unknown) => void>();
    let serial = 0;
    let finish: (() => void) | undefined;
    Object.assign(window, {
      isTauri: true,
      __TAURI_INTERNALS__: {
        transformCallback(callback: (value: unknown) => void) {
          callbacks.set(++serial, callback);
          return serial;
        },
        unregisterCallback(id: number) {
          callbacks.delete(id);
        },
        invoke(command: string, args: any) {
          if (command === 'ask_ai') {
            const callback = callbacks.get(args.output.id)!;
            callback({
              index: 0,
              message: { type: 'thinking', text: 'Model preview' },
            });
            callback({
              index: 1,
              message: {
                type: 'text',
                text: '# Hello\n\n**Streaming** answer. <img src="https://example.com/tracker"><script>alert(1)</script>',
              },
            });
            return new Promise<void>((resolve) => {
              finish = resolve;
            });
          }
          if (command === 'cancel_ai') {
            finish?.();
            return Promise.resolve();
          }
          return Promise.resolve();
        },
      },
    });
  });
  await page
    .getByRole('textbox', { name: 'Ask AI', exact: true })
    .press('Enter');
  await expect(
    page.getByRole('heading', { name: 'Hello', exact: true }),
  ).toBeVisible();
  await expect(page.getByText('Thinking', { exact: true })).toBeVisible();
  await expect(page.locator('.markdown strong')).toHaveText('Streaming');
  await page.screenshot({ path: 'artifacts/ai-answer.png' });
  await expect(page.locator('.markdown img, .markdown script')).toHaveCount(0);
  await page.getByRole('button', { name: 'Stop generating' }).click();
  await expect(page.locator('.ai-status')).toContainText('Stopped');
  await page
    .getByRole('button', { name: 'Back to search', exact: true })
    .click();
  await page.getByRole('combobox').fill('? web');
  await expect(page.getByRole('region', { name: 'AI answer' })).toHaveCount(0);
});

test('BYOK settings switch providers and keep keys out of browser storage', async ({
  page,
}) => {
  await page.goto('http://127.0.0.1:1422');
  await page.getByRole('button', { name: 'Open Mycast settings' }).click();
  await page.evaluate(() => {
    Object.assign(window, {
      isTauri: true,
      __TAURI_INTERNALS__: {
        invoke(command: string, args: any) {
          if (command === 'get_ai_config')
            return Promise.resolve({
              provider: args.provider,
              model: '',
              baseUrl: 'https://api.example.com/v1',
              hasKey: false,
            });
          if (command === 'save_ai_config')
            return Promise.resolve({ ...args.config, hasKey: !!args.apiKey });
          return Promise.resolve();
        },
      },
    });
  });
  await page.getByRole('tab', { name: 'AI', exact: true }).click();
  for (const provider of [
    'openai',
    'claude',
    'gemini',
    'custom',
    'ollama',
    'lmstudio',
  ]) {
    await chooseProvider(page, provider);
    await expect(page.getByLabel('Model', { exact: true })).toBeVisible();
  }
  await chooseProvider(page, 'custom');
  await page.getByLabel('Model', { exact: true }).fill('test-model');
  await page.getByLabel(/API key/).fill('synthetic-test-key');
  await page.getByRole('button', { name: 'Save changes' }).click();
  await expect(page.getByRole('status')).toHaveText('Saved');
  await expect(page.getByLabel(/API key/)).toHaveValue('');
  const storage = await page.evaluate(() =>
    JSON.stringify({ ...localStorage }),
  );
  expect(storage).not.toContain('synthetic-test-key');
});
