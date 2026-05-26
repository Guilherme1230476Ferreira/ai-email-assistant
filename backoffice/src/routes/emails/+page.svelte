<script lang="ts">
  import type { PageData } from './$types';
  import { invalidateAll } from '$app/navigation';
  import { Sparkles, Search, X, Database, ChevronDown, ChevronUp, Copy, Check, Trash2, Loader2, GitBranch } from '@lucide/svelte';
  import { api } from '$lib/api';
  import { goto } from '$app/navigation';
  import { locale, t, type Locale } from '$lib/i18n';

  let { data }: { data: PageData } = $props();

  let currentLocale = $state<Locale>($locale);
  locale.subscribe((val) => (currentLocale = val));

  let searchQuery = $state('');
  let isModalOpen = $state(false);
  let promptText = $state('');

  // Generation states
  let isGenerating = $state(false);
  let processingStep = $state<'idle' | 'gathering' | 'generating'>('idle');
  let terminalLogs = $state<string[]>([]);
  let modalError = $state<string | null>(null);

  // Email detail expand state
  let expandedEmailId = $state<string | null>(null);
  let copiedEmailId = $state<string | null>(null);
  let deletingEmailId = $state<string | null>(null);

  // RAG trace state
  type TraceItem = { source: string; title: string; text: string; score: number };
  let traceMap = $state<Record<string, TraceItem[] | 'loading' | 'error' | null>>({});
  let showTraceForId = $state<string | null>(null);

  async function loadTrace(emailId: string) {
    if (traceMap[emailId] !== undefined) {
      // Toggle visibility
      showTraceForId = showTraceForId === emailId ? null : emailId;
      return;
    }
    showTraceForId = emailId;
    traceMap = { ...traceMap, [emailId]: 'loading' };
    try {
      const result = await api.getEmailTrace(emailId);
      traceMap = { ...traceMap, [emailId]: result.error ? 'error' : (result.data ?? []) };
    } catch {
      traceMap = { ...traceMap, [emailId]: 'error' };
    }
  }

  function openModal() {
    isModalOpen = true;
    modalError = null;
    promptText = '';
    terminalLogs = [];
    processingStep = 'idle';
  }

  function closeModal() {
    if (isGenerating) return; // block close while running
    isModalOpen = false;
    isGenerating = false;
    processingStep = 'idle';
    terminalLogs = [];
    modalError = null;
  }

  async function handleGenerate(e: Event) {
    e.preventDefault();
    if (!promptText.trim()) return;

    isGenerating = true;
    modalError = null;
    terminalLogs = [];
    processingStep = 'gathering';

    // Flag to know when to start appending tokens on the last line
    let streamingTokens = false;

    await api.generateEmailStream(
        promptText,
        (token) => {
            if (!streamingTokens) {
                // First token — add a fresh line to append to
                terminalLogs = [...terminalLogs, '> '];
                streamingTokens = true;
            }
            terminalLogs[terminalLogs.length - 1] += token;
            terminalLogs = [...terminalLogs];
        },
        (err) => {
            terminalLogs = [...terminalLogs, `> ERROR: ${err}`];
            modalError = err;
            isGenerating = false;
            processingStep = 'idle';
        },
        async () => {
            promptText = '';
            isGenerating = false;
            processingStep = 'idle';
            isModalOpen = false;
            await invalidateAll();
        },
        (msg: string) => {
            // Real backend log event — push to terminal
            terminalLogs = [...terminalLogs, msg];
            // When stage 4 log arrives, prepare token stream line
            if (msg.includes('[4/4]')) {
                processingStep = 'generating';
                streamingTokens = false; // next token triggers new line
            }
        }
    );
  }

  function toggleExpand(id: string) {
      if (expandedEmailId === id) {
          expandedEmailId = null;
      } else {
          expandedEmailId = id;
      }
  }

  function copyReply(id: string, text: string, e: Event) {
      e.stopPropagation(); // prevent toggling expand
      navigator.clipboard.writeText(text);
      copiedEmailId = id;
      setTimeout(() => {
          copiedEmailId = null;
      }, 2000);
  }

  function goToPage(page: number) {
      goto(`/emails?page=${page}&limit=${data.pagination.limit}`);
  }

  async function deleteEmail(id: string, e: Event) {
      e.stopPropagation();
      if (!confirm('Are you sure you want to delete this email?')) return;
      deletingEmailId = id;
      try {
          const result = await api.deleteEmail(id);
          if (result.error) {
              console.error('Delete failed:', result.error);
          } else {
              if (expandedEmailId === id) expandedEmailId = null;
              await invalidateAll();
          }
      } finally {
          deletingEmailId = null;
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
  <title>{t('emails.title', currentLocale)} · MailMate</title>
</svelte:head>

<div class="mx-auto w-full max-w-5xl space-y-8 pb-12">
  <header class="flex flex-col gap-4 mb-8">
    <div>
      <h1 class="text-2xl font-semibold tracking-tight text-white">{t('emails.title', currentLocale)}</h1>
      <p class="text-sm text-[var(--color-muted-foreground)]">{t('emails.subtitle', currentLocale)}</p>
    </div>

    <div class="flex flex-col sm:flex-row items-center gap-4">
      <div class="relative flex-1 w-full">
        <Search class="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-[var(--color-muted)]" />
        <input
          type="text"
          bind:value={searchQuery}
          placeholder={t('emails.search', currentLocale)}
          class="w-full rounded-md border border-[var(--color-border)] bg-[var(--color-surface-2)] py-2.5 pl-9 pr-4 text-sm text-white placeholder:text-[var(--color-muted)] focus:border-[var(--color-accent)] focus:outline-none focus:ring-1 focus:ring-[var(--color-accent)]"
        />
      </div>
      <button
        onclick={openModal}
        class="flex w-full sm:w-auto items-center gap-2 rounded-md bg-[var(--color-accent)] px-4 py-2.5 text-sm font-semibold text-black transition hover:bg-[var(--color-accent-hover)]"
      >
        <Sparkles class="h-4 w-4" />
        {t('emails.generate', currentLocale)}
      </button>
    </div>
  </header>

  <div class="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] overflow-hidden">
    <ul class="divide-y divide-[var(--color-border)]">
      {#each filteredEmails as email (email.id)}
        <li class="hover:bg-[var(--color-surface-2)]/30 transition-colors">
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div class="px-5 py-4 cursor-pointer" onclick={() => toggleExpand(email.id)}>
            <div class="flex justify-between items-start">
              <div class="flex flex-col space-y-2 flex-1 min-w-0 pr-4">
                <span class="text-xs text-[var(--color-muted)]">
                  {new Date(email.created_at).toLocaleString()}
                </span>
              <p class="text-sm text-white font-medium truncate">{t('emails.received', currentLocale)}: {email.original_content}</p>
              <p class="text-sm text-[var(--color-muted-foreground)] truncate">
                <span class="text-[var(--color-accent)] mr-2">↳</span>{t('emails.reply_label', currentLocale)}: {email.generated_response}
              </p>
              </div>
              <button class="text-[var(--color-muted)] hover:text-white transition-colors mt-2">
                {#if expandedEmailId === email.id}
                    <ChevronUp class="w-5 h-5" />
                {:else}
                    <ChevronDown class="w-5 h-5" />
                {/if}
              </button>
            </div>
          </div>
          
          {#if expandedEmailId === email.id}
            <div class="px-5 pb-5 pt-2 border-t border-[var(--color-border)]/50 bg-[var(--color-surface-2)]/20 animate-in slide-in-from-top-2 fade-in duration-200">
                <div class="space-y-4 mt-2">
                    <div>
                    <h4 class="text-xs font-semibold text-[var(--color-muted-foreground)] uppercase tracking-wider mb-2">{t('emails.original', currentLocale)}</h4>
                        <div class="p-3 rounded-md bg-[var(--color-surface-2)] border border-[var(--color-border)] text-sm text-gray-300 whitespace-pre-wrap leading-relaxed">{email.original_content}</div>
                    </div>
                    <div>
                        <div class="flex items-center justify-between mb-2">
                            <h4 class="text-xs font-semibold text-[var(--color-accent)] uppercase tracking-wider">{t('emails.generated', currentLocale)}</h4>
                            <button 
                                onclick={(e) => copyReply(email.id, email.generated_response, e)}
                                class="flex items-center gap-1.5 text-xs font-medium text-[var(--color-muted-foreground)] hover:text-white transition-colors bg-[var(--color-surface-2)] px-2.5 py-1 rounded-md border border-[var(--color-border)] hover:border-[var(--color-muted)]"
                            >
                                {#if copiedEmailId === email.id}
                                    <Check class="w-3.5 h-3.5 text-green-400" />
                                    <span class="text-green-400">{t('emails.copied', currentLocale)}</span>
                                {:else}
                                    <Copy class="w-3.5 h-3.5" />
                                    <span>{t('emails.copy', currentLocale)}</span>
                                {/if}
                            </button>
                        </div>
                        <div class="p-4 rounded-md bg-black/30 border border-[var(--color-border)] text-sm text-white whitespace-pre-wrap leading-relaxed relative group">
                            {email.generated_response}
                        </div>
                    </div>
                    <div class="flex items-center gap-2 mt-4 pt-3 border-t border-[var(--color-border)]/30">
                        <!-- RAG Trace button -->
                        <button
                            onclick={() => loadTrace(email.id)}
                            class="flex items-center gap-1.5 text-xs font-medium text-[var(--color-accent)]/80 hover:text-[var(--color-accent)] transition-colors bg-[var(--color-accent)]/5 hover:bg-[var(--color-accent)]/10 px-3 py-1.5 rounded-md border border-[var(--color-accent)]/20 hover:border-[var(--color-accent)]/40"
                        >
                            <GitBranch class="w-3.5 h-3.5" />
                            <span>{t('emails.view_trace', currentLocale)}</span>
                        </button>
                        <button
                            onclick={(e) => deleteEmail(email.id, e)}
                            disabled={deletingEmailId === email.id}
                            class="flex items-center gap-1.5 text-xs font-medium text-red-400/80 hover:text-red-400 transition-colors bg-red-500/5 hover:bg-red-500/10 px-3 py-1.5 rounded-md border border-red-500/20 hover:border-red-500/40 disabled:opacity-50 disabled:cursor-not-allowed ml-auto"
                        >
                            {#if deletingEmailId === email.id}
                                <Loader2 class="w-3.5 h-3.5 animate-spin" />
                                <span>{t('emails.deleting', currentLocale)}</span>
                            {:else}
                                <Trash2 class="w-3.5 h-3.5" />
                                <span>{t('emails.delete', currentLocale)}</span>
                            {/if}
                        </button>
                    </div>

                    <!-- RAG Trace Panel -->
                    {#if showTraceForId === email.id}
                        <div class="mt-3 pt-3 border-t border-[var(--color-border)]/40 animate-in fade-in slide-in-from-top-2 duration-200">
                            <h4 class="text-xs font-semibold text-[var(--color-accent)] uppercase tracking-wider mb-2 flex items-center gap-1.5">
                                <GitBranch class="w-3 h-3" />
                                {t('emails.rag_trace', currentLocale)}
                            </h4>
                            {#if traceMap[email.id] === 'loading'}
                                <div class="text-xs text-[var(--color-muted)] flex items-center gap-2 py-2">
                                    <Loader2 class="w-3 h-3 animate-spin" />
                                    {t('emails.rag_trace_loading', currentLocale)}
                                </div>
                            {:else if traceMap[email.id] === 'error'}
                                <p class="text-xs text-red-400 py-2">{t('emails.rag_trace_empty', currentLocale)}</p>
                            {:else if Array.isArray(traceMap[email.id]) && (traceMap[email.id] as any[]).length === 0}
                                <p class="text-xs text-[var(--color-muted)] py-2">{t('emails.rag_trace_empty', currentLocale)}</p>
                            {:else if Array.isArray(traceMap[email.id])}
                                {@const traceItems = traceMap[email.id] as TraceItem[]}
                                <div class="space-y-2">
                                    {#each traceItems as item}
                                        <div class="rounded-md bg-[var(--color-surface-2)] border border-[var(--color-border)] p-2.5 text-xs">
                                            <div class="flex items-center justify-between mb-1.5 gap-2">
                                                <span class="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] font-semibold
                                                    {item.source === 'knowledge_base'
                                                        ? 'bg-blue-500/15 text-blue-400 border border-blue-500/30'
                                                        : 'bg-purple-500/15 text-purple-400 border border-purple-500/30'}">
                                                    {item.source === 'knowledge_base' ? t('emails.rag_source_kb', currentLocale) : t('emails.rag_source_email', currentLocale)}
                                                </span>
                                                <span class="text-[var(--color-muted)] font-mono shrink-0">
                                                    {t('emails.rag_similarity', currentLocale)}: <span class="text-[var(--color-accent)]">{(item.score * 100).toFixed(1)}%</span>
                                                </span>
                                            </div>
                                            <p class="font-medium text-white mb-1 truncate">{item.title}</p>
                                            <!-- Similarity bar -->
                                            <div class="w-full bg-[var(--color-surface)] h-1 rounded-full overflow-hidden mb-1.5">
                                                <div class="h-full bg-[var(--color-accent)] rounded-full transition-all duration-700"
                                                     style="width: {Math.min(100, item.score * 100).toFixed(1)}%"></div>
                                            </div>
                                            <p class="text-[var(--color-muted-foreground)] line-clamp-2 leading-relaxed">{item.text}</p>
                                        </div>
                                    {/each}
                                </div>
                            {/if}
                        </div>
                    {/if}

                </div>
            </div>
          {/if}
        </li>
      {:else}
        <li class="px-5 py-10 text-center text-sm text-[var(--color-muted-foreground)]">
          {searchQuery ? t('emails.no_match', currentLocale) : t('emails.no_emails', currentLocale)}
        </li>
      {/each}
    </ul>
    
    <!-- Pagination -->
    {#if data.pagination && data.pagination.total > data.pagination.limit && !searchQuery}
        <div class="px-5 py-4 border-t border-[var(--color-border)] flex items-center justify-between bg-[var(--color-surface-2)]/30">
            <span class="text-sm text-[var(--color-muted-foreground)]">
                {t('common.showing', currentLocale)} {((data.pagination.page - 1) * data.pagination.limit) + 1} {t('common.to', currentLocale)} {Math.min(data.pagination.page * data.pagination.limit, data.pagination.total)} {t('common.of', currentLocale)} {data.pagination.total} {t('emails.title', currentLocale).toLowerCase()}
            </span>
            <div class="flex gap-2">
                <button 
                    disabled={data.pagination.page <= 1}
                    onclick={() => goToPage(data.pagination.page - 1)}
                    class="px-3 py-1.5 rounded-md text-sm font-medium border border-[var(--color-border)] bg-[var(--color-surface)] text-white hover:bg-[var(--color-surface-2)] disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                >
                    {t('common.previous', currentLocale)}
                </button>
                <button 
                    disabled={data.pagination.page * data.pagination.limit >= data.pagination.total}
                    onclick={() => goToPage(data.pagination.page + 1)}
                    class="px-3 py-1.5 rounded-md text-sm font-medium border border-[var(--color-border)] bg-[var(--color-surface)] text-white hover:bg-[var(--color-surface-2)] disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                >
                    {t('common.next', currentLocale)}
                </button>
            </div>
        </div>
    {/if}
  </div>
</div>

{#if isModalOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-sm p-4">
    <div class="w-full max-w-lg rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-6 shadow-2xl relative overflow-hidden">

      {#if isGenerating}
        <div class="absolute -top-32 -left-32 w-64 h-64 bg-[var(--color-accent)] opacity-5 rounded-full blur-3xl pointer-events-none animate-pulse"></div>
        <div class="absolute -bottom-32 -right-32 w-64 h-64 bg-[#a855f7] opacity-5 rounded-full blur-3xl pointer-events-none animate-pulse" style="animation-delay:1s;"></div>
      {/if}

      <div class="flex items-center justify-between mb-4 relative z-10">
        <h2 class="text-lg font-semibold text-white">
          {isGenerating ? t('emails.ai_processing', currentLocale) : t('emails.generate_new', currentLocale)}
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
            <!-- Error banner -->
            {#if modalError}
              <div class="rounded-md border border-red-500/30 bg-red-500/8 px-4 py-3 text-sm text-red-300">
                {modalError}
              </div>
            {/if}

            <div>
              <label for="prompt" class="block text-sm font-medium text-[var(--color-muted-foreground)] mb-1">
                {t('emails.prompt_label', currentLocale)}
              </label>
              <textarea
                id="prompt"
                bind:value={promptText}
                rows="4"
                required
                class="w-full rounded-md border border-[var(--color-border)] bg-[var(--color-surface-2)] p-3 text-sm text-white focus:border-[var(--color-accent)] focus:outline-none focus:ring-1 focus:ring-[var(--color-accent)]"
                placeholder={t('emails.prompt_placeholder', currentLocale)}
              ></textarea>
            </div>

            <!-- Honest context info -->
            {#if promptText.length > 5}
              <div class="flex items-center gap-2 text-xs text-[var(--color-muted)]">
                <Database class="h-3 w-3" />
                {t('emails.context_info', currentLocale)}
              </div>
            {/if}

            <div class="flex justify-end gap-3 pt-2">
              <button type="button" onclick={closeModal}
                class="rounded-md px-4 py-2 text-sm font-medium text-white hover:bg-[var(--color-surface-2)] transition-colors">{t('common.cancel', currentLocale)}
              </button>
              <button type="submit"
                class="flex items-center justify-center gap-2 rounded-md bg-[var(--color-accent)] px-4 py-2 text-sm font-semibold text-black transition hover:bg-[var(--color-accent-hover)]">
                <Sparkles class="h-4 w-4" />
                {t('emails.generate', currentLocale)}
              </button>
            </div>
          </div>
        {:else}
          <div class="space-y-6 pt-4">
            <div class="flex justify-center items-center py-6 h-32 relative">
              <div class="relative z-10 flex h-14 w-14 items-center justify-center rounded-full bg-[var(--color-surface-2)] border-2 border-[var(--color-accent)] shadow-[0_0_15px_rgba(45,212,191,0.4)]">
                {#if processingStep === 'gathering'}
                  <Database class="h-6 w-6 text-[var(--color-accent)] animate-pulse" />
                {:else}
                  <Sparkles class="h-6 w-6 text-[var(--color-accent)] animate-spin-slow" />
                {/if}
              </div>
            </div>

            <!-- Terminal log -->
            <div class="rounded-md border border-[var(--color-border)] bg-black/50 p-4 font-mono text-xs text-green-400 h-40 overflow-y-auto shadow-inner flex flex-col gap-1.5 whitespace-pre-wrap">
              {#each terminalLogs as log}
                <div class="animate-in slide-in-from-left-2 fade-in duration-300 opacity-90">{log}</div>
              {/each}
              <div class="opacity-50 animate-pulse flex items-center h-4">_</div>
            </div>

            <!-- Progress bar -->
            <div class="w-full bg-[var(--color-surface-2)] h-1.5 rounded-full overflow-hidden">
              <div class="bg-[var(--color-accent)] h-full transition-all duration-500 ease-out"
                style={`width: ${processingStep === 'gathering' ? '45%' : '85%'}`}
              ></div>
            </div>
          </div>
        {/if}
      </form>
    </div>
  </div>
{/if}
