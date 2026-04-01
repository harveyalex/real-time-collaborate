import { test, expect } from '@playwright/test';

test.describe('Help Overlay', () => {
  test('? toggles help overlay', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(1000);

    // Help should not be visible initially
    await expect(page.locator('text=Keyboard Shortcuts')).not.toBeVisible();

    // Press ? to show help
    await page.keyboard.press('?');
    await page.waitForTimeout(300);
    await expect(page.locator('text=Keyboard Shortcuts')).toBeVisible();

    // Press ? again to hide
    await page.keyboard.press('?');
    await page.waitForTimeout(300);
    await expect(page.locator('text=Keyboard Shortcuts')).not.toBeVisible();
  });

  test('clicking outside closes help', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(1000);

    await page.keyboard.press('?');
    await expect(page.locator('text=Keyboard Shortcuts')).toBeVisible();

    // Click the backdrop (outside the modal)
    await page.mouse.click(10, 10);
    await page.waitForTimeout(300);
    await expect(page.locator('text=Keyboard Shortcuts')).not.toBeVisible();
  });
});
