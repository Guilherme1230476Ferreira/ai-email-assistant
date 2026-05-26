<script lang="ts">
  import { enhance } from '$app/forms';
  import { Sparkles, Mail, Lock, ArrowRight, Loader2 } from '@lucide/svelte';
  import { page } from '$app/state';
  import type { ActionData } from './$types';
  import { locale, t, type Locale } from '$lib/i18n';

  let { form }: { form: ActionData } = $props();

  let currentLocale = $state<Locale>($locale);
  locale.subscribe((val) => (currentLocale = val));

  let loading = $state(false);

  // URL params from Google OAuth redirects
  let urlError = $derived(page.url.searchParams.get('error'));
  let urlSuccess = $derived(page.url.searchParams.get('success'));

  let errorMessage = $derived(
    form?.error ??
    (urlError === 'not_registered' ? 'User not registered. Please create an account first.' : null)
  );
  let successMessage = $derived(
    urlSuccess === 'registered' ? 'Registration successful! You can now sign in.' : null
  );

  function handleGoogleLogin() {
    window.location.href = '/api/auth/google?intent=login';
  }
</script>

<div class="relative flex min-h-screen items-center justify-center px-4 py-12">
  <div class="pointer-events-none absolute inset-0 bg-[radial-gradient(ellipse_at_top,_rgba(34,211,238,0.08),_transparent_60%)]"></div>

  <div class="relative w-full max-w-md">
    <div class="mb-10 flex flex-col items-center text-center">
      <div class="mb-6 flex h-14 w-14 items-center justify-center rounded-lg bg-[var(--color-accent)] text-black shadow-lg">
        <Sparkles class="h-8 w-8" strokeWidth={2.5} />
      </div>
      <h1 class="text-3xl font-bold tracking-tight text-white">{t('login.title', currentLocale)}</h1>
      <p class="mt-2.5 text-base text-[var(--color-muted)]">Welcome back. Enter your details to continue.</p>
    </div>

    <div class="rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] p-8 shadow-2xl shadow-black/50">

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
        <span class="mx-4 flex-shrink text-xs uppercase text-[var(--color-muted-foreground)]">{t('login.or', currentLocale)}</span>
        <div class="flex-grow border-t border-[var(--color-border)]"></div>
      </div>

      <!-- Server-action form — no client-side fetch needed -->
      <form method="POST" use:enhance={() => {
        loading = true;
        return async ({ update }) => { loading = false; await update(); };
      }}>

        {#if errorMessage}
          <div class="mb-6 rounded-md border border-red-500/30 bg-red-500/5 px-4 py-3 text-sm text-red-300">
            {errorMessage}
            {#if urlError === 'not_registered'}
              <a href="/signup" class="ml-1 font-bold underline text-white hover:text-[var(--color-accent)]">Create one &rarr;</a>
            {/if}
          </div>
        {/if}

        {#if successMessage}
          <div class="mb-6 rounded-md border border-emerald-500/30 bg-emerald-500/5 px-4 py-3 text-sm text-emerald-300">
            {successMessage}
          </div>
        {/if}

        <div class="space-y-6">
          <div>
            <label for="email" class="mb-2 block text-sm font-medium text-[var(--color-muted-foreground)]">{t('login.email', currentLocale)}</label>
            <div class="relative">
              <Mail class="pointer-events-none absolute left-4 top-1/2 h-5 w-5 -translate-y-1/2 text-[var(--color-muted)]" />
              <input id="email" name="email" type="email" required autocomplete="email" placeholder="you@company.com"
                class="w-full rounded-lg border border-[var(--color-border)] bg-[var(--color-surface-2)] py-3 pl-12 pr-4 text-base text-white placeholder:text-[var(--color-muted)] focus:border-[var(--color-accent)] focus:outline-none focus:ring-1 focus:ring-[var(--color-accent)]" />
            </div>
          </div>

          <div>
            <div class="mb-2 flex items-center justify-between">
              <label for="password" class="block text-sm font-medium text-[var(--color-muted-foreground)]">{t('login.password', currentLocale)}</label>
            </div>
            <div class="relative">
              <Lock class="pointer-events-none absolute left-4 top-1/2 h-5 w-5 -translate-y-1/2 text-[var(--color-muted)]" />
              <input id="password" name="password" type="password" required autocomplete="current-password" placeholder="••••••••"
                class="w-full rounded-lg border border-[var(--color-border)] bg-[var(--color-surface-2)] py-3 pl-12 pr-4 text-base text-white placeholder:text-[var(--color-muted)] focus:border-[var(--color-accent)] focus:outline-none focus:ring-1 focus:ring-[var(--color-accent)]" />
            </div>
          </div>
        </div>

        <button type="submit" disabled={loading}
          class="mt-8 inline-flex w-full items-center justify-center gap-2 rounded-lg bg-[var(--color-accent)] px-4 py-3.5 text-base font-bold text-black transition hover:bg-[var(--color-accent-hover)] disabled:cursor-not-allowed disabled:opacity-60">
          {#if loading}
            <Loader2 class="h-5 w-5 animate-spin" /> {t('login.signing_in', currentLocale)}
          {:else}
            {t('login.submit', currentLocale)} <ArrowRight class="h-5 w-5" />
          {/if}
        </button>
      </form>
    </div>

    <p class="mt-8 text-center text-sm text-[var(--color-muted)]">
      {t('login.no_account', currentLocale)}
      <a href="/signup" class="font-medium text-white hover:text-[var(--color-accent)]">{t('login.register', currentLocale)}</a>
    </p>

    <!-- Copyright + Privacy Policy -->
    <p class="mt-6 text-center text-xs text-[var(--color-muted)]/60 leading-relaxed">
      © {new Date().getFullYear()} AI Email Assistant — ISEP.<br />
      By signing in you agree to our
      <a href="/privacy" class="text-[var(--color-accent)]/80 hover:text-[var(--color-accent)] underline underline-offset-2 transition-colors">Privacy Policy</a>.
      Replies are generated by AI and must be reviewed before sending.
    </p>
  </div>
</div>
