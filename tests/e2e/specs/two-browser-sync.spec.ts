import { test, expect } from '@playwright/test';
import { waitForConnected, waitForElementCountChange, getElementCount, drawRectangle } from '../helpers';

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

      // Get baseline element counts
      await pageA.waitForTimeout(1000); // let subscription settle
      const baseA = await getElementCount(pageA);
      const baseB = await getElementCount(pageB);

      // Draw a rectangle in page A
      await drawRectangle(pageA, 100, 100, 200, 150);

      // Verify element count increased by 1 in page A
      await waitForElementCountChange(pageA, baseA, 1);

      // Verify element syncs to page B (server-assigned element appears)
      await waitForElementCountChange(pageB, baseB, 1, 15_000);
    } finally {
      await contextA.close();
      await contextB.close();
    }
  });
});
