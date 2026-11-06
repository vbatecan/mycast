import { test, expect } from '@playwright/test';

test.beforeEach(async ({ page }) => {
  await page.goto('http://127.0.0.1:1422');
});

test('search receives focus, arrows wrap and keep input focus', async ({
  page,
}) => {
  const search = page.getByRole('combobox');
  await expect(search).toBeFocused();
  await expect(page.getByRole('option', { selected: true })).toContainText(
    'Open Notepad',
  );
  await search.press('ArrowUp');
  await expect(page.getByRole('option', { selected: true })).toContainText(
    'About Application',
  );
  await expect(search).toBeFocused();
  await search.press('ArrowDown');
  await expect(page.getByRole('option', { selected: true })).toContainText(
    'Open Notepad',
  );
  await search.fill('downloads');
  await expect(page.getByRole('option', { selected: true })).toContainText(
    'Open Downloads',
  );
  await expect(search).toBeFocused();
});

test('calculator handles examples and division by zero', async ({ page }) => {
  const search = page.getByRole('combobox');
  for (const [expression, answer] of [
    ['2 + 2', '4'],
    ['120 * 0.15', '18'],
    ['(50 + 20) / 2', '35'],
  ]) {
    await search.fill(expression);
    await expect(page.getByRole('option')).toContainText(answer);
  }
  await page.screenshot({
    animations: 'disabled',
    path: 'artifacts/calculator.png',
  });
  await search.fill('5 / 0');
  await expect(
    page.getByRole('status').filter({ hasText: 'Check your calculation' }),
  ).toContainText('Cannot divide by zero');
});

test('native commands explicitly report desktop-only in browser', async ({
  page,
}) => {
  const search = page.getByRole('combobox');
  await search.fill('notepad');
  await search.press('Enter');
  await expect(page.getByRole('status')).toContainText(
    'Available in the desktop app',
  );
  await search.fill('2 + 2');
  await search.press('Enter');
  await expect(page.getByRole('status')).toContainText(
    'Available in the desktop app',
  );
});

test('no-results state and clearing query recover selection', async ({
  page,
}) => {
  await page.getByRole('combobox').fill('zzzzzzzzzzzzzzzzz');
  await expect(
    page.getByRole('heading', { name: 'No commands found' }),
  ).toBeVisible();
  await page.screenshot({
    animations: 'disabled',
    path: 'artifacts/empty.png',
  });
  await page.getByRole('button', { name: 'Clear search' }).click();
  await expect(page.getByRole('combobox')).toBeFocused();
  await expect(page.getByRole('option', { selected: true })).toContainText(
    'Open Notepad',
  );
});

test('settings tabs, toggles, persistence and Escape back work', async ({
  page,
}) => {
  await page.getByRole('button', { name: 'Open Mycast settings' }).click();
  await expect(
    page.getByRole('button', { name: 'Back to commands' }),
  ).toBeFocused();
  const descriptions = page.getByRole('switch', { name: 'Show descriptions' });
  await descriptions.click();
  await expect(descriptions).toHaveAttribute('aria-checked', 'false');
  await expect(
    page.getByRole('switch', { name: 'Launch at startup', exact: true }),
  ).toBeDisabled();
  await page.getByRole('tab', { name: 'General', exact: true }).focus();
  await page.keyboard.press('ArrowRight');
  await expect(
    page.getByRole('tab', { name: 'Search', exact: true }),
  ).toHaveAttribute('aria-selected', 'true');
  await page.keyboard.press('ArrowRight');
  await expect(
    page.getByRole('tab', { name: 'Commands', exact: true }),
  ).toHaveAttribute('aria-selected', 'true');
  await page.keyboard.press('ArrowRight');
  await expect(
    page.getByRole('tab', { name: 'Files & Run', exact: true }),
  ).toHaveAttribute('aria-selected', 'true');
  await page.keyboard.press('ArrowRight');
  await expect(
    page.getByRole('tab', { name: 'Appearance', exact: true }),
  ).toHaveAttribute('aria-selected', 'true');
  await page.getByRole('switch', { name: 'Animations', exact: true }).click();
  await page.screenshot({
    animations: 'disabled',
    path: 'artifacts/settings.png',
  });
  await page.keyboard.press('Escape');
  await expect(page.getByRole('combobox')).toBeFocused();
  await expect(page.getByText('A fresh page for your next idea')).toHaveCount(
    0,
  );
  await page.reload();
  await page.getByRole('button', { name: 'Open Mycast settings' }).click();
  await expect(
    page.getByRole('switch', { name: 'Show descriptions' }),
  ).toHaveAttribute('aria-checked', 'false');
});

test('pinning preserves selection and aliases survive reload', async ({
  page,
}) => {
  const search = page.getByRole('combobox');
  await search.press('ArrowDown');
  await search.press('Control+d');
  await expect(page.getByRole('option', { selected: true })).toContainText(
    'Open Calculator',
  );
  await page.getByRole('button', { name: 'Open Mycast settings' }).click();
  await page.getByRole('tab', { name: 'Commands', exact: true }).click();
  await page
    .getByRole('textbox', { name: 'Search saved commands' })
    .fill('Open Calculator');
  await page.getByRole('button', { name: 'Edit aliases', exact: true }).click();
  await page
    .getByRole('dialog')
    .getByRole('textbox')
    .fill('mathbox\nquickmath');
  await page.getByRole('button', { name: 'Save aliases' }).click();
  await page.reload();
  await page.getByRole('combobox').fill('QUICKMATH');
  await expect(page.getByRole('option', { selected: true })).toContainText(
    'Open Calculator',
  );
  await page.getByRole('combobox').fill('');
  await page.getByRole('button', { name: 'Favorites', exact: true }).click();
  await expect(page.getByRole('option')).toHaveCount(1);
  await expect(page.getByRole('option')).toContainText('Open Calculator');
});

test('history records successful intent and excludes failed native dispatch', async ({
  page,
}) => {
  const search = page.getByRole('combobox');
  await search.fill('notepad');
  await search.press('Enter');
  await expect(page.getByRole('status')).toContainText(
    'Available in the desktop app',
  );
  expect(
    await page.evaluate(
      () =>
        JSON.parse(localStorage.getItem('mycast.personalization.v1')!).recents,
    ),
  ).toEqual([]);
  await search.fill('about');
  await search.press('Enter');
  await expect(
    page.getByRole('tab', { name: 'About', exact: true }),
  ).toHaveAttribute('aria-selected', 'true');
  expect(
    await page.evaluate(
      () =>
        JSON.parse(localStorage.getItem('mycast.personalization.v1')!).recents,
    ),
  ).toEqual(['builtin:about']);
  await page.reload();
  await page.getByRole('button', { name: 'Recents', exact: true }).click();
  await expect(page.getByRole('option')).toHaveCount(1);
  await expect(page.getByRole('option')).toContainText('About Application');
});

test('footer keyboard activation opens settings without executing a command', async ({
  page,
}) => {
  await page.getByRole('button', { name: 'Open Mycast settings' }).focus();
  await page.keyboard.press('Enter');
  await expect(
    page.getByRole('tab', { name: 'General', exact: true }),
  ).toBeVisible();
  await expect(
    page.getByText('Available in the desktop app', { exact: false }),
  ).toHaveCount(0);
});

test('mouse hover selects, click executes and About is searchable', async ({
  page,
}) => {
  const search = page.getByRole('combobox');
  const calculator = page
    .getByRole('option')
    .filter({ hasText: 'Open Calculator' });
  await calculator.hover();
  await expect(calculator).toHaveAttribute('aria-selected', 'true');
  await calculator.click();
  await expect(page.getByRole('status')).toContainText(
    'Available in the desktop app',
  );
  await expect(search).toBeFocused();
  await search.press('ControlOrMeta+A');
  await search.pressSequentially('about');
  await search.press('Enter');
  await expect(
    page.getByRole('tab', { name: 'About', exact: true }),
  ).toHaveAttribute('aria-selected', 'true');
  await expect(page.getByText('Built with Tauri and Rust')).toBeVisible();
});

test('palette is compact, fixed-footer and makes no external requests', async ({
  page,
}) => {
  const external: string[] = [];
  page.on('request', (request) => {
    if (!request.url().startsWith('http://127.0.0.1:1422'))
      external.push(request.url());
  });
  await page.reload();
  await page.screenshot({
    animations: 'disabled',
    path: 'artifacts/launcher.png',
  });
  const palette = page.getByRole('region', { name: 'Mycast command launcher' });
  await expect(palette).toBeVisible();
  const box = await palette.boundingBox();
  expect(box?.width).toBe(720);
  expect(box?.height).toBe(480);
  const footer = page.locator('footer');
  const before = await footer.boundingBox();
  await page.getByRole('combobox').press('ArrowUp');
  const after = await footer.boundingBox();
  expect(after).toEqual(before);
  expect(external).toEqual([]);
});
