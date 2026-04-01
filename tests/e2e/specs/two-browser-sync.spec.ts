import { test, expect } from '@playwright/test';
import { waitForConnected, waitForElementCount, drawRectangle } from '../helpers';

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

      await drawRectangle(pageA, 100, 100, 200, 150);

      await waitForElementCount(pageA, 1);
      await waitForElementCount(pageB, 1, 15_000);
    } finally {
      await contextA.close();
      await contextB.close();
    }
  });
});
