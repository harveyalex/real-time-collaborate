import { test, expect } from '@playwright/test';

test.describe('Vim Mode Switching', () => {
  test('pressing r enters INSERT mode', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(1000);
    await expect(page.locator('text=NORMAL')).toBeVisible();

    await page.keyboard.press('r');
    await expect(page.locator('text=INSERT')).toBeVisible();
  });

  test('pressing Escape returns to NORMAL', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(1000);
    await page.keyboard.press('r');
    await expect(page.locator('text=INSERT')).toBeVisible();

    await page.keyboard.press('Escape');
    await expect(page.locator('text=NORMAL')).toBeVisible();
  });

  test('pressing v enters VISUAL mode', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(1000);
    await page.keyboard.press('v');
    await expect(page.locator('text=VISUAL')).toBeVisible();
  });

  test('pressing : enters COMMAND mode', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(1000);
    await page.keyboard.press(':');
    await expect(page.locator('text=COMMAND')).toBeVisible();
  });

  test('each shape key enters INSERT mode', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(1000);

    for (const key of ['r', 'e', 'a', 't', 'f']) {
      await page.keyboard.press(key);
      await expect(page.locator('text=INSERT')).toBeVisible();
      await page.keyboard.press('Escape');
      await expect(page.locator('text=NORMAL')).toBeVisible();
    }
  });
});
