<script lang="ts">
  import { Mail, Users, Shield, Cpu, ArrowUpRight, Check } from '@lucide/svelte';
  import type { PageData } from './$types';

  let { data }: { data: PageData } = $props();

  let stats = $derived([
    { label: 'Emails generated', value: data.stats.emailsCount.toString(), icon: Mail, href: '/emails' },
    { label: 'Users', value: data.stats.usersCount.toString(), icon: Users, href: '/users' },
    { label: 'Roles', value: data.stats.rolesCount.toString(), icon: Shield, href: '/roles' },
    { label: 'LLM model', value: data.llm?.llm_model || 'Not set', icon: Cpu, href: '/settings' }
  ]);

  let llmConfig = $derived({
    model: data.llm?.llm_model || 'Unknown',
    base_url: data.llm?.llm_base_url || 'Unknown',
    has_api_key: !!data.llm?.has_api_key,
    embedding_model: 'text-embedding-3-small' 
  });
</script>

<svelte:head>
  <title>Dashboard · Mailwise</title>
</svelte:head>

<div class="mx-auto w-full max-w-5xl space-y-10">
  <header class="flex flex-col gap-1">
    <h1 class="text-2xl font-semibold tracking-tight text-white">Dashboard</h1>
    <p class="text-sm text-[var(--color-muted-foreground)]">
      Overview of your AI email assistant.
    </p>
  </header>

  <section class="grid grid-cols-2 gap-3 lg:grid-cols-4">
    {#each stats as s (s.label)}
      <a
        href={s.href}
        class="group flex flex-col justify-between rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] p-4 transition hover:border-[var(--color-border-strong)]"
      >
        <div class="flex items-center justify-between">
          <span class="text-xs text-[var(--color-muted-foreground)]">{s.label}</span>
          <s.icon class="h-4 w-4 text-[var(--color-muted)]" />
        </div>
        <div class="mt-4 flex items-end justify-between">
          <span class="text-2xl font-semibold tracking-tight text-white">{s.value}</span>
          <ArrowUpRight
            class="h-4 w-4 text-[var(--color-muted)] transition group-hover:text-[var(--color-accent)]"
          />
        </div>
      </a>
    {/each}
  </section>

  <section class="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)]">
    <div class="flex items-center justify-between border-b border-[var(--color-border)] px-5 py-4">
      <div>
        <h2 class="text-sm font-medium text-white">LLM configuration</h2>
        <p class="text-xs text-[var(--color-muted-foreground)]">
          Used by <code class="text-[var(--color-muted-foreground)]">/api/emails/generate</code>
        </p>
      </div>
      <a
        href="/settings"
        class="rounded-md border border-[var(--color-border)] px-3 py-1.5 text-xs font-medium text-white transition hover:border-[var(--color-accent)]/40 hover:text-[var(--color-accent)]"
      >
        Manage
      </a>
    </div>

    <dl
      class="grid grid-cols-1 divide-y divide-[var(--color-border)] sm:grid-cols-2 sm:divide-x sm:divide-y-0"
    >
      <div class="px-5 py-4">
        <dt class="text-[11px] uppercase tracking-wider text-[var(--color-muted)]">Model</dt>
        <dd class="mt-1 font-mono text-sm text-white">{llmConfig.model}</dd>
      </div>
      <div class="px-5 py-4">
        <dt class="text-[11px] uppercase tracking-wider text-[var(--color-muted)]">Base URL</dt>
        <dd class="mt-1 truncate font-mono text-sm text-white">{llmConfig.base_url}</dd>
      </div>
      <div class="px-5 py-4">
        <dt class="text-[11px] uppercase tracking-wider text-[var(--color-muted)]">API key</dt>
        <dd class="mt-1 flex items-center gap-2 text-sm text-white">
          {#if llmConfig.has_api_key}
            <span
              class="inline-flex h-4 w-4 items-center justify-center rounded-full bg-[var(--color-success)]/15 text-[var(--color-success)]"
            >
              <Check class="h-3 w-3" strokeWidth={3} />
            </span>
            <span>Configured</span>
          {:else}
            <span class="text-[var(--color-warning)]">Not set</span>
          {/if}
        </dd>
      </div>
      <div class="px-5 py-4">
        <dt class="text-[11px] uppercase tracking-wider text-[var(--color-muted)]">
          Embedding model
        </dt>
        <dd class="mt-1 font-mono text-sm text-white">{llmConfig.embedding_model}</dd>
      </div>
    </dl>
  </section>

  <section class="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)]">
    <div class="flex items-center justify-between border-b border-[var(--color-border)] px-5 py-4">
      <div>
        <h2 class="text-sm font-medium text-white">Recent generations</h2>
        <p class="text-xs text-[var(--color-muted-foreground)]">Latest AI-drafted replies.</p>
      </div>
      <a href="/emails" class="text-xs font-medium text-[var(--color-accent)] hover:underline">
        View all
      </a>
    </div>

    <ul class="divide-y divide-[var(--color-border)]">
      {#each data.recentEmails as email (email.id)}
        <li class="px-5 py-4 transition hover:bg-[var(--color-surface-2)]/50">
          <div class="flex items-start justify-between gap-4">
            <div class="min-w-0 flex-1 space-y-1.5">
              <p class="line-clamp-1 text-sm text-white">{email.original_content || "No prompt provided"}</p>
              <p class="line-clamp-1 text-xs text-[var(--color-muted-foreground)]">
                <span class="text-[var(--color-accent)]">↳</span>
                {email.generated_response || "Empty response"}
              </p>
            </div>
            <span class="shrink-0 text-[11px] text-[var(--color-muted)]">
              {new Date(email.created_at || Date.now()).toLocaleDateString()}
            </span>
          </div>
        </li>
      {:else}
        <li class="px-5 py-10 text-center text-sm text-[var(--color-muted-foreground)]">
          No emails generated yet.
        </li>
      {/each}
    </ul>
  </section>
</div>
