import { fail, redirect } from '@sveltejs/kit';
import type { Actions } from './$types';

export const actions = {
	default: async ({ request, cookies, fetch }) => {
		const form = await request.formData();
		const email = form.get('email') as string;
		const password = form.get('password') as string;

		if (!email || !password) {
			return fail(400, { error: 'Email and password are required.' });
		}

		let token: string;
		try {
			const res = await fetch('/api/auth/login', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ email, password })
			});

			if (!res.ok) {
				return fail(401, { error: 'Invalid email or password.' });
			}

			const body = await res.json();
			token = body.token;
		} catch {
			return fail(500, { error: 'Could not reach the server.' });
		}

		// Set HttpOnly cookie — JS cannot read this via document.cookie
		cookies.set('token', token, {
			httpOnly: true,
			path: '/',
			maxAge: 86400,
			sameSite: 'lax',
			secure: false // set true in production (HTTPS)
		});

		throw redirect(302, '/');
	}
} satisfies Actions;
