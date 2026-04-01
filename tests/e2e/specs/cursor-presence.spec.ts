import { test, expect } from '@playwright/test';
import { waitForConnected, waitForCursorCount } from '../helpers';

test.describe('Cursor Presence', () => {
  test('cursor movement in browser A is visible in browser B', async ({ browser }) => {
    const contextA = await browser.newContext();
    const contextB = await browser.newContext();
    const pageA = await contextA.newPage();
    const pageB = await contextB.newPage();

    try {
      await pageA.goto('/');
      await pageB.goto('/');
      await waitForConnected(pageA);
      await waitForConnected(pageB);

      const canvas = pageA.locator('canvas');
      const box = await canvas.boundingBox();
      if (!box) throw new Error('Canvas not found');
      await pageA.mouse.move(box.x + 200, box.y + 200);

      await pageA.waitForTimeout(500);

      await waitForCursorCount(pageB, 1, 10_000);
    } finally {
      await contextA.close();
      await contextB.close();
    }
  });
});
