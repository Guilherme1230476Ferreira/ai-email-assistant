import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ fetch, cookies }) => {
	const token = cookies.get('token');
	const headers = token ? { Authorization: `Bearer ${token}` } : {};

	try {
		const res = await fetch('/api/admin/roles', { headers });
		if (res.ok) {
			const roles = await res.json();
			return { roles };
		}
	} catch (e) {
		console.error("Error loading roles:", e);
	}
	return { roles: [] };
};