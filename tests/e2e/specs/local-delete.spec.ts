import { test, expect } from '@playwright/test';
import { getElementCount, drawRectangle } from '../helpers';

test.describe('Local Delete', () => {
  test('dd deletes selected element', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(2000);

    const before = await getElementCount(page);

    // Draw a rectangle
    await drawRectangle(page, 100, 100, 200, 150);
    await page.waitForTimeout(500);

    const afterDraw = await getElementCount(page);
    expect(afterDraw).toBeGreaterThan(before);

    // Return to normal mode
    await page.keyboard.press('Escape');
    await page.waitForTimeout(200);

    // Click on the rectangle to select it
    const canvas = page.locator('canvas');
    const box = await canvas.boundingBox();
    if (!box) throw new Error('Canvas not found');
    await page.mouse.click(box.x + 200, box.y + 175);
    await page.waitForTimeout(200);

    // Delete with dd
    await page.keyboard.press('d');
    await page.keyboard.press('d');
    await page.waitForTimeout(500);

    const afterDelete = await getElementCount(page);
    expect(afterDelete).toBeLessThan(afterDraw);
  });
});
