import { test, expect } from '@playwright/test';

/**
 * E2E TEST SUITE - Core Functionality
 * 
 * This suite tests the backoffice frontend UI and navigation 
 * without requiring the Rust backend to be running.
 */

// ─── Route Protection ─────────────────────────────────────────────────────────

test.describe('Route Protection', () => {
	const protectedRoutes = ['/users', '/roles', '/settings', '/audit-logs', '/emails'];

	for (const route of protectedRoutes) {
		test(`redirects "${route}" to /login when unauthenticated`, async ({ page }) => {
			await page.goto(route);
			await expect(page).toHaveURL(/.*\/login/);
		});
	}

	test('root "/" redirects to /login when unauthenticated', async ({ page }) => {
		await page.goto('/');
		await expect(page).toHaveURL(/.*\/login/);
	});
});

// ─── Login Page ───────────────────────────────────────────────────────────────

test.describe('Login Page', () => {
	test.beforeEach(async ({ page }) => {
		await page.goto('/login');
	});

	test('renders all required elements', async ({ page }) => {
		await expect(page.getByRole('heading', { name: /sign in to mailmate/i })).toBeVisible();
		await expect(page.getByLabel(/email/i)).toBeVisible();
		await expect(page.getByLabel(/password/i)).toBeVisible();
		await expect(page.getByRole('button', { name: 'Sign in', exact: true })).toBeVisible();
	});

	test('has a working link to the signup page', async ({ page }) => {
		const link = page.getByRole('link', { name: /create one/i });
		await expect(link).toBeVisible();
		await expect(link).toHaveAttribute('href', '/signup');
	});

	test('shows error when backend is unreachable', async ({ page }) => {
		await page.getByLabel(/email/i).fill('user@example.com');
		await page.getByLabel(/password/i).fill('password123');
		await page.getByRole('button', { name: 'Sign in', exact: true }).click();
		
		// Graceful error handling when backend is down
		await expect(page.getByText(/Could not reach the server|Invalid email or password/i)).toBeVisible();
	});
});

// ─── Signup Page ──────────────────────────────────────────────────────────────

test.describe('Signup Page', () => {
	test.beforeEach(async ({ page }) => {
		await page.goto('/signup');
	});

	test('renders all required elements', async ({ page }) => {
		await expect(page.getByRole('heading', { name: /create an account/i })).toBeVisible();
		await expect(page.getByLabel(/^email$/i)).toBeVisible();
		await expect(page.getByLabel(/^password$/i)).toBeVisible();
		await expect(page.getByLabel(/repeat password/i)).toBeVisible();
		await expect(page.getByRole('button', { name: /create account/i })).toBeVisible();
	});

	test('has a working link back to login', async ({ page }) => {
		const link = page.getByRole('link', { name: /log in instead/i });
		await expect(link).toBeVisible();
		await expect(link).toHaveAttribute('href', '/login');
	});
});

// ─── Navigation ───────────────────────────────────────────────────────────────

test.describe('Public Navigation', () => {
	test('can navigate from login to signup', async ({ page }) => {
		await page.goto('/login');
		await page.getByRole('link', { name: /create one/i }).click();
		await expect(page).toHaveURL(/.*\/signup/);
	});

	test('can navigate from signup to login', async ({ page }) => {
		await page.goto('/signup');
		await page.getByRole('link', { name: /log in instead/i }).click();
		await expect(page).toHaveURL(/.*\/login/);
	});
});
