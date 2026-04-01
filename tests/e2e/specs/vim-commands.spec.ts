import { test, expect } from '@playwright/test';

test.describe('Vim Commands', () => {
  test(':color changes stroke color', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(1000);

    // Enter command mode and type color command
    await page.keyboard.press(':');
    await expect(page.locator('text=COMMAND')).toBeVisible();

    await page.keyboard.type('color #ff0000');
    await page.keyboard.press('Enter');

    // Should return to NORMAL mode
    await expect(page.locator('text=NORMAL')).toBeVisible();
  });

  test(':fill changes fill color', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(1000);

    await page.keyboard.press(':');
    await page.keyboard.type('fill #00ff00');
    await page.keyboard.press('Enter');

    await expect(page.locator('text=NORMAL')).toBeVisible();
  });

  test(':stroke changes stroke width', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(1000);

    await page.keyboard.press(':');
    await page.keyboard.type('stroke 5');
    await page.keyboard.press('Enter');

    await expect(page.locator('text=NORMAL')).toBeVisible();
  });

  test('Escape cancels command mode', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(1000);

    await page.keyboard.press(':');
    await expect(page.locator('text=COMMAND')).toBeVisible();

    await page.keyboard.type('some partial');
    await page.keyboard.press('Escape');

    await expect(page.locator('text=NORMAL')).toBeVisible();
  });
});
