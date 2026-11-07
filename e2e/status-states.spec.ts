import { test, expect } from '@playwright/test';
import { mockFileSearch, waitForFileSearch } from './helpers/file-search';

test('empty collections explain themselves and Search All preserves the query', async ({
  page,
}) => {
  await page.goto('http://127.0.0.1:1422');
  await page.getByRole('button', { name: 'Favorites', exact: true }).click();
  await expect(
    page.getByRole('heading', { name: 'Your favorites go here' }),
  ).toBeVisible();
  await expect(page.getByText('Pin selected result')).toBeVisible();
  await page.getByRole('button', { name: 'Recents', exact: true }).click();
  await expect(
    page.getByRole('heading', { name: 'Your recent items go here' }),
  ).toBeVisible();
  await page.getByRole('combobox').fill('zzzz-no-match');
  await expect(
    page.getByRole('heading', { name: 'No matching recent items' }),
  ).toBeVisible();
  await page.getByRole('button', { name: 'Search All', exact: true }).click();
  await expect(
    page.getByRole('button', { name: 'All', exact: true }),
  ).toHaveAttribute('aria-pressed', 'true');
  await expect(page.getByRole('combobox')).toHaveValue('zzzz-no-match');
  await expect(page.getByRole('combobox')).toBeFocused();
  await expect(
    page.getByRole('heading', { name: 'No matches for “zzzz-no-match”' }),
  ).toBeVisible();
  await page.screenshot({
    path: 'artifacts/empty-search.png',
    animations: 'disabled',
  });
});

test('folder setup opens the correct settings and counts appear only there', async ({
  page,
}) => {
  await mockFileSearch(page, { roots: [], initialItems: [], indexed: 123456 });
  await page.goto('http://127.0.0.1:1422');
  await page.getByRole('button', { name: 'Files', exact: true }).click();
  await expect(
    page.getByRole('heading', { name: 'Choose where to search' }),
  ).toBeVisible();
  await expect(page.locator('body')).not.toContainText('123,456');
  await page
    .getByRole('button', { name: 'Manage folders', exact: true })
    .click();
  await expect(
    page.getByRole('tab', { name: 'Files & Run', exact: true }),
  ).toHaveAttribute('aria-selected', 'true');
  await expect(
    page.getByRole('button', { name: 'Add folder', exact: true }),
  ).toBeVisible();
  await expect(page.locator('.index-status')).toContainText(
    '123,456 files and folders indexed',
  );
});

test('indexing stays quiet about counts and keeps keyboard hints visible', async ({
  page,
}) => {
  await mockFileSearch(page, {
    state: 'indexing',
    indexed: 123456,
    initialItems: [],
  });
  await page.goto('http://127.0.0.1:1422');
  await page.getByRole('button', { name: 'Files', exact: true }).click();
  await expect(
    page.getByRole('heading', { name: 'Preparing file search' }),
  ).toBeVisible();
  await expect(page.locator('footer')).toContainText('Preparing file search…');
  await expect(page.getByLabel('Keyboard hints')).toContainText('Navigate');
  await expect(page.getByLabel('Keyboard hints')).toContainText('Open');
  await expect(page.locator('body')).not.toContainText('123,456');
  await expect(page.locator('.file-search-status')).toHaveCount(0);
  await page.screenshot({
    path: 'artifacts/indexing-state.png',
    animations: 'disabled',
  });
  await page.evaluate(() =>
    (window as any).setFileStatus({ state: 'ready', revision: 2 }),
  );
  await expect(
    page.getByRole('heading', { name: 'No files to show' }),
  ).toBeVisible();
});

test('pending searches do not claim no matches; failures offer details and a working retry', async ({
  page,
}) => {
  await mockFileSearch(page, { initialItems: [] });
  await page.goto('http://127.0.0.1:1422');
  await page.getByRole('button', { name: 'Files', exact: true }).click();
  const search = page.getByRole('combobox');
  await search.fill('missing');
  await waitForFileSearch(page, 'missing');
  await expect(page.locator('footer')).toContainText('Searching files…');
  await expect(
    page.getByRole('heading', { name: 'No matching files' }),
  ).toHaveCount(0);
  await page.evaluate(() => (window as any).failFileSearch('missing'));
  await expect(
    page.getByRole('heading', { name: 'File search couldn’t finish' }),
  ).toBeVisible();
  await expect(page.locator('body')).not.toContainText('E_TEST_FAILURE');
  await page
    .getByRole('button', { name: 'Details: File search unavailable' })
    .click();
  const details = page.getByRole('region', { name: 'Status details' });
  await details.getByText('Technical details', { exact: true }).click();
  await expect(details).toContainText('E_TEST_FAILURE');
  await page.screenshot({
    path: 'artifacts/search-error-details.png',
    animations: 'disabled',
  });
  await page.keyboard.press('Escape');
  await expect(details).toHaveCount(0);
  await expect(search).toBeVisible();
  await page.getByRole('button', { name: 'Retry', exact: true }).click();
  await waitForFileSearch(page, 'missing');
  await page.evaluate(() => (window as any).completeFileSearch('missing', []));
  await expect(
    page.getByRole('heading', { name: 'No matching files' }),
  ).toBeVisible();
  await expect(page.locator('footer')).not.toContainText(
    'File search unavailable',
  );
});

test('setup errors can be retried without accumulating file listeners', async ({
  page,
}) => {
  await mockFileSearch(page, { setupFailure: true });
  await page.goto('http://127.0.0.1:1422');
  await page.getByRole('button', { name: 'Files', exact: true }).click();
  await expect(
    page.getByRole('heading', { name: 'File search couldn’t finish' }),
  ).toBeVisible();
  await expect
    .poll(() => page.evaluate(() => (window as any).fileListenerCount()))
    .toBe(0);
  await page.evaluate(() => (window as any).recoverFileSetup());
  await page.getByRole('button', { name: 'Retry', exact: true }).click();
  await expect(
    page.getByRole('option').filter({ hasText: 'project-notes.txt' }),
  ).toBeVisible();
  await expect
    .poll(() => page.evaluate(() => (window as any).fileListenerCount()))
    .toBe(1);
});

test('application refresh errors offer recovery without exposing native errors', async ({
  page,
}) => {
  await mockFileSearch(page, {
    applications: true,
    applicationFailure: true,
    initialItems: [],
  });
  await page.goto('http://127.0.0.1:1422');
  await page.getByRole('button', { name: 'Applications', exact: true }).click();
  await page.getByRole('combobox').fill('missingapp');
  await expect(
    page.getByRole('heading', { name: 'Applications couldn’t refresh' }),
  ).toBeVisible();
  await expect(page.locator('body')).not.toContainText('E_TEST_APPS');
  await page.evaluate(() => (window as any).recoverApplications());
  await page.getByRole('button', { name: 'Retry', exact: true }).click();
  await expect(
    page.getByRole('heading', { name: 'No matching applications' }),
  ).toBeVisible();
});
