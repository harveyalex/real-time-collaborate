import { test, expect } from '@playwright/test';
import { waitForConnected, waitForElementCountAtLeast, getElementCount, drawRectangle } from '../helpers';

test.describe('Two-Browser Sync', () => {
  test('rectangle drawn in browser A appears in browser B', async ({ browser }) => {
    const contextA = await browser.newContext();
    const contextB = await browser.newContext();
    const pageA = await contextA.newPage();
    const pageB = await contextB.newPage();

    try {
      await pageA.goto('/');
      await pageB.goto('/');

      await waitForConnected(pageA);
      await waitForConnected(pageB);

      // Get baseline element count for browser B
      await pageB.waitForTimeout(1000); // let subscription settle
      const baseB = await getElementCount(pageB);

      // Draw a rectangle in page A
      await drawRectangle(pageA, 100, 100, 200, 150);

      // Page A should have at least 1 more element (local insert)
      await pageA.waitForTimeout(500);
      const afterA = await getElementCount(pageA);
      expect(afterA).toBeGreaterThan(0);

      // Page B should see the element arrive from server
      await waitForElementCountAtLeast(pageB, baseB + 1, 15_000);
    } finally {
      await contextA.close();
      await contextB.close();
    }
  });
});
