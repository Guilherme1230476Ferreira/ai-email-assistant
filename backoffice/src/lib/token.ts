/**
 * Reads the JWT auth token from the browser cookie.
 * Single source of truth — import this instead of copy-pasting the regex.
 */
export function getToken(): string | null {
	if (typeof document === 'undefined') return null;
	const match = document.cookie.match(/(^| )token=([^;]+)/);
	return match ? match[2] : null;
}
