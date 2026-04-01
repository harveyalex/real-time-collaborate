import { test, expect } from '@playwright/test';
import { waitForConnected, waitForElementCountAtLeast, waitForElementCount, getElementCount, drawRectangle } from '../helpers';

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

      await pageA.waitForTimeout(1000);
      const baseA = await getElementCount(pageA);
      const baseB = await getElementCount(pageB);

      // Draw rectangle in A
      await drawRectangle(pageA, 100, 100, 200, 150);

      // Wait for element to sync to both browsers
      await waitForElementCountAtLeast(pageA, baseA + 1, 10_000);
      await waitForElementCountAtLeast(pageB, baseB + 1, 15_000);

      // Wait for dedup to settle (temp ID replaced by server ID)
      await pageA.waitForTimeout(2000);
      const beforeDeleteA = await getElementCount(pageA);
      const beforeDeleteB = await getElementCount(pageB);

      // Return to normal mode
      await pageA.keyboard.press('Escape');
      await pageA.waitForTimeout(200);

      // Click on the rectangle to select it (center of where we drew)
      const canvasA = pageA.locator('canvas');
      const boxA = await canvasA.boundingBox();
      if (!boxA) throw new Error('Canvas not found');
      await pageA.mouse.click(boxA.x + 200, boxA.y + 175);
      await pageA.waitForTimeout(300);

      // Delete with dd
      await pageA.keyboard.press('d');
      await pageA.keyboard.press('d');

      // Wait for delete to propagate
      await pageA.waitForTimeout(1000);
      const afterDeleteA = await getElementCount(pageA);
      expect(afterDeleteA).toBeLessThan(beforeDeleteA);

      // Browser B should also see the delete
      await pageB.waitForFunction(
        (expected) => ((window as any).__TEST_ELEMENT_COUNT ?? 999) < expected,
        beforeDeleteB,
        { timeout: 15_000 },
      );
    } finally {
      await contextA.close();
      await contextB.close();
    }
  });
});
