import type { PageServerLoad } from './$types';
import { redirect } from '@sveltejs/kit';

export const load: PageServerLoad = async ({ parent }) => {
    const { user, token } = await parent();
    if (!user) throw redirect(303, '/login');
    return { user, token };
};
