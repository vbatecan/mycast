import type { Page } from '@playwright/test';

export async function chooseProvider(page: Page, value: string) {
  const labels: Record<string, string> = {
    openai: 'OpenAI',
    claude: 'Claude',
    gemini: 'Gemini',
    custom: 'Custom endpoint',
    ollama: 'Ollama',
    lmstudio: 'LM Studio',
  };
  await page.getByRole('combobox', { name: 'Provider', exact: true }).click();
  await page.getByRole('option', { name: labels[value], exact: true }).click();
}

export async function chooseSearchEngine(page: Page, value: string) {
  const labels: Record<string, string> = {
    google: 'Google',
    bing: 'Bing',
    duckduckgo: 'DuckDuckGo',
    custom: 'Custom',
  };
  await page
    .getByRole('combobox', { name: 'Search engine', exact: true })
    .click();
  await page.getByRole('option', { name: labels[value], exact: true }).click();
}
