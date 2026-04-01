import { test, expect } from '@playwright/test';
import { getElementCount, waitForConnected } from '../helpers';

test.describe('Text Input', () => {
  test('pressing t, clicking, and typing creates text element', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(2000);

    const before = await getElementCount(page);

    // Enter text mode
    await page.keyboard.press('t');
    await expect(page.locator('text=INSERT')).toBeVisible();

    // Click on canvas to place text cursor
    const canvas = page.locator('canvas');
    const box = await canvas.boundingBox();
    if (!box) throw new Error('Canvas not found');
    await page.mouse.click(box.x + 200, box.y + 200);
    await page.waitForTimeout(200);

    // Type some text
    await page.keyboard.type('hello world');
    await page.waitForTimeout(200);

    // Press Escape to commit
    await page.keyboard.press('Escape');
    await page.waitForTimeout(500);

    const after = await getElementCount(page);
    expect(after).toBeGreaterThan(before);
  });
});
