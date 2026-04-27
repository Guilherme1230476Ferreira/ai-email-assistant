<script lang="ts">
  import type { PageData } from './$types';

  let { data }: { data: PageData } = $props();
  
  let model = $state(data.settings?.llm_model || '');
  let baseUrl = $state(data.settings?.llm_base_url || '');
  let apiKey = $state('');
</script>

<svelte:head>
  <title>Settings · Mailwise</title>
</svelte:head>

<div class="mx-auto w-full max-w-3xl space-y-8">
  <header class="flex flex-col gap-1">
    <h1 class="text-2xl font-semibold tracking-tight text-white">Settings</h1>
    <p class="text-sm text-[var(--color-muted-foreground)]">Manage your LLM configurations and models.</p>
  </header>

  <div class="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] p-6">
    <div class="space-y-4">
      <div>
        <label for="model" class="mb-1.5 block text-xs font-medium text-[var(--color-muted-foreground)]">LLM Model</label>
        <input id="model" type="text" bind:value={model} class="w-full rounded-md border border-[var(--color-border)] bg-[var(--color-surface-2)] py-2 px-3 text-sm text-white focus:border-[var(--color-accent)] focus:outline-none" />
      </div>

      <div>
        <label for="baseUrl" class="mb-1.5 block text-xs font-medium text-[var(--color-muted-foreground)]">Base URL</label>
        <input id="baseUrl" type="text" bind:value={baseUrl} class="w-full rounded-md border border-[var(--color-border)] bg-[var(--color-surface-2)] py-2 px-3 text-sm text-white focus:border-[var(--color-accent)] focus:outline-none" />
      </div>

      <div>
        <label for="apiKey" class="mb-1.5 block text-xs font-medium text-[var(--color-muted-foreground)]">API Key</label>
        <input id="apiKey" type="password" bind:value={apiKey} placeholder={data.settings?.has_api_key ? '••••••••' : 'Enter API Key'} class="w-full rounded-md border border-[var(--color-border)] bg-[var(--color-surface-2)] py-2 px-3 text-sm text-white focus:border-[var(--color-accent)] focus:outline-none" />
      </div>

      <button class="rounded-md bg-[var(--color-accent)] px-4 py-2 text-sm font-semibold text-black hover:bg-[var(--color-accent-hover)]">
        Save Settings
      </button>
    </div>
  </div>
</div>