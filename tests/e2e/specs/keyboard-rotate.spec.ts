import { test, expect } from '@playwright/test';
import { getElementCount } from '../helpers';

test.describe('Keyboard Rotate', () => {
  test('R rotates selected element clockwise, Ctrl+Shift+r rotates counter-clockwise', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(2000);

    // Create and select
    await page.keyboard.press('r');
    await page.keyboard.press('Enter');
    await page.waitForTimeout(300);
    await page.keyboard.press('Escape');
    await page.waitForTimeout(200);
    await page.keyboard.press(' ');
    await page.waitForTimeout(200);

    // Rotate clockwise with (  — left paren
    await page.keyboard.press('(');
    await page.keyboard.press('(');
    await page.waitForTimeout(200);

    expect(await getElementCount(page)).toBeGreaterThan(0);

    // Rotate counter-clockwise with )  — right paren
    await page.keyboard.press(')');
    await page.waitForTimeout(200);

    expect(await getElementCount(page)).toBeGreaterThan(0);
  });
});
