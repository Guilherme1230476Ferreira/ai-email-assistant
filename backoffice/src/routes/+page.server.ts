import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ fetch, cookies, parent }) => {
	try {
		const token = cookies.get('token');
		const headers: Record<string, string> = token ? { 'Authorization': `Bearer ${token}` } : {};

		// Get user role from parent layout data
		const parentData = await parent();
		const userRole = parentData.user?.role || 'user';
		const isAdmin = userRole === 'admin';

		// Always fetch user-accessible data
		const [emailsRes, telemetryRes] = await Promise.all([
			fetch('/api/emails', { headers }).catch(() => ({ json: () => [], ok: false })),
			fetch('/api/telemetry', { headers }).catch(() => ({ json: () => null, ok: false })),
		]);

		// Only fetch admin data if the user is an admin
		let usersCount = 0;
		let rolesCount = 0;
		let settings = null;

		if (isAdmin) {
			const [usersRes, rolesRes, settingsRes] = await Promise.all([
				fetch('/api/admin/users', { headers }).catch(() => ({ json: () => [], ok: false })),
				fetch('/api/admin/roles', { headers }).catch(() => ({ json: () => [], ok: false })),
				fetch('/api/admin/settings', { headers }).catch(() => ({ json: () => null, ok: false })),
			]);

			const [users, roles, settingsData] = await Promise.all([
				usersRes.ok ? usersRes.json() : { items: [], total: 0 },
				rolesRes.ok ? rolesRes.json() : [],
				settingsRes.ok ? settingsRes.json() : null,
			]);

			usersCount = users.total !== undefined ? users.total : (Array.isArray(users) ? users.length : 0);
			rolesCount = Array.isArray(roles) ? roles.length : 0;
			settings = settingsData;
		} else {
			// Non-admin users can still see LLM settings (read-only)
			try {
				const settingsRes = await fetch('/api/admin/settings', { headers });
				if (settingsRes.ok) settings = await settingsRes.json();
			} catch {
				// Silently fail — user doesn't have access
			}
		}

		const [emails, telemetry] = await Promise.all([
			emailsRes.ok ? emailsRes.json() : { items: [], total: 0 },
			telemetryRes.ok ? telemetryRes.json() : null,
		]);

		return {
			userRole,
			stats: {
				emailsCount: emails.total !== undefined ? emails.total : (Array.isArray(emails) ? emails.length : 0),
				usersCount,
				rolesCount,
			},
			telemetry,
			llm: settings || null,
			recentEmails: Array.isArray(emails.items) ? emails.items.slice(0, 4) : []
		};
	} catch (error) {
		console.error("Error fetching dashboard data:", error);
		return {
			userRole: 'user',
			stats: { emailsCount: 0, usersCount: 0, rolesCount: 0 },
			telemetry: null,
			llm: null,
			recentEmails: []
		};
	}
};
