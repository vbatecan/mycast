import { test, expect } from '@playwright/test';

test('recent programs and folders come first and their limit persists', async ({
  page,
}) => {
  await page.goto('http://127.0.0.1:1422');
  await page.evaluate(() =>
    localStorage.setItem(
      'mycast.personalization.v1',
      JSON.stringify({
        version: 1,
        favorites: ['builtin:explorer', 'builtin:downloads'],
        recents: [
          'builtin:about',
          'builtin:downloads',
          'builtin:documents',
          'builtin:desktop',
          'builtin:notepad',
          'builtin:windows-calculator',
          'builtin:explorer',
        ],
        aliases: {},
        usage: Object.fromEntries(
          [
            'about',
            'downloads',
            'documents',
            'desktop',
            'notepad',
            'windows-calculator',
            'explorer',
          ].map((id) => [`builtin:${id}`, { count: 1, lastUsed: Date.now() }]),
        ),
        items: {},
      }),
    ),
  );
  await page.reload();
  await expect(page.getByRole('option').first()).toContainText(
    'Open Downloads',
  );
  await expect(page.getByRole('option').nth(1)).toContainText('Open Documents');
  await page.getByRole('button', { name: 'Recents', exact: true }).click();
  await expect(page.getByRole('option')).toHaveCount(5);
  await page.getByRole('button', { name: 'Open Mycast settings' }).click();
  const limit = page.getByRole('spinbutton', { name: 'Recent items' });
  await expect(limit).toHaveValue('5');
  await limit.fill('3');
  await limit.press('Tab');
  await page
    .getByRole('button', { name: 'Back to commands', exact: true })
    .click();
  await expect(page.getByRole('option')).toHaveCount(3);
  await page.reload();
  await page.getByRole('button', { name: 'Recents', exact: true }).click();
  await expect(page.getByRole('option')).toHaveCount(3);
  await page.getByRole('button', { name: 'All', exact: true }).click();
  await expect(page.getByRole('option').first()).toContainText(
    'Open Downloads',
  );
  await page
    .getByRole('region', { name: 'Mycast command launcher' })
    .screenshot({ path: 'artifacts/recent-first.png', animations: 'disabled' });
});
