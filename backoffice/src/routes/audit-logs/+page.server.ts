import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ fetch, cookies, url }) => {
	const token = cookies.get('token');
	const headers: Record<string, string> = token ? { Authorization: `Bearer ${token}` } : {};

	const page = url.searchParams.get('page') || '1';
	const limit = url.searchParams.get('limit') || '20';

	try {
		const res = await fetch(`/api/admin/audit-logs?page=${page}&limit=${limit}`, { headers });
		
		if (res.ok) {
			const data = await res.json();
			return {
				logs: data.items,
				pagination: { page: data.page, limit: data.limit, total: data.total }
			};
		}
	} catch (e) {
		console.error('Error loading audit logs:', e);
	}
	return { logs: [], pagination: { page: 1, limit: 20, total: 0 } };
};
