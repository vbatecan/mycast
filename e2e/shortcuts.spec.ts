import { test, expect } from '@playwright/test';

test('filter shortcuts, recording, conflicts, persistence, and reset', async ({
  page,
}) => {
  await page.goto('http://127.0.0.1:1422');
  await expect(page.getByText('Local search', { exact: true })).toHaveCount(0);
  await expect(page.locator('.catalogue-info')).toHaveCount(0);
  const scopes = [
    'All',
    'Applications',
    'Commands',
    'Files',
    'Favorites',
    'Recents',
  ];
  for (let index = 0; index < scopes.length; index++) {
    await page.getByRole('combobox').press(`Alt+Shift+Digit${index + 1}`);
    await expect(
      page.getByRole('button', { name: scopes[index], exact: true }),
    ).toHaveAttribute('aria-pressed', 'true');
    await expect(page.getByRole('combobox')).toBeFocused();
  }
  await page.getByRole('button', { name: 'Open Mycast settings' }).click();
  await page.getByRole('tab', { name: 'Shortcuts', exact: true }).click();
  const all = page.getByRole('button', {
    name: 'Change shortcut for All',
    exact: true,
  });
  await all.click();
  await all.press('Alt+Shift+2');
  await expect(page.getByRole('alert')).toContainText(
    'Already used for Applications',
  );
  await all.press('Control+Shift+Q');
  await expect(all).toHaveText('Ctrl+Shift+Q');
  await page
    .getByRole('button', { name: 'Change shortcut for Files', exact: true })
    .click();
  await page.keyboard.press('Escape');
  await expect(
    page.getByRole('tab', { name: 'Shortcuts', exact: true }),
  ).toHaveAttribute('aria-selected', 'true');
  await page
    .getByRole('region', { name: 'Mycast command launcher' })
    .screenshot({
      path: 'artifacts/shortcut-settings.png',
      animations: 'disabled',
    });
  await page
    .getByRole('button', { name: 'Back to commands', exact: true })
    .click();
  await page.getByRole('combobox').press('Control+Shift+Q');
  await expect(
    page.getByRole('button', { name: 'All', exact: true }),
  ).toHaveAttribute('aria-pressed', 'true');
  await page.reload();
  await page.getByRole('combobox').press('Alt+Shift+4');
  await page.getByRole('combobox').press('Alt+Shift+1');
  await expect(
    page.getByRole('button', { name: 'Files', exact: true }),
  ).toHaveAttribute('aria-pressed', 'true');
  await page.getByRole('combobox').press('Control+Shift+Q');
  await expect(
    page.getByRole('button', { name: 'All', exact: true }),
  ).toHaveAttribute('aria-pressed', 'true');
  await page.getByRole('button', { name: 'Open Mycast settings' }).click();
  await page.getByRole('tab', { name: 'Shortcuts', exact: true }).click();
  await page
    .getByRole('button', { name: 'Reset controls', exact: true })
    .click();
  await expect(all).toHaveText('Alt+Shift+1');
});

test('navigation and open/back controls can be changed', async ({ page }) => {
  await page.goto('http://127.0.0.1:1422');
  await page.getByRole('button', { name: 'Open Mycast settings' }).click();
  await page.getByRole('tab', { name: 'Shortcuts', exact: true }).click();
  for (const [label, keys] of [
    ['Next result', 'Control+Alt+J'],
    ['Open selected item', 'Control+Alt+O'],
    ['Close / Back', 'Control+Alt+B'],
  ]) {
    const recorder = page.getByRole('button', {
      name: 'Change shortcut for ' + label,
      exact: true,
    });
    await recorder.click();
    await recorder.press(keys);
  }
  await page
    .getByRole('button', { name: 'Back to commands', exact: true })
    .click();
  const search = page.getByRole('combobox');
  await search.press('Control+Alt+J');
  await expect(page.getByRole('option', { selected: true })).toContainText(
    'Open Calculator',
  );
  await search.press('Alt+Shift+ArrowRight');
  await expect(
    page.getByRole('button', { name: 'Applications', exact: true }),
  ).toHaveAttribute('aria-pressed', 'true');
  await search.press('Alt+Shift+ArrowLeft');
  await expect(
    page.getByRole('button', { name: 'All', exact: true }),
  ).toHaveAttribute('aria-pressed', 'true');
  await search.fill('about');
  await search.press('Enter');
  await expect(search).toBeVisible();
  await search.press('Control+Alt+O');
  await expect(
    page.getByRole('tab', { name: 'About', exact: true }),
  ).toHaveAttribute('aria-selected', 'true');
  await page.keyboard.press('Control+Alt+B');
  await expect(search).toBeFocused();
});
