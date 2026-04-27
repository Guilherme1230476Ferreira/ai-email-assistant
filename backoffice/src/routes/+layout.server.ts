import type { LayoutServerLoad } from './$types';
import { redirect } from '@sveltejs/kit';

export const load: LayoutServerLoad = async ({ cookies, fetch, url }) => {
	const token = cookies.get('token');
	
	if (!token && !url.pathname.startsWith('/login')) {
		throw redirect(302, '/login');
	}

	if (!token) return { user: null };

	let sub = null;
	try {
		const payload = token.split('.')[1];
		// Base64 decode
		const decodedStr = Buffer.from(payload, 'base64').toString('utf-8');
		const decoded = JSON.parse(decodedStr);
		sub = decoded.sub;
	} catch (e) {
		if (!url.pathname.startsWith('/login')) throw redirect(302, '/login');
		return { user: null };
	}

	try {
		const headers = { 'Authorization': `Bearer ${token}` };
		// Attempt to grab all users to find 'me'
		const usersRes = await fetch('/api/admin/users', { headers });
		let userProfile = { email: "User", initials: "U", role: "user" };
		
		if (usersRes.ok) {
			const users = await usersRes.json();
			const me = users.find((u: any) => u.id === sub);
			if (me) {
				userProfile.email = me.email;
				userProfile.initials = me.email.substring(0, 2).toUpperCase();
			}
		}

		return { user: userProfile };
	} catch (err) {
		return { user: null };
	}
};
