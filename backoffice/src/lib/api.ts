import { get } from 'svelte/store';
import { authToken } from './stores/auth';

export type ApiRole = { id: string; name: string };
export type ApiUser = {
	id: string;
	email: string;
	role_id: string;
	password_hash: string;
	created_at: string;
};
export type ApiEmail = {
	id: string;
	user_id: string;
	original_content: string;
	generated_response: string | null;
	created_at: string;
	updated_at: string;
};
export type ApiSettings = {
	id: string;
	llm_base_url: string;
	llm_model: string;
	has_api_key: boolean;
	masked_api_key: string | null;
};
export type ApiTelemetry = {
	context_retrieval_rate: number;
	avg_similarity_score: number;
	chars_processed: number;
	knowledge_matches: number;
	kb_hit_rate: number;
};
export type ApiRagTraceItem = {
	source: string;
	title: string;
	text: string;
	score: number;
};
export type ApiAuditLog = {
	id: string;
	user_id: string | null;
	action: string;
	metadata: Record<string, any> | null;
	created_at: string;
};

export type ApiResult<T> = { data: T; error: null } | { data: null; error: string };

function authHeaders(): Record<string, string> {
	const token = get(authToken);
	const h: Record<string, string> = { 'Content-Type': 'application/json' };
	if (token) h['Authorization'] = `Bearer ${token}`;
	return h;
}

async function request<T>(method: string, path: string, body?: unknown): Promise<ApiResult<T>> {
	try {
		const res = await fetch(`/api${path}`, {
			method,
			headers: authHeaders(),
			...(body !== undefined ? { body: JSON.stringify(body) } : {})
		});
		if (!res.ok) {
			const json = await res.json().catch(() => ({}));
			return { data: null, error: (json as { error?: string }).error ?? `Error ${res.status}` };
		}
		if (res.status === 204) return { data: undefined as T, error: null };
		return { data: (await res.json()) as T, error: null };
	} catch {
		return { data: null, error: 'Could not reach the server.' };
	}
}

export const api = {
	me: () => request<ApiUser>('GET', '/auth/me'),
	getUsers: (page = 1, limit = 20) => request<{items: ApiUser[], total: number, page: number, limit: number}>('GET', `/admin/users?page=${page}&limit=${limit}`),
	getAuditLogs: (page = 1, limit = 20) => request<{items: ApiAuditLog[], total: number, page: number, limit: number}>('GET', `/admin/audit-logs?page=${page}&limit=${limit}`),
	createUser: (email: string, password: string) =>
		request<ApiUser>('POST', '/admin/users', { email, password }),
	deleteUser: (id: string) => request<void>('DELETE', `/admin/users/${id}`),
	updateUserRole: (userId: string, roleId: string) =>
		request<ApiUser>('PUT', `/admin/users/${userId}/role`, { role_id: roleId }),
	getRoles: () => request<ApiRole[]>('GET', '/admin/roles'),
	createRole: (name: string) => request<ApiRole>('POST', '/admin/roles', { name }),
	deleteRole: (id: string) => request<void>('DELETE', `/admin/roles/${id}`),
	generateEmail: (prompt: string) => request<ApiEmail>('POST', '/emails/generate', { prompt }),
	deleteEmail: (id: string) => request<void>('DELETE', `/emails/${id}`),
	getEmailTrace: (id: string) => request<ApiRagTraceItem[]>('GET', `/emails/${id}/trace`),
	
	// Streaming generation
	generateEmailStream: async (
		prompt: string,
		onToken: (token: string) => void,
		onError: (err: string) => void,
		onDone: () => void,
		onLog?: (msg: string) => void
	) => {
		const headers = authHeaders();

		try {
			const res = await fetch('/api/emails/generate/stream', {
				method: 'POST',
				headers,
				body: JSON.stringify({ prompt })
			});

			if (!res.ok) {
				const errBody = await res.text();
				onError(`Stream failed: ${res.status} - ${errBody}`);
				return;
			}

			const reader = res.body?.getReader();
			if (!reader) {
				onError('No response stream available');
				return;
			}

			const decoder = new TextDecoder('utf-8');
			let buffer = '';

			while (true) {
				const { done, value } = await reader.read();
				if (done) break;

				buffer += decoder.decode(value, { stream: true });

				// Process SSE events
				const lines = buffer.split('\n');
				buffer = lines.pop() || ''; // Keep incomplete line in buffer

				for (let i = 0; i < lines.length; i++) {
					const line = lines[i].trim();
					if (!line) continue;

					if (line.startsWith('event: ')) {
						const eventType = line.substring(7);
						const dataLine = lines[++i]?.trim() || '';
						
						if (dataLine.startsWith('data: ')) {
							const data = dataLine.substring(6);
							
							if (eventType === 'token') {
								onToken(data);
							} else if (eventType === 'log') {
								onLog?.(data);
							} else if (eventType === 'error') {
								onError(data);
							} else if (eventType === 'done') {
								onDone();
							}
						}
					} else if (line.startsWith('data: ')) {
						// Default event type (token)
						const data = line.substring(6);
						if (data === '[DONE]') {
							onDone();
						} else {
							onToken(data);
						}
					}
				}
			}
		} catch (err: any) {
			onError(err.message || String(err));
		}
	},
	
	updateSettings: (p: { llm_base_url: string; llm_model: string; llm_api_key?: string }) =>
		request<ApiSettings>('PUT', '/admin/settings', p),
	verifySettings: (p: { llm_base_url?: string; llm_model?: string; llm_api_key?: string }) =>
		request<{ status: string; message: string }>('POST', '/admin/settings/verify', p)
};
