import { test, expect } from '@playwright/test';
import { chooseSearchEngine } from './helpers/pickers';
test('web mode follows engine settings and is independent of filters', async ({
  page,
}) => {
  await page.goto('http://127.0.0.1:1422');
  await page.getByRole('button', { name: 'Files', exact: true }).click();
  await page.getByRole('combobox').fill('? hello world');
  await expect(page.getByRole('option')).toHaveCount(1);
  await expect(page.getByRole('option')).toContainText('Search Google');
  await page
    .getByRole('button', { name: 'Back to search', exact: true })
    .click();
  await expect(
    page.getByRole('button', { name: 'Files', exact: true }),
  ).toHaveAttribute('aria-pressed', 'true');
  await page.getByRole('combobox').fill('? hello world');
  await expect(page.getByRole('option')).toContainText('Search Google');
  await page.getByRole('button', { name: 'Open Mycast settings' }).click();
  await page.getByRole('tab', { name: 'Search', exact: true }).click();
  await chooseSearchEngine(page, 'bing');
  await page
    .getByRole('button', { name: 'Back to commands', exact: true })
    .click();
  await page.getByRole('combobox').fill('? hello world');
  await expect(page.getByRole('option')).toContainText('Search Bing');
  await page.getByRole('combobox').fill('?');
  await expect(page.getByRole('option')).toHaveCount(0);
});
