<script lang="ts">
  import { Sparkles, Mail, Lock, ArrowRight, Loader2 } from '@lucide/svelte';

  import { page } from '$app/state';

  let email = $state('');
  let password = $state('');
  let loading = $state(false);
  let error = $state<string | null>(null);
  let success = $state<string | null>(null);

  $effect(() => {
     let searchParams = page.url.searchParams;
     if (searchParams.get('error') === 'not_registered') {
        error = "User not registered yet. Please create an account first.";
     }
     if (searchParams.get('success') === 'registered') {
        success = "Registration successful! You can now sign in.";
     }
  });

  function handleGoogleLogin() {
    window.location.href = '/api/auth/google?intent=login';
  }

  async function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    error = null;
    loading = true;
    try {
      // POST /api/auth/login -> { token }
      const res = await fetch('/api/auth/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email, password })
      });
      if (!res.ok) {
        error = 'Invalid email or password.';
        return;
      }
      const data = await res.json();
      document.cookie = `token=${data.token}; path=/; max-age=86400; samesite=lax`;
      window.location.href = '/';
    } catch {
      error = 'Could not reach the server.';
    } finally {
      loading = false;
    }
  }
</script>

<div class="relative flex min-h-screen items-center justify-center px-4 py-12">
  <div
    class="pointer-events-none absolute inset-0 bg-[radial-gradient(ellipse_at_top,_rgba(34,211,238,0.08),_transparent_60%)]"
  ></div>

  <div class="relative w-full max-w-md">
    <div class="mb-10 flex flex-col items-center text-center">
      <div
        class="mb-6 flex h-14 w-14 items-center justify-center rounded-lg bg-[var(--color-accent)] text-black shadow-lg"
      >
        <Sparkles class="h-8 w-8" strokeWidth={2.5} />
      </div>
      <h1 class="text-3xl font-bold tracking-tight text-white">Sign in to Mailwise</h1>
      <p class="mt-2.5 text-base text-[var(--color-muted)]">
        Welcome back. Enter your details to continue.
      </p>
    </div>

    <div class="rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] p-8 shadow-2xl shadow-black/50">
      
      <!-- Gmail / OAuth Option -->
      <button
        type="button"
        onclick={handleGoogleLogin}
        class="mb-6 flex w-full items-center justify-center gap-3 rounded-lg border border-[var(--color-border)] bg-[var(--color-surface-2)] px-4 py-3 text-sm font-medium text-white transition hover:bg-[var(--color-surface)] hover:text-[var(--color-accent)]"
      >
        <svg class="h-5 w-5" viewBox="0 0 24 24">
          <path d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z" fill="#4285F4"/>
          <path d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z" fill="#34A853"/>
          <path d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z" fill="#FBBC05"/>
          <path d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z" fill="#EA4335"/>
        </svg>
        Sign in with Google
      </button>

      <div class="relative mb-6 flex items-center py-2">
        <div class="flex-grow border-t border-[var(--color-border)]"></div>
        <span class="mx-4 flex-shrink text-xs text-[var(--color-muted-foreground)] uppercase">Or continue with email</span>
        <div class="flex-grow border-t border-[var(--color-border)]"></div>
      </div>

      <form onsubmit={handleSubmit}>
        {#if error}
          <div
            class="mb-6 rounded-md border border-red-500/30 bg-red-500/5 px-4 py-3 text-sm text-red-300"
          >
            {error}
            {#if error.includes("User not registered")}
              <a href="/signup" class="font-bold underline ml-1 text-white hover:text-[var(--color-accent)]">Create one &rarr;</a>
            {/if}
          </div>
        {/if}
        {#if success}
          <div
            class="mb-6 rounded-md border border-[#34A853]/30 bg-[#34A853]/5 px-4 py-3 text-sm text-[#34A853]"
          >
            {success}
          </div>
        {/if}

      <div class="space-y-6">
        <div>
          <label for="email" class="mb-2 block text-sm font-medium text-[var(--color-muted-foreground)]">
            Email
          </label>
          <div class="relative">
            <Mail
              class="pointer-events-none absolute left-4 top-1/2 h-5 w-5 -translate-y-1/2 text-[var(--color-muted)]"
            />
            <input
              id="email"
              type="email"
              required
              autocomplete="email"
              bind:value={email}
              placeholder="you@company.com"
              class="w-full rounded-lg border border-[var(--color-border)] bg-[var(--color-surface-2)] py-3 pl-12 pr-4 text-base text-white placeholder:text-[var(--color-muted)] focus:border-[var(--color-accent)] focus:outline-none focus:ring-1 focus:ring-[var(--color-accent)]"
            />
          </div>
        </div>

        <div>
          <div class="mb-2 flex items-center justify-between">
            <label for="password" class="block text-sm font-medium text-[var(--color-muted-foreground)]">
              Password
            </label>
            <a href="/forgot" class="text-sm font-medium text-[var(--color-accent)] hover:underline">Forgot password?</a>
          </div>
          <div class="relative">
            <Lock
              class="pointer-events-none absolute left-4 top-1/2 h-5 w-5 -translate-y-1/2 text-[var(--color-muted)]"
            />
            <input
              id="password"
              type="password"
              required
              autocomplete="current-password"
              bind:value={password}
              placeholder="••••••••"
              class="w-full rounded-lg border border-[var(--color-border)] bg-[var(--color-surface-2)] py-3 pl-12 pr-4 text-base text-white placeholder:text-[var(--color-muted)] focus:border-[var(--color-accent)] focus:outline-none focus:ring-1 focus:ring-[var(--color-accent)]"
            />
          </div>
        </div>
      </div>

        <button
          type="submit"
          disabled={loading}
          class="mt-8 inline-flex w-full items-center justify-center gap-2 rounded-lg bg-[var(--color-accent)] px-4 py-3.5 text-base font-bold text-black transition hover:bg-[var(--color-accent-hover)] disabled:cursor-not-allowed disabled:opacity-60"
        >
          {#if loading}
            <Loader2 class="h-5 w-5 animate-spin" />
            <span>Signing in...</span>
          {:else}
            <span>Sign in</span>
            <ArrowRight class="h-5 w-5" />
          {/if}
        </button>
      </form>
    </div>

    <p class="mt-8 text-center text-sm text-[var(--color-muted)]">
      Don&apos;t have an account?
      <a href="/signup" class="font-medium text-white hover:text-[var(--color-accent)]">Create one</a>
    </p>
  </div>
</div>
