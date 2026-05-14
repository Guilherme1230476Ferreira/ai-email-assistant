import type { LayoutServerLoad } from './$types';
import { redirect } from '@sveltejs/kit';

export const load: LayoutServerLoad = async ({ cookies, fetch, url }) => {
	const token = cookies.get('token');
	const isPublic =
		url.pathname.startsWith('/login') || url.pathname.startsWith('/signup');

	if (!token) {
		if (!isPublic) throw redirect(302, '/login');
		return { user: null, token: null };
	}

	try {
		// Validate token server-side via /api/auth/me (backend verifies JWT signature)
		const res = await fetch('/api/auth/me', {
			headers: { Authorization: `Bearer ${token}` }
		});

		if (!res.ok) {
			if (!isPublic) throw redirect(302, '/login');
			return { user: null, token: null };
		}

		const me = await res.json();

		// Resolve the role name from role_id by fetching roles list
		let roleName = 'user';
		try {
			const rolesRes = await fetch('/api/admin/roles', {
				headers: { Authorization: `Bearer ${token}` }
			});
			if (rolesRes.ok) {
				const roles = await rolesRes.json();
				const match = roles.find((r: any) => r.id === me.role_id);
				if (match) roleName = match.name.toLowerCase();
			}
		} catch {
			// If roles fetch fails (non-admin user), default to 'user'
		}

		return {
			token,
			user: {
				email: me.email as string,
				initials: (me.email as string).substring(0, 2).toUpperCase(),
				role: roleName
			}
		};
	} catch (e) {
		if (e && typeof e === 'object' && 'status' in e) throw e;
		return { user: null, token: null };
	}
};
