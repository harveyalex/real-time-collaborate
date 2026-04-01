import { test, expect } from '@playwright/test';

test.describe('Keyboard Zoom', () => {
  test('+ zooms in and - zooms out', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(1000);

    // Get initial zoom text from bottom bar
    const getZoom = () => page.evaluate(() => {
      const text = document.body?.innerText || '';
      const match = text.match(/(\d+)%/);
      return match ? parseInt(match[1]) : 100;
    });

    const initial = await getZoom();

    // Zoom in with +
    await page.keyboard.press('+');
    await page.waitForTimeout(200);
    await page.keyboard.press('+');
    await page.waitForTimeout(200);

    const afterIn = await getZoom();
    expect(afterIn).toBeGreaterThan(initial);

    // Zoom out with -
    await page.keyboard.press('-');
    await page.waitForTimeout(200);
    await page.keyboard.press('-');
    await page.waitForTimeout(200);
    await page.keyboard.press('-');
    await page.waitForTimeout(200);

    const afterOut = await getZoom();
    expect(afterOut).toBeLessThan(afterIn);
  });
});
