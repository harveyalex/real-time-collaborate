import { test, expect } from '@playwright/test';
import { getElementCount } from '../helpers';

test.describe('Keyboard-Only Drawing', () => {
  test('create rectangle with keyboard only (r, Enter to place at center)', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(2000);

    const before = await getElementCount(page);

    // Press r to enter rect mode, then Enter to place at cursor/center
    await page.keyboard.press('r');
    await expect(page.locator('text=INSERT')).toBeVisible();

    // Press Enter to create a default-sized rectangle at center of canvas
    await page.keyboard.press('Enter');
    await page.waitForTimeout(500);

    const after = await getElementCount(page);
    expect(after).toBeGreaterThan(before);
  });

  test('create ellipse with keyboard only', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(2000);

    const before = await getElementCount(page);

    await page.keyboard.press('e');
    await page.keyboard.press('Enter');
    await page.waitForTimeout(500);

    const after = await getElementCount(page);
    expect(after).toBeGreaterThan(before);
  });

  test('create text with keyboard only (t, Enter to place, type, Escape)', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(2000);

    const before = await getElementCount(page);

    await page.keyboard.press('t');
    await page.keyboard.press('Enter'); // place text at center
    await page.keyboard.type('keyboard text');
    await page.keyboard.press('Escape');
    await page.waitForTimeout(500);

    const after = await getElementCount(page);
    expect(after).toBeGreaterThan(before);
  });
});
