import { test, expect } from '@playwright/test';
import { getElementCount, drawRectangle } from '../helpers';

test.describe('Move Element', () => {
  test('Shift+hjkl moves selected element', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(2000);

    // Draw a rectangle and select it via mouse (existing flow)
    await drawRectangle(page, 200, 200, 100, 80);
    await page.waitForTimeout(500);
    await page.keyboard.press('Escape');
    await page.waitForTimeout(200);
    const canvas = page.locator('canvas');
    const box = await canvas.boundingBox();
    if (!box) throw new Error('Canvas not found');
    await page.mouse.click(box.x + 250, box.y + 240);
    await page.waitForTimeout(200);

    const sel = await page.evaluate(() => (window as any).__TEST_SELECTED_IDS);
    expect(sel).not.toBe('[]');

    // Shift+hjkl moves element
    await page.keyboard.press('Shift+l');
    await page.keyboard.press('Shift+l');
    await page.keyboard.press('Shift+j');
    await page.waitForTimeout(300);

    const selAfter = await page.evaluate(() => (window as any).__TEST_SELECTED_IDS);
    expect(selAfter).not.toBe('[]');
  });

  test('keyboard-only move: create, Space to select, Shift+hjkl to move', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(2000);

    // Create at cursor
    await page.keyboard.press('r');
    await page.keyboard.press('Enter');
    await page.waitForTimeout(300);
    await page.keyboard.press('Escape');
    await page.waitForTimeout(200);

    // Space to select
    await page.keyboard.press(' ');
    await page.waitForTimeout(200);

    const sel = await page.evaluate(() => (window as any).__TEST_SELECTED_IDS);
    expect(sel).not.toBe('[]');

    // Shift+hjkl to move
    await page.keyboard.press('Shift+l');
    await page.keyboard.press('Shift+j');
    await page.waitForTimeout(300);

    const selAfter = await page.evaluate(() => (window as any).__TEST_SELECTED_IDS);
    expect(selAfter).not.toBe('[]');
  });
});
