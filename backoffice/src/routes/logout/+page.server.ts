import { redirect } from '@sveltejs/kit';
import type { Actions } from './$types';

export const actions = {
	default: async ({ cookies }) => {
		// Delete the HttpOnly cookie server-side
		cookies.delete('token', { path: '/' });
		throw redirect(302, '/login');
	}
} satisfies Actions;
