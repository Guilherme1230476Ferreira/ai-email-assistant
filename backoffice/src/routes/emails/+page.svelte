<script lang="ts">
  import type { PageData } from './$types';
  import { invalidateAll } from '$app/navigation';
  import { Sparkles, Search, Loader2, X, Database, Users, Calendar, FileText, CheckCircle2 } from '@lucide/svelte';

  let { data }: { data: PageData } = $props();

  let searchQuery = $state('');
  let isModalOpen = $state(false);
  let promptText = $state('');
  
  // Animation & AI Context States
  let isGenerating = $state(false);
  let processingStep = $state<'idle' | 'gathering' | 'generating'>('idle');
  let terminalLogs = $state<string[]>([]);

  // Idea 2: Dynamic Context Tags (Knowledge Badges)
  let detectedContexts = $derived.by(() => {
    const text = promptText.toLowerCase();
    const tags = [];
    if (text.includes('refund') || text.includes('money')) {
      tags.push({ label: '2 past refund tickets', icon: FileText, color: 'text-emerald-400', bg: 'bg-emerald-400/10', border: 'border-emerald-400/20' });
    }
    if (text.includes('meeting') || text.includes('call') || text.includes('schedule')) {
      tags.push({ label: 'Calendar: 1 free slot', icon: Calendar, color: 'text-blue-400', bg: 'bg-blue-400/10', border: 'border-blue-400/20' });
    }
    if (text.includes('joão') || text.includes('client') || text.includes('customer')) {
      tags.push({ label: 'CRM: Profile Found', icon: Users, color: 'text-purple-400', bg: 'bg-purple-400/10', border: 'border-purple-400/20' });
    }
    return tags;
  });

  function getToken() {
    const match = document.cookie.match(/(^| )token=([^;]+)/);
    return match ? match[2] : null;
  }

  // Idea 1 & 4 Implementation: Terminal Log Sequence
  async function handleGenerate(e: Event) {
    e.preventDefault();
    if (!promptText.trim()) return;
    
    isGenerating = true;
    terminalLogs = [];
    
    // Simulate Chain-of-Thought
    processingStep = 'gathering';
    terminalLogs = [...terminalLogs, '> Analyzing prompt semantics...'];
    await new Promise(r => setTimeout(r, 600));
    
    terminalLogs = [...terminalLogs, `> Found ${detectedContexts.length} relevant context markers.`];
    await new Promise(r => setTimeout(r, 800));
    
    terminalLogs = [...terminalLogs, '> Querying vector database for similar tone...'];
    await new Promise(r => setTimeout(r, 900));

    terminalLogs = [...terminalLogs, '> Initiating LLM text generation sequence...'];
    processingStep = 'generating';
    
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
        terminalLogs = [...terminalLogs, '> LLM completion successful. Applying safety filters...'];
        await new Promise(r => setTimeout(r, 800));
        
        terminalLogs = [...terminalLogs, '> Ready.'];
        await new Promise(r => setTimeout(r, 400));

        promptText = '';
        isModalOpen = false;
        processingStep = 'idle';
        await invalidateAll(); // Refresh data
      } else {
        terminalLogs = [...terminalLogs, '> ERROR: LLM returned an invalid response.'];
        console.error('Failed to generate.');
      }
    } catch (err) {
      terminalLogs = [...terminalLogs, '> ERROR: Connection failed.'];
      console.error(err);
    } finally {
      if (processingStep === 'generating') {
        setTimeout(() => {
          isGenerating = false;
          processingStep = 'idle';
        }, 1500);
      } else {
         isGenerating = false;
         processingStep = 'idle';
      }
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
  
  // Closing modal resets state
  function closeModal() {
    isModalOpen = false;
    isGenerating = false;
    processingStep = 'idle';
    terminalLogs = [];
  }
</script>

<svelte:head>
  <title>Emails · Mailwise</title>
</svelte:head>

<div class="mx-auto w-full max-w-5xl space-y-8">
  <header class="flex flex-col gap-4 mb-8">
    <div>
      <h1 class="text-2xl font-semibold tracking-tight text-white">Emails</h1>
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
              {new Date(email.created_at).toLocaleString()}
            </span>
            <p class="text-sm text-white font-medium">Prompt: {email.original_content}</p>
            <p class="text-sm text-[var(--color-muted-foreground)]"><span class="text-[var(--color-accent)] mr-2">↳</span>{email.generated_response}</p>
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
    <div class="w-full max-w-lg rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-6 shadow-2xl transition-all duration-300 relative overflow-hidden">
      
      {#if isGenerating}
        <!-- Decorative Glow Background while generating -->
        <div class="absolute -top-32 -left-32 w-64 h-64 bg-[var(--color-accent)] opacity-5 rounded-full blur-3xl pointer-events-none animate-pulse"></div>
        <div class="absolute -bottom-32 -right-32 w-64 h-64 bg-[#a855f7] opacity-5 rounded-full blur-3xl pointer-events-none animate-pulse" style="animation-delay: 1s;"></div>
      {/if}

      <div class="flex items-center justify-between mb-4 relative z-10">
        <h2 class="text-lg font-semibold text-white">
          {#if isGenerating}
            AI Processing
          {:else}
            Generate New Email
          {/if}
        </h2>
        
        {#if !isGenerating}
        <button onclick={closeModal} class="text-[var(--color-muted)] hover:text-white transition-colors">
          <X class="h-5 w-5" />
        </button>
        {/if}
      </div>

      <form onsubmit={handleGenerate} class="relative z-10">
        {#if !isGenerating}
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
                placeholder="E.g., Write a follow-up email to João about the refund..."
              ></textarea>
            </div>

            <!-- Idea 2 UI: Dynamic Context Badges -->
            <div class="min-h-[40px]">
              {#if detectedContexts.length > 0}
                <div class="flex flex-wrap gap-2 animate-in fade-in slide-in-from-bottom-2 duration-300">
                  {#each detectedContexts as tag}
                    <span class={`inline-flex items-center gap-1.5 rounded-full border px-2.5 py-1 text-xs font-medium ${tag.bg} ${tag.color} ${tag.border}`}>
                      <tag.icon class="h-3 w-3" />
                      {tag.label}
                    </span>
                  {/each}
                </div>
              {:else if promptText.length > 5}
                <span class="text-xs text-[var(--color-muted)] flex items-center gap-1.5 animate-in fade-in duration-300">
                  <Database class="h-3 w-3" /> Waiting for identifiable context keywords...
                </span>
              {/if}
            </div>

            <div class="flex justify-end gap-3 pt-2">
              <button
                type="button"
                onclick={closeModal}
                class="rounded-md px-4 py-2 text-sm font-medium text-white hover:bg-[var(--color-surface-2)] transition-colors"
              >
                Cancel
              </button>
              <button
                type="submit"
                class="flex items-center justify-center gap-2 rounded-md bg-[var(--color-accent)] px-4 py-2 text-sm font-semibold text-black transition hover:bg-[var(--color-accent-hover)]"
              >
                <Sparkles class="h-4 w-4" />
                Generate Email
              </button>
            </div>
          </div>
        {:else}
          <!-- Idea 1 & 4 UI: Mini Knowledge Graph & Terminal -->
          <div class="space-y-6 pt-4 animate-in zoom-in-95 duration-500">
            
            <!-- Visual Graph Area -->
            <div class="flex justify-center items-center py-6 h-32 relative">
              <!-- Animated connection lines -->
              <div class="absolute inset-0 flex justify-center items-center">
                {#if processingStep === 'gathering'}
                <div class="w-32 border-t border-dashed border-[var(--color-accent)] opacity-40 absolute animate-pulse"></div>
                <div class="h-32 border-l border-dashed border-[var(--color-accent)] opacity-40 absolute animate-pulse" style="animation-delay: 0.5s;"></div>
                {/if}
              </div>
              
              <!-- Center LLM Node -->
              <div class="relative z-10 flex h-14 w-14 items-center justify-center rounded-full bg-[var(--color-surface-2)] border-2 border-[var(--color-accent)] shadow-[0_0_15px_rgba(45,212,191,0.4)]">
                {#if processingStep === 'gathering'}
                  <Database class="h-6 w-6 text-[var(--color-accent)] animate-pulse" />
                {:else}
                  <Sparkles class="h-6 w-6 text-[var(--color-accent)] animate-spin-slow" />
                {/if}
              </div>

              <!-- Orbiting context node (CSS mock) -->
              {#if processingStep === 'gathering' && detectedContexts.length > 0}
              <div class="absolute inset-0 animate-spin-slow pointer-events-none" style="animation-duration: 4s;">
                <div class="absolute top-2 left-1/2 -ml-3 flex h-6 w-6 items-center justify-center rounded-full bg-emerald-500/20 border border-emerald-500/50">
                  <CheckCircle2 class="h-3 w-3 text-emerald-400" />
                </div>
              </div>
              {/if}
            </div>

            <!-- Terminal Chain of Thought Log -->
            <div class="rounded-md border border-[var(--color-border)] bg-black/50 p-4 font-mono text-xs text-green-400 h-40 overflow-y-auto shadow-inner flex flex-col gap-1.5">
              {#each terminalLogs as log}
                <div class="animate-in slide-in-from-left-2 fade-in duration-300 opacity-90">{log}</div>
              {/each}
              <div class="opacity-50 animate-pulse flex items-center h-4">_</div>
            </div>

            <!-- Progress Bar -->
            <div class="w-full bg-[var(--color-surface-2)] h-1.5 rounded-full overflow-hidden">
               <div 
                 class="bg-[var(--color-accent)] h-full transition-all duration-500 ease-out"
                 style={`width: ${processingStep === 'gathering' ? '45%' : '85%'}`}
               ></div>
            </div>
          </div>
        {/if}
      </form>
    </div>
  </div>
{/if}

<style>
  .animate-spin-slow {
    animation: spin 3s linear infinite;
  }
</style>