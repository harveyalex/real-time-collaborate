import { test, expect } from '@playwright/test';
import { getElementCount, drawRectangle } from '../helpers';

test.describe('Copy and Paste', () => {
  test('yy copies and p pastes the selected element', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(2000);

    // Draw a rectangle
    await drawRectangle(page, 100, 100, 200, 150);
    await page.waitForTimeout(500);
    const afterDraw = await getElementCount(page);

    // Return to normal, select it
    await page.keyboard.press('Escape');
    await page.waitForTimeout(200);
    const canvas = page.locator('canvas');
    const box = await canvas.boundingBox();
    if (!box) throw new Error('Canvas not found');
    await page.mouse.click(box.x + 200, box.y + 175);
    await page.waitForTimeout(200);

    // Copy with yy
    await page.keyboard.press('y');
    await page.keyboard.press('y');
    await page.waitForTimeout(200);

    // Paste with p
    await page.keyboard.press('p');
    await page.waitForTimeout(500);

    const afterPaste = await getElementCount(page);
    expect(afterPaste).toBeGreaterThan(afterDraw);
  });
});
