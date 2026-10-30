import { test, expect } from '@playwright/test';

test('callsite table loads', async ({ page }) => {
    await page.goto('/finetune');

    const row = page.locator('tbody tr', { hasText: /TRACE/ });
    await expect(row).toBeVisible();
});

test('enable callsite', async ({ page }) => {
    await page.goto('/finetune');

    // 1. Wait for the table to load
    const tbody = page.locator('tbody');
    await expect(tbody.locator('tr').first()).toBeVisible();

    // 2. Find the row containing TRACE
    const row = tbody.locator('tr', { hasText: 'TRACE' }).first();

    // 3. Find the Enable button on that row
    const enableBtn = row.getByRole('button', { name: 'Enable' });

    // 4. Click it (clicking disables the button while the request is in flight)
    await enableBtn.click();

    // 5. Wait for the button to become enabled again
    await expect(enableBtn).toBeEnabled();

    // 6. Check another cell in the same row now shows ExplicitlyEnabled
    await expect(row.locator('td', { hasText: 'ExplicitlyEnabled' })).toBeVisible();
});

test('disable callsite', async ({ page }) => {
    await page.goto('/finetune');

    // 1. Wait for the table to load
    const tbody = page.locator('tbody');
    await expect(tbody.locator('tr').first()).toBeVisible();

    // 2. Find the row containing TRACE
    const row = tbody.locator('tr', { hasText: 'TRACE' }).first();

    // 3. Find the Disable button on that row
    const disableBtn = row.getByRole('button', { name: 'Disable' });

    // 4. Click it (clicking disables the button while the request is in flight)
    await disableBtn.click();

    // 5. Wait for the button to become enabled again
    await expect(disableBtn).toBeEnabled();

    // 6. Check another cell in the same row now shows ExplicitlyDisabled
    await expect(row.locator('td', { hasText: 'ExplicitlyDisabled' })).toBeVisible();
});

test('enable callsite then clear', async ({ page }) => {
    await page.goto('/finetune');

    // 1. Wait for the table to load
    const tbody = page.locator('tbody');
    await expect(tbody.locator('tr').first()).toBeVisible();

    // 2. Find the row containing TRACE
    const row = tbody.locator('tr', { hasText: 'TRACE' }).first();

    // 3. Find the Enable button on that row
    const enableBtn = row.getByRole('button', { name: 'Enable' });

    // 4. Click it (clicking disables the button while the request is in flight)
    await enableBtn.click();

    // 5. Wait for the button to become enabled again
    await expect(enableBtn).toBeEnabled();

    // 6. Check another cell in the same row now shows ExplicitlyEnabled
    await expect(row.locator('td', { hasText: 'ExplicitlyEnabled' })).toBeVisible();

    // 7. Find the Clear button on that row
    const clearBtn = row.getByRole('button', { name: 'Clear' });

    // 4. Click it (clicking disables the button while the request is in flight)
    await clearBtn.click();

    // 5. Wait for the button to become enabled again
    await expect(clearBtn).toBeEnabled();

    // 6. Check another cell in the same row now shows ExplicitlyEnabled
    await expect(row.locator('td', { hasText: 'NoOverride' })).toBeVisible();
});
