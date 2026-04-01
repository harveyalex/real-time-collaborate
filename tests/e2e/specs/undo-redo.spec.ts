import { test, expect } from '@playwright/test';
import { getElementCount, drawRectangle } from '../helpers';

test.describe('Undo and Redo', () => {
  test('u undoes the last action (element creation)', async ({ page }) => {
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

    // Undo
    await page.keyboard.press('u');
    await page.waitForTimeout(500);

    const afterUndo = await getElementCount(page);
    expect(afterUndo).toBeLessThan(afterDraw);
  });

  test('Ctrl+r redoes the undone action', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(2000);

    const before = await getElementCount(page);

    await drawRectangle(page, 100, 100, 200, 150);
    await page.waitForTimeout(500);
    await page.keyboard.press('Escape');
    await page.waitForTimeout(200);

    // Undo
    await page.keyboard.press('u');
    await page.waitForTimeout(500);
    const afterUndo = await getElementCount(page);

    // Redo (Ctrl+r)
    await page.keyboard.press('Control+r');
    await page.waitForTimeout(500);

    const afterRedo = await getElementCount(page);
    expect(afterRedo).toBeGreaterThan(afterUndo);
  });
});
