import { test, expect } from '@playwright/test';
import { mockFileSearch } from './helpers/file-search';

test('file searches keep rows steady, show delayed progress in the footer, and ignore old responses', async ({
  page,
}) => {
  await mockFileSearch(page);
  await page.goto('http://127.0.0.1:1422');
  await page.getByRole('button', { name: 'Files', exact: true }).click();
  const row = page.getByRole('option').filter({ hasText: 'project-notes.txt' });
  await expect(row).toBeVisible();
  const search = page.getByRole('combobox');
  await search.fill('pr');
  await expect
    .poll(() => page.evaluate(() => (window as any).hasFileSearch('pr')))
    .toBe(true);
  await page.evaluate(() =>
    (window as any).completeFileSearch('pr', 'project-notes.txt'),
  );
  await expect(page.getByRole('listbox', { name: 'Commands' })).toHaveAttribute(
    'aria-busy',
    'false',
  );
  const before = await row.boundingBox();
  const footer = await page.locator('footer').boundingBox();
  const hints = await page.getByLabel('Keyboard hints').boundingBox();
  await search.fill('pro');
  await expect(page.locator('footer')).toContainText('Searching files…');
  await expect(row).toBeVisible();
  expect(await row.boundingBox()).toEqual(before);
  expect(await page.locator('footer').boundingBox()).toEqual(footer);
  expect(await page.getByLabel('Keyboard hints').boundingBox()).toEqual(hints);
  await expect(page.getByLabel('Keyboard hints')).toContainText('Navigate');
  await expect(page.locator('.file-search-status')).toHaveCount(0);

  await search.fill('proj');
  await expect
    .poll(() => page.evaluate(() => (window as any).hasFileSearch('proj')))
    .toBe(true);
  await page.evaluate(() =>
    (window as any).completeFileSearch('proj', 'project-new.txt'),
  );
  await expect(
    page.getByRole('option').filter({ hasText: 'project-new.txt' }),
  ).toBeVisible();
  await expect(page.locator('footer')).not.toContainText('Searching files…');
  await page.evaluate(() =>
    (window as any).completeFileSearch('pro', 'project-stale.txt'),
  );
  await expect(
    page.getByRole('option').filter({ hasText: 'project-stale.txt' }),
  ).toHaveCount(0);
  await expect(
    page.getByRole('option').filter({ hasText: 'project-new.txt' }),
  ).toBeVisible();
});
