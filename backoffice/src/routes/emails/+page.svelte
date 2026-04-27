<script lang="ts">
  import type { PageData } from './$types';
  import { invalidateAll } from '$app/navigation';
  import { Sparkles, Search, Loader2, X } from '@lucide/svelte';

  let { data }: { data: PageData } = $props();

  let searchQuery = $state('');
  let isModalOpen = $state(false);
  let promptText = $state('');
  let isGenerating = $state(false);

  function getToken() {
    const match = document.cookie.match(/(^| )token=([^;]+)/);
    return match ? match[2] : null;
  }

  async function handleGenerate(e: Event) {
    e.preventDefault();
    if (!promptText.trim()) return;
    
    isGenerating = true;
    try {
      const res = await fetch('/api/emails/generate', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${getToken()}`
        },
        body: JSON.stringify({ prompt: promptText })
      });

      if (res.ok) {
        promptText = '';
        isModalOpen = false;
        await invalidateAll(); // Refresh data!
      } else {
        console.error('Failed to generate.');
      }
    } catch (err) {
      console.error(err);
    } finally {
      isGenerating = false;
    }
  }

  let filteredEmails = $derived(
    (data.emails || []).filter((email: any) => {
      const q = searchQuery.toLowerCase();
      return (
        email.original_content?.toLowerCase().includes(q) ||
        email.generated_response?.toLowerCase().includes(q)
      );
    })
  );
</script>

<svelte:head>
  <title>Emails · Mailwise</title>
</svelte:head>

<div class="mx-auto w-full max-w-5xl space-y-8">
  <header class="flex flex-col gap-4 mb-8">
    <div>
      <h1 class="text-2xl font-semibold tracking-tight text-whiteStr">Emails</h1>
      <p class="text-sm text-[var(--color-muted-foreground)]">Generated replies and incoming messages.</p>
    </div>

    <div class="flex flex-col sm:flex-row items-center gap-4">
      <div class="relative flex-1 w-full">
        <Search class="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-[var(--color-muted)]" />
        <input
          type="text"
          bind:value={searchQuery}
          placeholder="Search by sender or subject"
          class="w-full rounded-md border border-[var(--color-border)] bg-[var(--color-surface-2)] py-2.5 pl-9 pr-4 text-sm text-white placeholder:text-[var(--color-muted)] focus:border-[var(--color-accent)] focus:outline-none focus:ring-1 focus:ring-[var(--color-accent)]"
        />
      </div>
      <button
        onclick={() => (isModalOpen = true)}
        class="flex w-full sm:w-auto items-center gap-2 rounded-md bg-[var(--color-accent)] px-4 py-2.5 text-sm font-semibold text-black transition hover:bg-[var(--color-accent-hover)]"
      >
        <Sparkles class="h-4 w-4" />
        Generate
      </button>
    </div>
  </header>

  <div class="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)]">
    <ul class="divide-y divide-[var(--color-border)]">
      {#each filteredEmails as email (email.id)}
        <li class="px-5 py-4 hover:bg-[var(--color-surface-2)]/50">
          <div class="flex flex-col space-y-2">
            <span class="text-xs text-[var(--color-muted)]">
              {new Date(email.created_at).toLocaleString(undefined, {
                month: 'short', day: 'numeric', hour: 'numeric', minute: 'numeric'
              })}
            </span>
            <p class="text-sm text-white font-medium">Prompt: {email.original_content}</p>
            <p class="text-sm text-[var(--color-muted-foreground)] whitespace-pre-wrap"><span class="text-[var(--color-accent)] mr-2 font-bold font-mono">↳</span>{email.generated_response}</p>
          </div>
        </li>
      {:else}
        <li class="px-5 py-10 text-center text-sm text-[var(--color-muted-foreground)]">
          {searchQuery ? "No matching emails found." : "No emails found."}
        </li>
      {/each}
    </ul>
  </div>
</div>

{#if isModalOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-sm p-4">
    <div class="w-full max-w-lg rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-6 shadow-2xl">
      <div class="flex items-center justify-between mb-4">
        <h2 class="text-lg font-semibold text-white">Generate New Email</h2>
        <button onclick={() => (isModalOpen = false)} class="text-[var(--color-muted)] hover:text-white">
          <X class="h-5 w-5" />
        </button>
      </div>
      <form onsubmit={handleGenerate}>
        <div class="space-y-4">
          <div>
            <label for="prompt" class="block text-sm font-medium text-[var(--color-muted-foreground)] mb-1">
              Instructions / Prompt
            </label>
            <textarea
              id="prompt"
              bind:value={promptText}
              rows="4"
              required
              class="w-full rounded-md border border-[var(--color-border)] bg-[var(--color-surface-2)] p-3 text-sm text-white focus:border-[var(--color-accent)] focus:outline-none focus:ring-1 focus:ring-[var(--color-accent)]"
              placeholder="E.g., Write a follow-up email to a client who missed a meeting..."
            ></textarea>
          </div>
          <div class="flex justify-end gap-3 pt-2">
            <button
              type="button"
              onclick={() => (isModalOpen = false)}
              class="rounded-md px-4 py-2 text-sm font-medium text-white hover:bg-[var(--color-surface-2)]"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={isGenerating}
              class="flex items-center justify-center gap-2 rounded-md bg-[var(--color-accent)] px-4 py-2 text-sm font-semibold text-black transition hover:bg-[var(--color-accent-hover)] disabled:opacity-50"
            >
              {#if isGenerating}
                <Loader2 class="h-4 w-4 animate-spin" />
                Generating...
              {:else}
                <Sparkles class="h-4 w-4" />
                Generate Email
              {/if}
            </button>
          </div>
        </div>
      </form>
    </div>
  </div>
{/if}