import { test, expect } from '@playwright/test';

test.describe('Keyboard Pan', () => {
  test('Ctrl+hjkl pans the canvas viewport', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(2000);

    // Get initial camera position via zoom display (shows world coords)
    const getCamera = () => page.evaluate(() => {
      const cam = (window as any).__TEST_CAMERA;
      return cam ? JSON.parse(cam) : { x: 0, y: 0, zoom: 1 };
    });

    const before = await getCamera();

    // Pan right with Ctrl+l
    await page.keyboard.press('Control+l');
    await page.keyboard.press('Control+l');
    await page.waitForTimeout(100);

    const afterRight = await getCamera();
    expect(afterRight.x).toBeGreaterThan(before.x);

    // Pan down with Ctrl+j
    await page.keyboard.press('Control+j');
    await page.waitForTimeout(100);

    const afterDown = await getCamera();
    expect(afterDown.y).toBeGreaterThan(afterRight.y);

    // Pan left with Ctrl+h
    await page.keyboard.press('Control+h');
    await page.waitForTimeout(100);

    const afterLeft = await getCamera();
    expect(afterLeft.x).toBeLessThan(afterDown.x);

    // Pan up with Ctrl+k
    await page.keyboard.press('Control+k');
    await page.waitForTimeout(100);

    const afterUp = await getCamera();
    expect(afterUp.y).toBeLessThan(afterLeft.y);
  });
});
