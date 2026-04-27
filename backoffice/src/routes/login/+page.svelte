<script lang="ts">
  import { Sparkles, Mail, Lock, ArrowRight, Loader2 } from '@lucide/svelte';

  let email = $state('');
  let password = $state('');
  let loading = $state(false);
  let error = $state<string | null>(null);

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

    <form
      onsubmit={handleSubmit}
      class="rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] p-8 shadow-2xl shadow-black/50"
    >
      {#if error}
        <div
          class="mb-6 rounded-md border border-red-500/30 bg-red-500/5 px-4 py-3 text-sm text-red-300"
        >
          {error}
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

    <p class="mt-8 text-center text-sm text-[var(--color-muted)]">
      Don&apos;t have an account?
      <a href="/signup" class="font-medium text-white hover:text-[var(--color-accent)]">Create one</a>
    </p>
  </div>
</div>
