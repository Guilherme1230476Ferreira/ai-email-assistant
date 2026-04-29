import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { api } from './api';
import { authToken } from './stores/auth';

// Mock the global fetch
const fetchMock = vi.fn();
global.fetch = fetchMock;

describe('api.ts', () => {
	beforeEach(() => {
		fetchMock.mockReset();
		authToken.set(null); // Clear auth token before each test
	});

	afterEach(() => {
		vi.clearAllMocks();
	});

	describe('authHeaders', () => {
		it('sends correct headers without token', async () => {
			fetchMock.mockResolvedValueOnce({
				ok: true,
				status: 200,
				json: async () => ({ id: '1' })
			});

			await api.me();

			expect(fetchMock).toHaveBeenCalledWith('/api/auth/me', {
				method: 'GET',
				headers: { 'Content-Type': 'application/json' }
			});
		});

		it('sends correct headers with token', async () => {
			authToken.set('fake-jwt-token');

			fetchMock.mockResolvedValueOnce({
				ok: true,
				status: 200,
				json: async () => ({ id: '1' })
			});

			await api.me();

			expect(fetchMock).toHaveBeenCalledWith('/api/auth/me', {
				method: 'GET',
				headers: {
					'Content-Type': 'application/json',
					Authorization: 'Bearer fake-jwt-token'
				}
			});
		});
	});

	describe('request error handling', () => {
		it('handles network errors gracefully', async () => {
			fetchMock.mockRejectedValueOnce(new Error('Network failure'));

			const result = await api.me();

			expect(result).toEqual({ data: null, error: 'Could not reach the server.' });
		});

		it('handles API errors gracefully', async () => {
			fetchMock.mockResolvedValueOnce({
				ok: false,
				status: 401,
				json: async () => ({ error: 'Unauthorized access' })
			});

			const result = await api.me();

			expect(result).toEqual({ data: null, error: 'Unauthorized access' });
		});

		it('handles missing error message in API response gracefully', async () => {
			fetchMock.mockResolvedValueOnce({
				ok: false,
				status: 500,
				json: async () => ({})
			});

			const result = await api.me();

			expect(result).toEqual({ data: null, error: 'Error 500' });
		});
	});

	describe('endpoints', () => {
		it('createUser sends correct body', async () => {
			fetchMock.mockResolvedValueOnce({
				ok: true,
				status: 201,
				json: async () => ({ id: '2', email: 'test@example.com' })
			});

			await api.createUser('test@example.com', 'password123');

			expect(fetchMock).toHaveBeenCalledWith('/api/admin/users', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ email: 'test@example.com', password: 'password123' })
			});
		});

		it('deleteUser expects 204 No Content', async () => {
			fetchMock.mockResolvedValueOnce({
				ok: true,
				status: 204
			});

			const result = await api.deleteUser('123');

			expect(fetchMock).toHaveBeenCalledWith('/api/admin/users/123', {
				method: 'DELETE',
				headers: { 'Content-Type': 'application/json' }
			});
			expect(result).toEqual({ data: undefined, error: null });
		});
	});
});
