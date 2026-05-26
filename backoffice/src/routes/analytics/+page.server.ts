import type { PageServerLoad } from './$types';
import { redirect } from '@sveltejs/kit';

export const load: PageServerLoad = async ({ fetch, parent }) => {
	const parentData = await parent();
	if (!parentData.token) throw redirect(302, '/login');

	const headers = { Authorization: `Bearer ${parentData.token}` };

	const [statsRes, distRes, historyRes] = await Promise.allSettled([
		fetch('/api/knowledge/stats', { headers }),
		fetch('/api/telemetry/distribution', { headers }),
		fetch('/api/telemetry/history', { headers })
	]);

	const knowledgeStats =
		statsRes.status === 'fulfilled' && statsRes.value.ok ? await statsRes.value.json() : [];

	const distribution =
		distRes.status === 'fulfilled' && distRes.value.ok ? await distRes.value.json() : [];

	const history =
		historyRes.status === 'fulfilled' && historyRes.value.ok ? await historyRes.value.json() : [];

	return { knowledgeStats, distribution, history };
};
