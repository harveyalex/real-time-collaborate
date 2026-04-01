import { test, expect } from '@playwright/test';
import { drawRectangle } from '../helpers';

test.describe('Export', () => {
  test(':w triggers PNG download', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(2000);

    // Draw something to export
    await drawRectangle(page, 100, 100, 200, 150);
    await page.keyboard.press('Escape');
    await page.waitForTimeout(300);

    // Listen for download
    const downloadPromise = page.waitForEvent('download', { timeout: 5000 });

    // Execute :w command
    await page.keyboard.press(':');
    await page.keyboard.type('w');
    await page.keyboard.press('Enter');

    const download = await downloadPromise;
    expect(download.suggestedFilename()).toContain('.png');
  });

  test(':ws triggers SVG download', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(2000);

    // Draw something to export
    await drawRectangle(page, 100, 100, 200, 150);
    await page.keyboard.press('Escape');
    await page.waitForTimeout(300);

    // Listen for download
    const downloadPromise = page.waitForEvent('download', { timeout: 5000 });

    // Execute :ws command
    await page.keyboard.press(':');
    await page.keyboard.type('ws');
    await page.keyboard.press('Enter');

    const download = await downloadPromise;
    expect(download.suggestedFilename()).toContain('.svg');
  });
});
