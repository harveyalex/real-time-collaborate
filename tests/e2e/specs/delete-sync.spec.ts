import { test, expect } from '@playwright/test';
import { waitForConnected, waitForElementCount, drawRectangle } from '../helpers';

test.describe('Delete Sync', () => {
  test('deleting an element in browser A removes it from browser B', async ({ browser }) => {
    const contextA = await browser.newContext();
    const contextB = await browser.newContext();
    const pageA = await contextA.newPage();
    const pageB = await contextB.newPage();

    try {
      await pageA.goto('/');
      await pageB.goto('/');
      await waitForConnected(pageA);
      await waitForConnected(pageB);

      await drawRectangle(pageA, 100, 100, 200, 150);
      await waitForElementCount(pageA, 1);
      await waitForElementCount(pageB, 1, 15_000);

      const canvasA = pageA.locator('canvas');
      const boxA = await canvasA.boundingBox();
      if (!boxA) throw new Error('Canvas not found');

      // Return to normal mode and select
      await pageA.keyboard.press('Escape');
      await pageA.waitForTimeout(100);
      await pageA.mouse.click(boxA.x + 200, boxA.y + 175);
      await pageA.waitForTimeout(200);

      // Delete with dd
      await pageA.keyboard.press('d');
      await pageA.keyboard.press('d');
      await pageA.waitForTimeout(200);

      await waitForElementCount(pageA, 0, 5_000);
      await waitForElementCount(pageB, 0, 15_000);
    } finally {
      await contextA.close();
      await contextB.close();
    }
  });
});
