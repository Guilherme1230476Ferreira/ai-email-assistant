import adapter from '@sveltejs/adapter-node';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	compilerOptions: {
		// Force runes mode for the project, except for libraries. Can be removed in svelte 6.
		runes: ({ filename }) => (filename.split(/[/\\]/).includes('node_modules') ? undefined : true)
	},
	kit: {
		// Disable CSRF origin check — safe because:
		// 1. Nginx is the only ingress (port 3001 is not exposed externally)
		// 2. All state-changing API calls use JWT auth enforced by the Rust backend
		// 3. The ISEP gateway rewrites Host/port making exact origin matching impossible
		csrf: { checkOrigin: false },
		adapter: adapter({
			// Output directory for the Node.js server build
			out: 'build'
		})
	}
};

export default config;
