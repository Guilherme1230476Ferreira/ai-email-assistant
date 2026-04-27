import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ fetch, cookies }) => {
	try {
		const token = cookies.get('token');
		const headers = token ? { 'Authorization': `Bearer ${token}` } : {};

		// Fetch data from the Rust endpoints via the Vite proxy
		const [emailsRes, usersRes, rolesRes, settingsRes] = await Promise.all([
			fetch('/api/emails', { headers }).catch(() => ({ json: () => [], ok: false })),
			fetch('/api/admin/users', { headers }).catch(() => ({ json: () => [], ok: false })),
			fetch('/api/admin/roles', { headers }).catch(() => ({ json: () => [], ok: false })),
			fetch('/api/admin/settings', { headers }).catch(() => ({ json: () => null, ok: false }))
		]);

		const [emails, users, roles, settings] = await Promise.all([
			emailsRes.ok ? emailsRes.json() : [],
			usersRes.ok ? usersRes.json() : [],
			rolesRes.ok ? rolesRes.json() : [],
			settingsRes.ok ? settingsRes.json() : null,
		]);

		return {
			stats: {
				emailsCount: Array.isArray(emails) ? emails.length : 0,
				usersCount: Array.isArray(users) ? users.length : 0,
				rolesCount: Array.isArray(roles) ? roles.length : 0,
			},
			llm: settings || null,
			recentEmails: Array.isArray(emails) ? emails.slice(0, 4) : []
		};
	} catch (error) {
		console.error("Error fetching dashboard data:", error);
		return {
			stats: { emailsCount: 0, usersCount: 0, rolesCount: 0 },
			llm: null,
			recentEmails: []
		};
	}
};
