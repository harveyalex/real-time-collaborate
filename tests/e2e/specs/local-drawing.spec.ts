import { test, expect } from '@playwright/test';
import { getElementCount, drawRectangle } from '../helpers';

test.describe('Local Drawing', () => {
  test('drawing a rectangle increases element count', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(2000);

    const before = await getElementCount(page);
    await drawRectangle(page, 100, 100, 200, 150);
    await page.waitForTimeout(500);
    const after = await getElementCount(page);

    expect(after).toBeGreaterThan(before);
  });

  test('drawing multiple shapes increases count each time', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(2000);

    const before = await getElementCount(page);

    // Draw 3 rectangles
    for (let i = 0; i < 3; i++) {
      await drawRectangle(page, 50 + i * 100, 50, 80, 60);
      await page.waitForTimeout(300);
    }

    const after = await getElementCount(page);
    expect(after).toBeGreaterThanOrEqual(before + 3);
  });
});
