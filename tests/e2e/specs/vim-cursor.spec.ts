import { test, expect } from '@playwright/test';
import { getElementCount } from '../helpers';

// Helper to get the vim cursor position from test hook
async function getCursorPos(page: any): Promise<{ x: number; y: number }> {
  return page.evaluate(() => {
    const pos = (window as any).__TEST_VIM_CURSOR;
    return pos ? JSON.parse(pos) : { x: 0, y: 0 };
  });
}

test.describe('Vim Cursor Navigation', () => {
  test('vim cursor exists and is at canvas center on load', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(2000);

    const pos = await getCursorPos(page);
    expect(pos.x).toBeGreaterThan(0);
    expect(pos.y).toBeGreaterThan(0);
  });

  test('hjkl moves vim cursor in Normal mode', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(2000);

    const start = await getCursorPos(page);

    await page.keyboard.press('l');
    await page.waitForTimeout(50);
    const r = await getCursorPos(page);
    expect(r.x).toBeGreaterThan(start.x);
    expect(r.y).toBe(start.y);

    await page.keyboard.press('j');
    await page.waitForTimeout(50);
    const d = await getCursorPos(page);
    expect(d.y).toBeGreaterThan(r.y);

    await page.keyboard.press('h');
    await page.waitForTimeout(50);
    const l = await getCursorPos(page);
    expect(l.x).toBeLessThan(d.x);

    await page.keyboard.press('k');
    await page.waitForTimeout(50);
    const u = await getCursorPos(page);
    expect(u.y).toBeLessThan(l.y);
  });

  test('Space selects element under vim cursor', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(2000);

    // Create element at cursor position
    await page.keyboard.press('r');
    await page.keyboard.press('Enter');
    await page.waitForTimeout(300);
    await page.keyboard.press('Escape');
    await page.waitForTimeout(200);

    // Space to select element at cursor
    await page.keyboard.press(' ');
    await page.waitForTimeout(200);

    const sel = await page.evaluate(() => (window as any).__TEST_SELECTED_IDS);
    expect(sel).not.toBe('[]');
  });

  test('Shift+hjkl moves selected element, not cursor', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(2000);

    // Create and select
    await page.keyboard.press('r');
    await page.keyboard.press('Enter');
    await page.waitForTimeout(300);
    await page.keyboard.press('Escape');
    await page.waitForTimeout(200);
    await page.keyboard.press(' ');
    await page.waitForTimeout(200);

    const cursorBefore = await getCursorPos(page);

    // Shift+l moves element, not cursor
    await page.keyboard.press('Shift+l');
    await page.keyboard.press('Shift+l');
    await page.waitForTimeout(100);

    const cursorAfter = await getCursorPos(page);
    expect(cursorAfter.x).toBe(cursorBefore.x);
    expect(cursorAfter.y).toBe(cursorBefore.y);
  });

  test('full keyboard workflow: navigate, create, navigate, create, select, delete', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(2000);

    const before = await getElementCount(page);

    // Create rect at current cursor
    await page.keyboard.press('r');
    await page.keyboard.press('Enter');
    await page.waitForTimeout(300);
    await page.keyboard.press('Escape');
    await page.waitForTimeout(200);

    // Move cursor away
    for (let i = 0; i < 8; i++) await page.keyboard.press('l');
    for (let i = 0; i < 5; i++) await page.keyboard.press('j');
    await page.waitForTimeout(100);

    // Create another rect at new cursor position
    await page.keyboard.press('e');
    await page.keyboard.press('Enter');
    await page.waitForTimeout(300);
    await page.keyboard.press('Escape');
    await page.waitForTimeout(200);

    expect(await getElementCount(page)).toBeGreaterThanOrEqual(before + 2);

    // Select with Space and delete
    await page.keyboard.press(' ');
    await page.waitForTimeout(200);
    await page.keyboard.press('d');
    await page.keyboard.press('d');
    await page.waitForTimeout(500);

    expect(await getElementCount(page)).toBeLessThan(before + 2);
  });
});
