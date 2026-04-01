import { test, expect } from '@playwright/test';
import { getElementCount } from '../helpers';

test.describe('Keyboard Freehand', () => {
  test('f enters freehand mode, hjkl draws path, Escape commits stroke', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(2000);

    const before = await getElementCount(page);

    // Enter freehand mode
    await page.keyboard.press('f');
    await expect(page.locator('text=INSERT')).toBeVisible();

    // Start drawing with Enter
    await page.keyboard.press('Enter');
    await page.waitForTimeout(100);

    // Draw a path with hjkl
    await page.keyboard.press('l');
    await page.keyboard.press('l');
    await page.keyboard.press('j');
    await page.keyboard.press('l');
    await page.keyboard.press('j');
    await page.keyboard.press('j');
    await page.keyboard.press('h');
    await page.keyboard.press('j');
    await page.waitForTimeout(100);

    // Escape commits the freehand stroke
    await page.keyboard.press('Escape');
    await page.waitForTimeout(500);

    const after = await getElementCount(page);
    expect(after).toBeGreaterThan(before);
  });

  test('multiple freehand strokes: Enter starts new, hjkl draws, Enter commits and starts another', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(2000);

    const before = await getElementCount(page);

    await page.keyboard.press('f');

    // First stroke
    await page.keyboard.press('Enter');
    for (let i = 0; i < 5; i++) await page.keyboard.press('l');
    await page.keyboard.press('Enter'); // commit and start new
    await page.waitForTimeout(200);

    // Move cursor for second stroke
    for (let i = 0; i < 3; i++) await page.keyboard.press('j');

    // Second stroke
    await page.keyboard.press('Enter');
    for (let i = 0; i < 5; i++) await page.keyboard.press('h');
    await page.keyboard.press('Escape'); // commit and exit
    await page.waitForTimeout(500);

    const after = await getElementCount(page);
    expect(after).toBeGreaterThanOrEqual(before + 2);
  });
});
