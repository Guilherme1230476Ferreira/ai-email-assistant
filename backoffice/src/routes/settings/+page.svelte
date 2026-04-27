<script lang="ts">
  import type { PageData } from './$types';
  import { Save, Loader2 } from '@lucide/svelte';
  import { invalidateAll } from '$app/navigation';

  let { data }: { data: PageData } = $props();
  
  let model = $state('');
  let baseUrl = $state('');
  let apiKey = $state('');
  
  $effect(() => {
    // Safely sync the initial prop data without raising the Svelte 5 locality warning
    if (data.settings) {
      if (!model && data.settings.llm_model) model = data.settings.llm_model;
      if (!baseUrl && data.settings.llm_base_url) baseUrl = data.settings.llm_base_url;
    }
  });
  
  let isSaving = $state(false);
  let isTesting = $state(false);

  function getToken() {
    const match = document.cookie.match(/(^| )token=([^;]+)/);
    return match ? match[2] : null;
  }

  async function handleSave() {
    isSaving = true;
    try {
      const res = await fetch('/api/admin/settings', {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${getToken()}`
        },
        body: JSON.stringify({
          llm_model: model,
          llm_base_url: baseUrl,
          ...(apiKey ? { llm_api_key: apiKey } : {})
        })
      });

      if (res.ok) {
        apiKey = '';
        await invalidateAll();
        alert('Settings saved successfully');
      } else {
        alert('Failed to save settings');
      }
    } catch (err) {
      console.error(err);
      alert('An error occurred while saving.');
    } finally {
      isSaving = false;
    }
  }

  async function handleTest() {
    isTesting = true;
    try {
      const res = await fetch('/api/admin/settings/verify', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${getToken()}`
        },
        body: JSON.stringify({
          llm_model: model,
          llm_base_url: baseUrl,
          ...(apiKey ? { llm_api_key: apiKey } : {})
        })
      });

      if (res.ok) {
        alert('Connection test successful!');
      } else {
        const errorData = await res.json().catch(() => ({}));
        alert(`Test failed: ${errorData.message || res.statusText || 'Unknown error'}`);
      }
    } catch (err) {
      console.error(err);
      alert('An error occurred during the test.');
    } finally {
      isTesting = false;
    }
  }
</script>

<svelte:head>
  <title>Settings · Mailwise</title>
</svelte:head>

<div class="mx-auto w-full max-w-3xl space-y-8">
  <header class="flex flex-col gap-1">
    <h1 class="text-2xl font-semibold tracking-tight text-white">Settings</h1>
    <p class="text-sm text-[var(--color-muted-foreground)]">Configure the language model used for replies.</p>
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

      <div class="flex items-center justify-end gap-3 pt-2">
        <button 
          onclick={handleTest}
          disabled={isTesting}
          class="flex items-center justify-center gap-2 rounded-md border border-[var(--color-border)] bg-transparent px-4 py-2 text-sm font-semibold text-white transition hover:bg-[var(--color-surface-2)] disabled:opacity-50"
        >
          {#if isTesting}
            <Loader2 class="h-4 w-4 animate-spin" />
            Testing...
          {:else}
            Test connection
          {/if}
        </button>
        
        <button 
          onclick={handleSave}
          disabled={isSaving}
          class="flex items-center justify-center gap-2 rounded-md bg-[#2dd4bf] px-4 py-2 text-sm font-semibold text-black transition hover:bg-[#14b8a6] disabled:opacity-50"
        >
          {#if isSaving}
            <Loader2 class="h-4 w-4 animate-spin" />
            Saving...
          {:else}
            <Save class="h-4 w-4" />
            Save changes
          {/if}
        </button>
      </div>
    </div>
  </div>
</div>