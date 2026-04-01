import { test, expect } from '@playwright/test';
import { getElementCount, drawRectangle } from '../helpers';

test.describe('Keyboard Selection', () => {
  test('Tab cycles through elements to select them', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(2000);

    // Draw two rectangles
    await drawRectangle(page, 50, 50, 100, 80);
    await page.waitForTimeout(300);
    await drawRectangle(page, 250, 50, 100, 80);
    await page.waitForTimeout(300);

    await page.keyboard.press('Escape');
    await page.waitForTimeout(200);

    // Tab to select first element
    await page.keyboard.press('Tab');
    await page.waitForTimeout(200);

    const sel1 = await page.evaluate(() => (window as any).__TEST_SELECTED_IDS);
    expect(sel1).not.toBe('[]');

    // Tab again to select next element
    await page.keyboard.press('Tab');
    await page.waitForTimeout(200);

    const sel2 = await page.evaluate(() => (window as any).__TEST_SELECTED_IDS);
    expect(sel2).not.toBe('[]');
    // Should be a different element
    expect(sel2).not.toBe(sel1);
  });
});
