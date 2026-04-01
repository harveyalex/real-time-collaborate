import { test, expect } from '@playwright/test';

test.describe('Canvas Loads', () => {
  test('page loads with canvas and mode indicator', async ({ page }) => {
    const errors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') errors.push(msg.text());
    });

    await page.goto('/');

    const canvas = page.locator('canvas');
    await expect(canvas).toBeVisible();

    await expect(page.locator('text=NORMAL')).toBeVisible();

    await page.waitForTimeout(2000);
    const realErrors = errors.filter(e => !e.includes('SpacetimeDB') && !e.includes('WebSocket'));
    expect(realErrors).toEqual([]);
  });
});
