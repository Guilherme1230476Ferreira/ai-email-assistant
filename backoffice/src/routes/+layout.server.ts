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
		return {
			// Pass the token value to the client so it can be stored in the
			// in-memory auth store and used for client-side API calls.
			token,
			user: {
				email: me.email as string,
				initials: (me.email as string).substring(0, 2).toUpperCase(),
				role: 'user' as string
			}
		};
	} catch (e) {
		if (e && typeof e === 'object' && 'status' in e) throw e;
		return { user: null, token: null };
	}
};
