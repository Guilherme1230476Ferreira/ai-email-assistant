import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ fetch, cookies }) => {
	const token = cookies.get('token');
	const headers = token ? { Authorization: `Bearer ${token}` } : {};

	try {
		const res = await fetch('/api/emails', { headers });
		if (res.ok) {
			const emails = await res.json();
			return { emails };
		}
	} catch (e) {
		console.error("Error loading emails:", e);
	}
	return { emails: [] };
};