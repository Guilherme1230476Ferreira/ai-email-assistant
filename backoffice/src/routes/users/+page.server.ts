import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ fetch, cookies }) => {
	const token = cookies.get('token');
	const headers = token ? { Authorization: `Bearer ${token}` } : {};

	try {
		const res = await fetch('/api/admin/users', { headers });
		if (res.ok) {
			const users = await res.json();
			return { users };
		}
	} catch (e) {
		console.error("Error loading users:", e);
	}
	return { users: [] };
};