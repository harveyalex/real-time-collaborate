import { test, expect } from '@playwright/test';
import { getElementCount, drawRectangle } from '../helpers';

test.describe('Visual Mode', () => {
  test('v enters visual mode, d deletes all selected', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(2000);

    // Draw two rectangles
    await drawRectangle(page, 50, 50, 100, 80);
    await page.waitForTimeout(300);
    await drawRectangle(page, 200, 50, 100, 80);
    await page.waitForTimeout(300);

    const afterDraw = await getElementCount(page);
    expect(afterDraw).toBeGreaterThanOrEqual(2);

    // Return to normal
    await page.keyboard.press('Escape');
    await page.waitForTimeout(200);

    // Enter visual mode
    await page.keyboard.press('v');
    await expect(page.locator('text=VISUAL')).toBeVisible();

    // Select elements by clicking
    const canvas = page.locator('canvas');
    const box = await canvas.boundingBox();
    if (!box) throw new Error('Canvas not found');
    await page.mouse.click(box.x + 100, box.y + 90);
    await page.waitForTimeout(200);

    // Delete in visual mode
    await page.keyboard.press('d');
    await page.waitForTimeout(500);

    // Should return to normal
    await expect(page.locator('text=NORMAL')).toBeVisible();

    const afterDelete = await getElementCount(page);
    expect(afterDelete).toBeLessThan(afterDraw);
  });
});
