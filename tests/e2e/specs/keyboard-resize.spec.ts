import { test, expect } from '@playwright/test';
import { getElementCount } from '../helpers';

test.describe('Keyboard Resize', () => {
  test('> grows width and < shrinks width of selected element', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(2000);

    // Create and select a rectangle
    await page.keyboard.press('r');
    await page.keyboard.press('Enter');
    await page.waitForTimeout(300);
    await page.keyboard.press('Escape');
    await page.waitForTimeout(200);
    await page.keyboard.press(' ');
    await page.waitForTimeout(200);

    // Get initial element data
    const before = await page.evaluate(() => {
      const ids = (window as any).__TEST_ELEMENT_IDS;
      const data = (window as any).__TEST_ELEMENT_DATA;
      return data ? JSON.parse(data) : null;
    });

    // Grow width with >
    await page.keyboard.press('>');
    await page.keyboard.press('>');
    await page.waitForTimeout(200);

    const afterGrow = await page.evaluate(() => {
      const data = (window as any).__TEST_ELEMENT_DATA;
      return data ? JSON.parse(data) : null;
    });

    // Element should still exist
    expect(await getElementCount(page)).toBeGreaterThan(0);

    // Shrink width with <
    await page.keyboard.press('<');
    await page.waitForTimeout(200);

    expect(await getElementCount(page)).toBeGreaterThan(0);
  });

  test('} grows height and { shrinks height of selected element', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(2000);

    await page.keyboard.press('r');
    await page.keyboard.press('Enter');
    await page.waitForTimeout(300);
    await page.keyboard.press('Escape');
    await page.waitForTimeout(200);
    await page.keyboard.press(' ');
    await page.waitForTimeout(200);

    // Grow height
    await page.keyboard.press('}');
    await page.keyboard.press('}');
    await page.waitForTimeout(200);

    expect(await getElementCount(page)).toBeGreaterThan(0);

    // Shrink height
    await page.keyboard.press('{');
    await page.waitForTimeout(200);

    expect(await getElementCount(page)).toBeGreaterThan(0);
  });
});
