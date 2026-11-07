import { test, expect } from '@playwright/test';
import { mockAiConnection } from './helpers/ai-connection';
import { chooseProvider, chooseSearchEngine } from './helpers/pickers';

test.use({ viewport: { width: 720, height: 480 } });

test('custom provider picker supports keyboard selection and Escape without leaving settings', async ({
  page,
}) => {
  await page.goto('http://127.0.0.1:1422');
  await page.getByRole('button', { name: 'Open Mycast settings' }).click();
  await mockAiConnection(page);
  await page.getByRole('tab', { name: 'AI', exact: true }).click();
  const picker = page.getByRole('combobox', { name: 'Provider', exact: true });
  await picker.click();
  await expect(
    page.getByRole('listbox', { name: 'Provider', exact: true }),
  ).toBeVisible();
  await page.screenshot({
    path: 'artifacts/ai-provider-picker.png',
    animations: 'disabled',
  });
  await page.keyboard.press('ArrowRight');
  await page.keyboard.press('Enter');
  await expect(picker).toContainText('Claude');
  await expect(picker).toBeFocused();
  await picker.press('ArrowDown');
  await page.keyboard.press('Escape');
  await expect(picker).toHaveAttribute('aria-expanded', 'false');
  await expect(
    page.getByRole('tab', { name: 'AI', exact: true }),
  ).toHaveAttribute('aria-selected', 'true');
});

test('bottom composer supports multiline prompts and reduced motion', async ({
  page,
}) => {
  await page.emulateMedia({ reducedMotion: 'reduce' });
  await page.goto('http://127.0.0.1:1422');
  await mockAiConnection(page);
  await page.getByRole('button', { name: 'Ask AI', exact: true }).click();
  const composer = page.getByRole('textbox', { name: 'Ask AI', exact: true });
  await expect(page.locator('.search-slot')).toHaveCSS(
    'transition-duration',
    '0s',
  );
  await composer.fill('First line');
  await composer.press('Shift+Enter');
  await composer.pressSequentially('Second line');
  await expect(composer).toHaveValue('First line\nSecond line');
  expect(await page.evaluate(() => (window as any).aiSubmitted)).toEqual([]);
  await composer.press('Enter');
  await expect(composer).toHaveValue('');
  expect(await page.evaluate(() => (window as any).aiSubmitted)).toEqual([
    'First line\nSecond line',
  ]);
  await expect(page.locator('footer')).toHaveCount(0);
});

test('AI has one contextual prompt action and keyboard prefixes still work', async ({
  page,
}) => {
  await page.goto('http://127.0.0.1:1422');
  await mockAiConnection(page);
  await page.getByRole('combobox').pressSequentially('??');
  const input = page.getByRole('textbox', { name: 'Ask AI', exact: true });
  await expect(input).toHaveValue('');
  await expect(input).toHaveAttribute('placeholder', 'Ask a question…');
  await expect(
    page.getByRole('toolbar', { name: 'Search filters' }),
  ).toHaveCount(0);
  await expect(page.locator('footer')).toHaveCount(0);
  await expect
    .poll(
      async () => (await page.locator('.search-slot').boundingBox())?.y ?? 0,
    )
    .toBeGreaterThan(370);
  await expect(input).toBeFocused();
  await expect(
    page.getByRole('button', { name: 'Clear', exact: true }),
  ).toHaveCount(0);
  await expect(
    page.getByRole('button', { name: 'Clear search', exact: true }),
  ).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'Send prompt' })).toHaveCount(
    0,
  );
  await expect(
    page.getByRole('heading', { name: 'Ask anything' }),
  ).toBeVisible();
  await page.screenshot({
    path: 'artifacts/ai-redesign-empty.png',
    animations: 'disabled',
  });
  await input.fill('Explain a quieter interface');
  await expect(page.getByRole('button', { name: 'Send prompt' })).toHaveCount(
    1,
  );
  expect(await page.evaluate(() => (window as any).aiSubmitted)).toEqual([]);
  await page.getByRole('button', { name: 'Send prompt' }).click();
  await expect(
    page.getByRole('document', { name: 'AI response' }),
  ).toContainText('Keep the focus');
  expect(await page.evaluate(() => (window as any).aiSubmitted)).toEqual([
    'Explain a quieter interface',
  ]);
  await expect(
    page.getByRole('button', { name: 'Copy response' }),
  ).toBeVisible();
  await page.screenshot({
    path: 'artifacts/ai-redesign-answer.png',
    animations: 'disabled',
  });
  await page.getByRole('button', { name: 'New conversation' }).click();
  await expect(input).toHaveValue('');
  await expect(input).toBeFocused();
  await input.press('Backspace');
  await expect(
    page.getByRole('toolbar', { name: 'Search filters' }),
  ).toBeVisible();
  await expect(
    page.getByRole('button', { name: 'Ask AI', exact: true }),
  ).toBeVisible();
});

test('AI settings disclose relevant controls and keep Save visible in a compact window', async ({
  page,
}) => {
  await page.goto('http://127.0.0.1:1422');
  await page.getByRole('button', { name: 'Open Mycast settings' }).click();
  await mockAiConnection(page);
  await page.getByRole('tab', { name: 'AI', exact: true }).click();
  await expect(page.getByLabel('Model', { exact: true })).toHaveValue(
    'design-model',
  );
  await expect(page.getByLabel('Base URL', { exact: true })).toHaveCount(0);
  await expect(
    page.getByRole('button', { name: 'Save changes' }),
  ).toBeDisabled();
  await page.screenshot({
    path: 'artifacts/ai-redesign-settings-cloud.png',
    animations: 'disabled',
  });
  await chooseProvider(page, 'custom');
  const url = page.getByLabel('Base URL', { exact: true });
  await expect(url).toHaveValue('https://api.example.com/v1');
  await expect(page.getByLabel('Remove saved key')).toBeVisible();
  await url.fill('https://other.example.com/v1');
  await expect(page.getByLabel('Remove saved key')).toHaveCount(0);
  await expect(
    page.getByRole('button', { name: 'Save changes' }),
  ).toBeInViewport({ ratio: 1 });
  await page.getByRole('button', { name: 'Save changes' }).click();
  await expect(page.getByRole('status')).toHaveText('Saved');
  await page.screenshot({
    path: 'artifacts/ai-redesign-settings-custom.png',
    animations: 'disabled',
  });
  await chooseProvider(page, 'ollama');
  await expect(page.getByLabel('Base URL', { exact: true })).toBeVisible();
  await expect(page.getByLabel('API key', { exact: true })).not.toBeVisible();
  await page.locator('.optional-auth summary').click();
  await expect(page.getByLabel('API key', { exact: true })).toBeVisible();
});

test('unconfigured AI opens its settings directly and preserves the draft', async ({
  page,
}) => {
  await page.goto('http://127.0.0.1:1422');
  await mockAiConnection(page, false);
  await page.getByRole('button', { name: 'Ask AI', exact: true }).click();
  await expect(
    page.getByRole('heading', { name: 'Connect an AI provider' }),
  ).toBeVisible();
  await page
    .getByRole('textbox', { name: 'Ask AI', exact: true })
    .fill('Keep this draft');
  await page.getByRole('button', { name: 'Set up AI' }).click();
  await expect(
    page.getByRole('tab', { name: 'AI', exact: true }),
  ).toHaveAttribute('aria-selected', 'true');
  await page.getByLabel('Model', { exact: true }).fill('configured-model');
  await page.getByRole('button', { name: 'Save changes' }).click();
  await page
    .getByRole('button', { name: 'Back to commands', exact: true })
    .click();
  await expect(
    page.getByRole('textbox', { name: 'Ask AI', exact: true }),
  ).toHaveValue('Keep this draft');
  await expect(
    page.getByRole('heading', { name: 'Ask anything' }),
  ).toBeVisible();
});

test('web settings validate a custom URL and the web field keeps query syntax out of the way', async ({
  page,
}) => {
  await page.goto('http://127.0.0.1:1422');
  await page.getByRole('button', { name: 'Open Mycast settings' }).click();
  await page.getByRole('tab', { name: 'Search', exact: true }).click();
  await expect(page.getByRole('button', { name: 'Save changes' })).toHaveCount(
    0,
  );
  await chooseSearchEngine(page, 'custom');
  const url = page.getByLabel('Search URL', { exact: true });
  await url.fill('javascript:alert(1)');
  await page.getByRole('button', { name: 'Save changes' }).click();
  await expect(page.getByRole('alert')).toContainText('{query}');
  await url.fill('https://example.com/find?q={query}');
  await page.getByRole('button', { name: 'Save changes' }).click();
  await expect(page.getByRole('status')).toHaveText('Saved');
  await expect(
    page.getByRole('button', { name: 'Save changes' }),
  ).toBeInViewport({ ratio: 1 });
  await page.evaluate(
    () =>
      new Promise<void>((resolve) =>
        requestAnimationFrame(() => requestAnimationFrame(() => resolve())),
      ),
  );
  await page.screenshot({
    path: 'artifacts/search-redesign-settings.png',
    animations: 'disabled',
  });
  await page
    .getByRole('button', { name: 'Back to commands', exact: true })
    .click();
  await page.getByRole('combobox').fill('? interaction design');
  await expect(page.getByRole('combobox')).toHaveValue('interaction design');
  await expect(page.getByRole('option')).toContainText('Search Custom search');
  await expect(page.locator('.file-search-status')).toHaveCount(0);
  await page
    .getByRole('button', { name: 'Back to search', exact: true })
    .click();
  await expect(
    page.getByRole('toolbar', { name: 'Search filters' }),
  ).toBeVisible();
});
