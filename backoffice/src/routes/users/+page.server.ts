import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ fetch, cookies, url }) => {
	const token = cookies.get('token');
	const headers: Record<string, string> = token ? { Authorization: `Bearer ${token}` } : {};

	const page = url.searchParams.get('page') || '1';
	const limit = url.searchParams.get('limit') || '20';

	try {
		const [usersRes, rolesRes] = await Promise.all([
			fetch(`/api/admin/users?page=${page}&limit=${limit}`, { headers }),
			fetch('/api/admin/roles', { headers })
		]);
		
		const usersData = usersRes.ok ? await usersRes.json() : { items: [], page: 1, limit: 20, total: 0 };
		
		return {
			users: usersData.items,
			pagination: { page: usersData.page, limit: usersData.limit, total: usersData.total },
			roles: rolesRes.ok ? await rolesRes.json() : []
		};
	} catch (e) {
		console.error('Error loading users/roles:', e);
	}
	return { users: [], pagination: { page: 1, limit: 20, total: 0 }, roles: [] };
};