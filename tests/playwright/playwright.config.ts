import { defineConfig } from '@playwright/test';
export default defineConfig({
  testDir: './specs', workers: 2, retries: 0, timeout: 20_000,
  reporter: [['list'], ['json', { outputFile: '../results/chromium-oracle.json' }]],
  use: { browserName: 'chromium', headless: true, trace: 'retain-on-failure' },
});
