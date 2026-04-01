import { Page } from '@playwright/test';

export async function getElementCount(page: Page): Promise<number> {
  return page.evaluate(() => (window as any).__TEST_ELEMENT_COUNT ?? 0);
}

export async function getCursorCount(page: Page): Promise<number> {
  return page.evaluate(() => (window as any).__TEST_CURSOR_COUNT ?? 0);
}

export async function waitForElementCount(
  page: Page,
  count: number,
  timeout = 10_000,
): Promise<void> {
  await page.waitForFunction(
    (expected) => (window as any).__TEST_ELEMENT_COUNT === expected,
    count,
    { timeout },
  );
}

export async function waitForElementCountAtLeast(
  page: Page,
  minCount: number,
  timeout = 10_000,
): Promise<void> {
  await page.waitForFunction(
    (min) => ((window as any).__TEST_ELEMENT_COUNT ?? 0) >= min,
    minCount,
    { timeout },
  );
}

export async function waitForElementCountChange(
  page: Page,
  baseline: number,
  delta: number,
  timeout = 10_000,
): Promise<void> {
  const target = baseline + delta;
  await page.waitForFunction(
    (expected) => (window as any).__TEST_ELEMENT_COUNT === expected,
    target,
    { timeout },
  );
}

export async function waitForCursorCount(
  page: Page,
  minCount: number,
  timeout = 10_000,
): Promise<void> {
  await page.waitForFunction(
    (min) => ((window as any).__TEST_CURSOR_COUNT ?? 0) >= min,
    minCount,
    { timeout },
  );
}

export async function waitForConnected(page: Page, timeout = 15_000): Promise<void> {
  await page.waitForFunction(
    () => document.body?.innerText?.toLowerCase().includes('connected'),
    undefined,
    { timeout },
  );
}

export async function drawRectangle(
  page: Page,
  x: number,
  y: number,
  width: number,
  height: number,
): Promise<void> {
  await page.keyboard.press('r');
  await page.waitForTimeout(100);

  const canvas = page.locator('canvas');
  const box = await canvas.boundingBox();
  if (!box) throw new Error('Canvas not found');

  const startX = box.x + x;
  const startY = box.y + y;

  await page.mouse.move(startX, startY);
  await page.mouse.down();
  await page.mouse.move(startX + width, startY + height, { steps: 5 });
  await page.mouse.up();

  await page.waitForTimeout(200);
}
