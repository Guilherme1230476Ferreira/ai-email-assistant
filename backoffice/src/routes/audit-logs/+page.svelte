<script lang="ts">
  import type { PageData } from './$types';
  import { goto } from '$app/navigation';
  import { ShieldAlert, Search, FileText } from '@lucide/svelte';
  import { locale, t, type Locale } from '$lib/i18n';

  let { data }: { data: PageData } = $props();

  let currentLocale = $state<Locale>($locale);
  locale.subscribe((val) => (currentLocale = val));

  let searchQuery = $state('');
  
  function goToPage(page: number) {
      goto(`/audit-logs?page=${page}&limit=${data.pagination.limit}`);
  }

  let filteredLogs = $derived(
    (data.logs || []).filter((log: any) => {
      const q = searchQuery.toLowerCase();
      return (
        log.action.toLowerCase().includes(q) ||
        (log.user_id && log.user_id.toLowerCase().includes(q))
      );
    })
  );

  function formatMetadata(metadata: any) {
    if (!metadata) return 'No metadata';
    return JSON.stringify(metadata, null, 2);
  }
</script>

<svelte:head>
  <title>{t('audit.title', currentLocale)} · MailMate</title>
</svelte:head>

<div class="mx-auto w-full max-w-5xl space-y-8 pb-12">
  <header class="flex flex-col gap-4 mb-8">
    <div>
      <h1 class="text-2xl font-semibold tracking-tight text-white flex items-center gap-2">
        <ShieldAlert class="h-6 w-6 text-[var(--color-accent)]" />
        {t('audit.title', currentLocale)}
      </h1>
      <p class="text-sm text-[var(--color-muted-foreground)]">{t('audit.subtitle', currentLocale)}</p>
    </div>

    <div class="flex items-center gap-4">
      <div class="relative flex-1 w-full max-w-md">
        <Search class="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-[var(--color-muted)]" />
        <input
          type="text"
          bind:value={searchQuery}
          placeholder={t('audit.search', currentLocale)}
          class="w-full rounded-md border border-[var(--color-border)] bg-[var(--color-surface-2)] py-2.5 pl-9 pr-4 text-sm text-white placeholder:text-[var(--color-muted)] focus:border-[var(--color-accent)] focus:outline-none focus:ring-1 focus:ring-[var(--color-accent)]"
        />
      </div>
    </div>
  </header>

  <div class="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] overflow-hidden shadow-sm">
    <div class="overflow-x-auto">
        <table class="w-full text-left text-sm text-[var(--color-muted-foreground)]">
            <thead class="bg-[var(--color-surface-2)] text-xs uppercase text-[var(--color-muted)] border-b border-[var(--color-border)]">
                <tr>
                    <th scope="col" class="px-5 py-3 font-medium">{t('audit.timestamp', currentLocale)}</th>
                    <th scope="col" class="px-5 py-3 font-medium">{t('audit.actor', currentLocale)}</th>
                    <th scope="col" class="px-5 py-3 font-medium">{t('audit.action', currentLocale)}</th>
                    <th scope="col" class="px-5 py-3 font-medium text-right">{t('audit.details', currentLocale)}</th>
                </tr>
            </thead>
            <tbody class="divide-y divide-[var(--color-border)]">
                {#each filteredLogs as log (log.id)}
                    <tr class="hover:bg-[var(--color-surface-2)]/30 transition-colors">
                        <td class="whitespace-nowrap px-5 py-4 text-xs font-mono text-[var(--color-muted)]">
                            {new Date(log.created_at).toLocaleString()}
                        </td>
                        <td class="px-5 py-4 font-mono text-xs text-white">
                            {log.user_id || t('common.system', currentLocale)}
                        </td>
                        <td class="px-5 py-4">
                            <span class="inline-flex items-center rounded-full bg-[var(--color-surface-2)] px-2.5 py-0.5 text-xs font-semibold text-white border border-[var(--color-border)]">
                                {log.action}
                            </span>
                        </td>
                        <td class="px-5 py-4 text-right group relative">
                            <details class="cursor-pointer">
                                <summary class="text-xs text-[var(--color-accent)] hover:text-[var(--color-accent-hover)] transition inline-flex items-center gap-1 list-none">
                                    <FileText class="w-3.5 h-3.5" />
                                    <span>{t('audit.view_json', currentLocale)}</span>
                                </summary>
                                <div class="absolute right-10 top-2 z-10 w-[400px] text-left p-3 rounded-md bg-black border border-[var(--color-border)] shadow-xl mt-2 hidden group-open:block group-focus-within:block max-h-[300px] overflow-y-auto">
                                    <h4 class="text-xs uppercase font-bold text-[var(--color-muted)] mb-2">{t('audit.payload', currentLocale)}</h4>
                                    <pre class="text-[10px] text-green-400 font-mono whitespace-pre-wrap leading-relaxed">{formatMetadata(log.metadata)}</pre>
                                </div>
                            </details>
                        </td>
                    </tr>
                {:else}
                    <tr>
                        <td colspan="4" class="px-5 py-10 text-center text-sm">
                            {searchQuery ? t('audit.no_match', currentLocale) : t('audit.no_logs', currentLocale)}
                        </td>
                    </tr>
                {/each}
            </tbody>
        </table>
    </div>
    
    <!-- Pagination -->
    {#if data.pagination && data.pagination.total > data.pagination.limit && !searchQuery}
        <div class="px-5 py-4 border-t border-[var(--color-border)] flex items-center justify-between bg-[var(--color-surface-2)]/30">
            <span class="text-sm text-[var(--color-muted-foreground)]">
                {t('audit.showing', currentLocale)} {((data.pagination.page - 1) * data.pagination.limit) + 1} {t('common.to', currentLocale)} {Math.min(data.pagination.page * data.pagination.limit, data.pagination.total)} {t('audit.of', currentLocale)} {data.pagination.total} {t('audit.events', currentLocale)}
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
