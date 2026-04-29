import { describe, it, expect, vi, afterEach } from 'vitest';
import { getToken } from './token';

describe('token.ts', () => {
	afterEach(() => {
		vi.restoreAllMocks();
	});

	describe('getToken()', () => {
		it('returns null if there are no cookies', () => {
			vi.spyOn(document, 'cookie', 'get').mockReturnValue('');
			expect(getToken()).toBeNull();
		});

		it('returns the token when it is the only cookie', () => {
			vi.spyOn(document, 'cookie', 'get').mockReturnValue('token=my-secret-jwt-token');
			expect(getToken()).toBe('my-secret-jwt-token');
		});

		it('returns the token when it is among other cookies', () => {
			vi.spyOn(document, 'cookie', 'get').mockReturnValue('foo=bar; token=my-secret-jwt-token; baz=qux');
			expect(getToken()).toBe('my-secret-jwt-token');
		});

		it('returns null if the token cookie is not present', () => {
			vi.spyOn(document, 'cookie', 'get').mockReturnValue('foo=bar; baz=qux');
			expect(getToken()).toBeNull();
		});
	});
});
