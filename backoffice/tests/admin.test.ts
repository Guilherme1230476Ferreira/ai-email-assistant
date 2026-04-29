import { test, expect } from '@playwright/test';

/**
 * Admin management tests.
 * 
 * These tests now pass even without the backend running, thanks to the 
 * SSR fetch mocking in src/hooks.server.ts.
 */

test.describe('Admin Management', () => {
    test.beforeEach(async ({ page, context }) => {
        // Set a fake token cookie to satisfy the layout's auth check
        await context.addCookies([
            {
                name: 'token',
                value: 'fake-session-token',
                domain: 'localhost',
                path: '/'
            }
        ]);
    });

    test('renders the user management page', async ({ page }) => {
        await page.goto('/users');
        await expect(page).toHaveURL('/users');
        // Use a more specific selector to avoid strict mode violations
        await expect(page.getByRole('heading', { level: 1, name: /^Users$/i })).toBeVisible();
    });

    test('renders the roles management page', async ({ page }) => {
        await page.goto('/roles');
        await expect(page).toHaveURL('/roles');
        await expect(page.getByRole('heading', { level: 1, name: /^Roles$/i })).toBeVisible();
    });

    test('renders the settings page', async ({ page }) => {
        await page.goto('/settings');
        await expect(page).toHaveURL('/settings');
        await expect(page.getByRole('heading', { level: 1, name: /^Settings$/i })).toBeVisible();
    });

    test('renders the audit logs page', async ({ page }) => {
        await page.goto('/audit-logs');
        await expect(page).toHaveURL('/audit-logs');
        await expect(page.getByRole('heading', { level: 1, name: /^Audit Logs$/i })).toBeVisible();
    });
});
