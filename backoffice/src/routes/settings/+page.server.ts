import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ fetch, cookies }) => {
	const token = cookies.get('token');
	const headers = token ? { Authorization: `Bearer ${token}` } : {};

	try {
		const res = await fetch('/api/admin/settings', { headers });
		if (res.ok) {
			const settings = await res.json();
			return { settings };
		}
	} catch (e) {
		console.error("Error loading settings:", e);
	}
	return { settings: null };
};