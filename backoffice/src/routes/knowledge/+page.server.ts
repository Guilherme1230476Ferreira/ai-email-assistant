import { redirect } from '@sveltejs/kit';

export const load = async ({ cookies, fetch }: { cookies: any; fetch: typeof globalThis.fetch }) => {
	const token = cookies.get('token');
	if (!token) throw redirect(302, '/login');

	const res = await fetch('/api/knowledge?page=1&per_page=50', {
		headers: { Authorization: `Bearer ${token}` }
	});

	if (!res.ok) {
		return { entries: [], total: 0 };
	}

	const data = await res.json();
	return {
		entries: data.items || [],
		total: data.total || 0
	};
};
