import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './e2e',
  fullyParallel: true,
  workers: 2,
  use: {
    browserName: 'chromium',
    channel: 'msedge',
    viewport: { width: 1000, height: 720 },
    trace: 'retain-on-failure',
    screenshot: { mode: 'only-on-failure' },
  },
  webServer: {
    command: 'npm run preview -- --host 127.0.0.1 --port 1422',
    url: 'http://127.0.0.1:1422',
    reuseExistingServer: !process.env.CI,
  },
});
