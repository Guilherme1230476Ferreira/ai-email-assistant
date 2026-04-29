import { writable } from 'svelte/store';

/**
 * In-memory token store. Set by +layout.svelte from server data.
 * This keeps the token out of document.cookie (since the cookie is HttpOnly)
 * while still allowing client-side API calls.
 */
export const authToken = writable<string | null>(null);
