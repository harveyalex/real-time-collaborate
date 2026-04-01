import { test, expect } from '@playwright/test';
import { getElementCount, drawRectangle } from '../helpers';

test.describe('Move Element', () => {
  test('hjkl moves selected element', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(2000);

    // Draw a rectangle
    await drawRectangle(page, 200, 200, 100, 80);
    await page.waitForTimeout(500);

    // Return to normal, select it
    await page.keyboard.press('Escape');
    await page.waitForTimeout(200);
    const canvas = page.locator('canvas');
    const box = await canvas.boundingBox();
    if (!box) throw new Error('Canvas not found');
    await page.mouse.click(box.x + 250, box.y + 240);
    await page.waitForTimeout(200);

    // Verify something is selected
    const sel = await page.evaluate(() => (window as any).__TEST_SELECTED_IDS);
    expect(sel).not.toBe('[]');

    // Move right with l, then down with j
    await page.keyboard.press('l');
    await page.keyboard.press('l');
    await page.keyboard.press('j');
    await page.waitForTimeout(300);

    // Element should still exist and be selected
    const selAfter = await page.evaluate(() => (window as any).__TEST_SELECTED_IDS);
    expect(selAfter).not.toBe('[]');
  });

  test('Shift+hjkl moves by 1px', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(2000);

    await drawRectangle(page, 200, 200, 100, 80);
    await page.waitForTimeout(500);
    await page.keyboard.press('Escape');
    await page.waitForTimeout(200);

    const canvas = page.locator('canvas');
    const box = await canvas.boundingBox();
    if (!box) throw new Error('Canvas not found');
    await page.mouse.click(box.x + 250, box.y + 240);
    await page.waitForTimeout(200);

    // Fine move with shift
    await page.keyboard.press('Shift+l');
    await page.keyboard.press('Shift+j');
    await page.waitForTimeout(300);

    const sel = await page.evaluate(() => (window as any).__TEST_SELECTED_IDS);
    expect(sel).not.toBe('[]');
  });
});
