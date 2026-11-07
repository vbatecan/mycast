import { test, expect } from '@playwright/test';
import { mockAiConnection } from './helpers/ai-connection';
import { chooseProvider } from './helpers/pickers';

test('follow-ups retain context across leaving AI, and new conversations clear it', async ({
  page,
}) => {
  await page.goto('http://127.0.0.1:1422');
  await mockAiConnection(page);
  await page.getByRole('button', { name: 'Ask AI', exact: true }).click();
  const composer = page.getByRole('textbox', { name: 'Ask AI', exact: true });
  await composer.fill('Remember my plant is an orchid');
  await composer.press('Enter');
  await expect(
    page.getByRole('document', { name: 'AI response' }),
  ).toContainText('Keep the focus');
  await page
    .getByRole('button', { name: 'Back to search', exact: true })
    .click();
  await page.getByRole('button', { name: 'Ask AI', exact: true }).click();
  await composer.fill('Which plant?');
  await composer.press('Enter');
  await expect(page.locator('.past-turn')).toContainText(
    'Remember my plant is an orchid',
  );
  await page.screenshot({
    path: 'artifacts/ai-conversation.png',
    animations: 'disabled',
  });
  const requests = await page.evaluate(() => (window as any).aiRequests);
  expect(requests[1]).toEqual({
    prompt: 'Which plant?',
    history: [
      { role: 'user', content: 'Remember my plant is an orchid' },
      {
        role: 'assistant',
        content: '## A quieter interface\n\nKeep the focus on your question.',
      },
    ],
  });
  await page
    .getByRole('button', { name: 'New conversation', exact: true })
    .click();
  await expect(page.locator('.past-turn')).toHaveCount(0);
  await composer.fill('A fresh start');
  await composer.press('Enter');
  expect(
    (await page.evaluate(() => (window as any).aiRequests))[2].history,
  ).toEqual([]);
  await page.getByRole('button', { name: 'AI settings', exact: true }).click();
  await chooseProvider(page, 'claude');
  await page
    .getByRole('button', { name: 'Back to commands', exact: true })
    .click();
  await expect(
    page.getByRole('heading', { name: 'Ask anything' }),
  ).toBeVisible();
  await composer.fill('Hello Claude');
  await composer.press('Enter');
  expect(
    (await page.evaluate(() => (window as any).aiRequests))[3].history,
  ).toEqual([]);
});
