import { test, expect } from '@playwright/test';

test.beforeEach(async ({ page }) => {
  await page.goto('http://127.0.0.1:1422');
  await page.getByRole('button', { name: 'Open Mycast settings' }).click();
});

test('Commands keeps its search and footer visible while the list scrolls', async ({
  page,
}) => {
  await page.getByRole('tab', { name: 'Commands', exact: true }).click();
  const search = page.getByRole('textbox', { name: 'Search saved commands' });
  const list = page.getByRole('region', { name: 'Commands', exact: true });
  const before = await search.boundingBox();
  const footer = await page.locator('footer').boundingBox();
  await list.evaluate((element) => {
    element.scrollTop = element.scrollHeight;
  });
  expect(await list.evaluate((element) => element.scrollTop)).toBeGreaterThan(
    0,
  );
  expect(await search.boundingBox()).toEqual(before);
  expect(await page.locator('footer').boundingBox()).toEqual(footer);
  await search.fill('notepad');
  await expect(list.locator('article')).toHaveCount(1);
  await expect(list.locator('article')).toContainText('Open Notepad');
  await list.getByRole('button', { name: 'Pin', exact: true }).click();
  await page
    .getByRole('checkbox', { name: 'Favorites and aliases only' })
    .check();
  await expect(list.locator('article')).toHaveCount(1);
  await list.getByRole('button', { name: 'Edit aliases' }).click();
  await page.getByRole('dialog').getByRole('textbox').fill('scratchpad');
  await page.getByRole('button', { name: 'Save aliases' }).click();
  await search.fill('scratchpad');
  await expect(list.locator('article')).toContainText('scratchpad');
  await search.fill('');
  await page
    .getByRole('checkbox', { name: 'Favorites and aliases only' })
    .uncheck();
  await list.evaluate((element) => {
    element.scrollTop = 0;
  });
  await page
    .getByRole('region', { name: 'Mycast command launcher' })
    .screenshot({
      path: 'artifacts/commands-settings.png',
      animations: 'disabled',
    });
});

test('materials preview immediately and opacity and finishes survive reload', async ({
  page,
}) => {
  await page.getByRole('tab', { name: 'Appearance', exact: true }).click();
  const materials = page.getByRole('group', { name: 'Window material' });
  for (const name of ['Solid', 'Transparent', 'Blur', 'Mica', 'Acrylic']) {
    await materials.getByRole('button', { name, exact: true }).click();
    await expect(page.locator('html')).toHaveAttribute(
      'data-material',
      name.toLowerCase(),
    );
  }
  const opacity = page.getByRole('slider', { name: 'Background opacity' });
  await opacity.focus();
  await opacity.press('Home');
  await opacity.press('ArrowRight');
  await expect(opacity).toHaveValue('41');
  await page.getByRole('checkbox', { name: 'Soft accent glow' }).check();
  await page.getByRole('checkbox', { name: 'Highlight border' }).uncheck();
  await page.reload();
  await page.getByRole('button', { name: 'Open Mycast settings' }).click();
  await page.getByRole('tab', { name: 'Appearance', exact: true }).click();
  await expect(
    materials.getByRole('button', { name: 'Acrylic', exact: true }),
  ).toHaveAttribute('aria-pressed', 'true');
  await expect(opacity).toHaveValue('41');
  await expect(
    page.getByRole('checkbox', { name: 'Soft accent glow' }),
  ).toBeChecked();
  await expect(
    page.getByRole('checkbox', { name: 'Highlight border' }),
  ).not.toBeChecked();
  await materials.getByRole('button', { name: 'Solid', exact: true }).click();
  await expect(opacity).toBeDisabled();
  await materials.getByRole('button', { name: 'Acrylic', exact: true }).click();
  await opacity.focus();
  await opacity.press('End');
  await opacity.press('ArrowLeft');
  await page
    .getByRole('region', { name: 'Mycast command launcher' })
    .screenshot({
      path: 'artifacts/appearance-effects.png',
      animations: 'disabled',
    });
});

test('old preferences migrate and malformed material settings recover', async ({
  page,
}) => {
  for (const stored of [
    { animations: false, density: 'compact', textSize: 125 },
    {
      material: 'bad-value',
      opacity: 'invalid',
      glow: 1,
      highlightBorder: null,
    },
  ]) {
    await page.evaluate(
      (value) =>
        localStorage.setItem('mycast.preferences.v1', JSON.stringify(value)),
      stored,
    );
    await page.reload();
    await page.getByRole('button', { name: 'Open Mycast settings' }).click();
    await page.getByRole('tab', { name: 'Appearance', exact: true }).click();
    await expect(
      page.getByRole('slider', { name: 'Background opacity' }),
    ).toHaveValue('78');
    await expect(
      page
        .getByRole('group', { name: 'Window material' })
        .getByRole('button', { name: 'Acrylic' }),
    ).toHaveAttribute('aria-pressed', 'true');
    expect(
      await page
        .locator('.settings-content')
        .evaluate((element) => element.scrollWidth <= element.clientWidth),
    ).toBe(true);
  }
});
