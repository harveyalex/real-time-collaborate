import { test, expect } from '@playwright/test';
import { getElementCount } from '../helpers';

test.describe('Keyboard Arrow/Line', () => {
  test('create arrow with keyboard: a, navigate to endpoint, Enter', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(2000);

    const before = await getElementCount(page);

    // Enter arrow mode
    await page.keyboard.press('a');
    await expect(page.locator('text=INSERT')).toBeVisible();

    // First Enter sets start point at vim cursor
    await page.keyboard.press('Enter');
    await page.waitForTimeout(100);

    // Move cursor to endpoint
    for (let i = 0; i < 5; i++) await page.keyboard.press('l');
    for (let i = 0; i < 3; i++) await page.keyboard.press('j');
    await page.waitForTimeout(100);

    // Second Enter sets end point and creates the arrow
    await page.keyboard.press('Enter');
    await page.waitForTimeout(500);

    const after = await getElementCount(page);
    expect(after).toBeGreaterThan(before);
  });
});
