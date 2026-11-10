// Oracle ONLY. This suite does not run aiweb-native and cannot prove native browser compatibility.
import { test, expect } from '@playwright/test';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
const url = pathToFileURL(resolve(__dirname, '../../fixtures/forms.html')).href;

test('oracle: locator re-resolution + fill + submit', async ({ page }) => {
  await page.goto(url);
  await page.getByLabel('Name').fill('Agent');
  await page.getByRole('button', { name: 'Submit' }).click();
  await expect(page.locator('#result')).toHaveText('Agent');
});

test('oracle: form value changes without HTML attribute mutation', async ({ page }) => {
  await page.goto(url);
  const input = page.locator('#name');
  const oldAttr = await input.getAttribute('value');
  await input.fill('updated');
  expect(await input.inputValue()).toBe('updated');
  expect(await input.getAttribute('value')).toBe(oldAttr);
});
