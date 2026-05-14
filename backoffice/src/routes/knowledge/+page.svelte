<script lang="ts">
  import { authToken } from '$lib/stores/auth';
  import { invalidateAll } from '$app/navigation';
  import { BookOpen, FileText, HelpCircle, Upload, Trash2, Plus, Loader2 } from '@lucide/svelte';

  let { data } = $props();

  let activeTab = $state<'documents' | 'qa'>('qa');

  // ── Q&A form state
  let question = $state('');
  let answer = $state('');
  let qaLoading = $state(false);
  let qaError = $state('');
  let qaSuccess = $state('');

  // ── Upload state
  let uploadLoading = $state(false);
  let uploadError = $state('');
  let uploadSuccess = $state('');
  let fileInput = $state<HTMLInputElement>();
  let dragOver = $state(false);

  // ── Delete state
  let deletingId = $state<string | null>(null);

  const apiBase = '/api';

  async function submitQA() {
    if (!question.trim() || !answer.trim()) return;
    qaLoading = true;
    qaError = '';
    qaSuccess = '';

    try {
      const res = await fetch(`${apiBase}/knowledge`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${$authToken}`
        },
        body: JSON.stringify({ question: question.trim(), answer: answer.trim() })
      });

      if (!res.ok) {
        const err = await res.json().catch(() => ({ message: 'Failed to create Q&A pair' }));
        throw new Error(err.message || 'Failed');
      }

      question = '';
      answer = '';
      qaSuccess = 'Q&A pair added and embedded successfully!';
      await invalidateAll();
      setTimeout(() => (qaSuccess = ''), 4000);
    } catch (e: any) {
      qaError = e.message;
    } finally {
      qaLoading = false;
    }
  }

  async function uploadFile(file: File) {
    uploadLoading = true;
    uploadError = '';
    uploadSuccess = '';

    const formData = new FormData();
    formData.append('file', file);

    try {
      const res = await fetch(`${apiBase}/knowledge/upload`, {
        method: 'POST',
        headers: { Authorization: `Bearer ${$authToken}` },
        body: formData
      });

      if (!res.ok) {
        const err = await res.json().catch(() => ({ message: 'Upload failed' }));
        throw new Error(err.message || 'Upload failed');
      }

      const result = await res.json();
      uploadSuccess = `"${result.title}" uploaded — ${result.chunk_count} chunks embedded.`;
      await invalidateAll();
      setTimeout(() => (uploadSuccess = ''), 5000);
    } catch (e: any) {
      uploadError = e.message;
    } finally {
      uploadLoading = false;
    }
  }

  function handleFileSelect(e: Event) {
    const target = e.target as HTMLInputElement;
    if (target.files?.[0]) uploadFile(target.files[0]);
  }

  function handleDrop(e: DragEvent) {
    e.preventDefault();
    dragOver = false;
    if (e.dataTransfer?.files?.[0]) uploadFile(e.dataTransfer.files[0]);
  }

  async function deleteEntry(id: string) {
    deletingId = id;
    try {
      const res = await fetch(`${apiBase}/knowledge/${id}`, {
        method: 'DELETE',
        headers: { Authorization: `Bearer ${$authToken}` }
      });
      if (!res.ok && res.status !== 204) throw new Error('Delete failed');
      await invalidateAll();
    } catch (e: any) {
      console.error(e);
    } finally {
      deletingId = null;
    }
  }

  // Split entries by type for display
  let documents = $derived(data.entries.filter((e: any) => e.entry_type === 'document'));
  let qaPairs = $derived(data.entries.filter((e: any) => e.entry_type === 'qa_pair'));
</script>

<svelte:head>
  <title>Knowledge Base · MailMate</title>
</svelte:head>

<div class="mx-auto w-full max-w-5xl space-y-8 pb-12">
  <!-- Header -->
  <div>
    <h1 class="text-2xl font-bold tracking-tight text-white">Knowledge Base</h1>
    <p class="mt-1 text-sm text-[var(--color-muted)]">
      Manage documents and Q&A pairs that power the RAG pipeline. Knowledge entries are global and available to all users.
    </p>
  </div>

  <!-- Tabs -->
  <div class="flex gap-1 rounded-lg bg-[var(--color-surface)] p-1 border border-[var(--color-border)]">
    <button
      class="flex-1 rounded-md px-4 py-2 text-sm font-medium transition {activeTab === 'qa'
        ? 'bg-[var(--color-accent)] text-black'
        : 'text-[var(--color-muted-foreground)] hover:text-white hover:bg-[var(--color-surface-2)]'}"
      onclick={() => (activeTab = 'qa')}>
      <span class="inline-flex items-center gap-2"><HelpCircle class="h-4 w-4" /> Q&A Pairs</span>
    </button>
    <button
      class="flex-1 rounded-md px-4 py-2 text-sm font-medium transition {activeTab === 'documents'
        ? 'bg-[var(--color-accent)] text-black'
        : 'text-[var(--color-muted-foreground)] hover:text-white hover:bg-[var(--color-surface-2)]'}"
      onclick={() => (activeTab = 'documents')}>
      <span class="inline-flex items-center gap-2"><FileText class="h-4 w-4" /> Documents</span>
    </button>
  </div>

  <!-- Q&A Tab -->
  {#if activeTab === 'qa'}
    <div class="space-y-6">
      <!-- Add Q&A Form -->
      <div class="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-6">
        <h2 class="mb-4 text-lg font-semibold text-white flex items-center gap-2">
          <Plus class="h-5 w-5 text-[var(--color-accent)]" />
          Add Q&A Pair
        </h2>
        <p class="mb-4 text-xs text-[var(--color-muted)]">
          Add a question and its ideal answer. The question is embedded so that when similar incoming emails arrive, the answer is used as RAG context.
        </p>

        <form onsubmit={(e) => { e.preventDefault(); submitQA(); }} class="space-y-4">
          <div>
            <label for="question" class="mb-1 block text-xs font-semibold uppercase tracking-wider text-[var(--color-muted-foreground)]">Question</label>
            <input id="question" type="text" bind:value={question} placeholder="e.g. What is our refund policy?"
              class="w-full rounded-lg border border-[var(--color-border)] bg-[var(--color-surface-2)] px-4 py-2.5 text-sm text-white placeholder-[var(--color-muted)] focus:border-[var(--color-accent)] focus:outline-none focus:ring-1 focus:ring-[var(--color-accent)]" />
          </div>
          <div>
            <label for="answer" class="mb-1 block text-xs font-semibold uppercase tracking-wider text-[var(--color-muted-foreground)]">Answer</label>
            <textarea id="answer" bind:value={answer} rows="4" placeholder="e.g. We offer a 30-day full refund on all products..."
              class="w-full rounded-lg border border-[var(--color-border)] bg-[var(--color-surface-2)] px-4 py-2.5 text-sm text-white placeholder-[var(--color-muted)] focus:border-[var(--color-accent)] focus:outline-none focus:ring-1 focus:ring-[var(--color-accent)] resize-none"></textarea>
          </div>

          {#if qaError}
            <p class="text-sm text-red-400">{qaError}</p>
          {/if}
          {#if qaSuccess}
            <p class="text-sm text-emerald-400">{qaSuccess}</p>
          {/if}

          <button type="submit" disabled={qaLoading || !question.trim() || !answer.trim()}
            class="inline-flex items-center gap-2 rounded-lg bg-[var(--color-accent)] px-5 py-2.5 text-sm font-semibold text-black transition hover:bg-[var(--color-accent-500)] disabled:opacity-50 disabled:cursor-not-allowed">
            {#if qaLoading}
              <Loader2 class="h-4 w-4 animate-spin" /> Embedding...
            {:else}
              <Plus class="h-4 w-4" /> Add Entry
            {/if}
          </button>
        </form>
      </div>

      <!-- Q&A Entries List -->
      <div class="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)]">
        <div class="border-b border-[var(--color-border)] px-6 py-4">
          <h2 class="text-sm font-semibold text-white">Existing Q&A Pairs ({qaPairs.length})</h2>
        </div>

        {#if qaPairs.length === 0}
          <div class="px-6 py-12 text-center text-sm text-[var(--color-muted)]">
            No Q&A pairs yet. Add one above to enrich the RAG context.
          </div>
        {:else}
          <div class="divide-y divide-[var(--color-border)]">
            {#each qaPairs as entry (entry.id)}
              <div class="group flex items-start gap-4 px-6 py-4 hover:bg-[var(--color-surface-2)] transition">
                <div class="mt-0.5 flex h-8 w-8 shrink-0 items-center justify-center rounded-lg bg-[var(--color-accent)]/10">
                  <HelpCircle class="h-4 w-4 text-[var(--color-accent)]" />
                </div>
                <div class="min-w-0 flex-1">
                  <p class="text-sm font-medium text-white">{entry.title}</p>
                  <p class="mt-1 text-xs text-[var(--color-muted)] line-clamp-2">{entry.content}</p>
                  <p class="mt-1 text-[10px] text-[var(--color-muted)]">
                    {entry.chunk_count} chunk{entry.chunk_count !== 1 ? 's' : ''} · {new Date(entry.created_at).toLocaleDateString()}
                  </p>
                </div>
                <button onclick={() => deleteEntry(entry.id)}
                  disabled={deletingId === entry.id}
                  class="shrink-0 rounded-md p-2 text-[var(--color-muted)] opacity-0 group-hover:opacity-100 hover:bg-red-500/10 hover:text-red-400 transition disabled:opacity-50">
                  {#if deletingId === entry.id}
                    <Loader2 class="h-4 w-4 animate-spin" />
                  {:else}
                    <Trash2 class="h-4 w-4" />
                  {/if}
                </button>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  {/if}

  <!-- Documents Tab -->
  {#if activeTab === 'documents'}
    <div class="space-y-6">
      <!-- Upload Area -->
      <div class="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-6">
        <h2 class="mb-4 text-lg font-semibold text-white flex items-center gap-2">
          <Upload class="h-5 w-5 text-[var(--color-accent)]" />
          Upload Document
        </h2>
        <p class="mb-4 text-xs text-[var(--color-muted)]">
          Upload a document (.txt, .md, .pdf). It will be chunked using the <strong class="text-[var(--color-accent-200)]">text-splitter</strong> framework and each chunk embedded for RAG retrieval.
        </p>

        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          class="relative flex flex-col items-center justify-center rounded-xl border-2 border-dashed px-6 py-12 transition cursor-pointer
            {dragOver
              ? 'border-[var(--color-accent)] bg-[var(--color-accent)]/5'
              : 'border-[var(--color-border)] hover:border-[var(--color-accent-700)] hover:bg-[var(--color-surface-2)]'}"
          ondragover={(e) => { e.preventDefault(); dragOver = true; }}
          ondragleave={() => (dragOver = false)}
          ondrop={handleDrop}
          onclick={() => fileInput?.click()}>
          <Upload class="h-8 w-8 text-[var(--color-muted)] mb-3" />
          <p class="text-sm text-[var(--color-muted-foreground)]">
            <span class="font-semibold text-[var(--color-accent)]">Click to upload</span> or drag and drop
          </p>
          <p class="mt-1 text-xs text-[var(--color-muted)]">.txt, .md, .pdf — max 10MB</p>
          <input bind:this={fileInput} type="file" accept=".txt,.md,.pdf" class="hidden" onchange={handleFileSelect} />
        </div>

        {#if uploadLoading}
          <div class="mt-4 flex items-center gap-2 text-sm text-[var(--color-accent)]">
            <Loader2 class="h-4 w-4 animate-spin" /> Extracting, chunking & embedding...
          </div>
        {/if}
        {#if uploadError}
          <p class="mt-3 text-sm text-red-400">{uploadError}</p>
        {/if}
        {#if uploadSuccess}
          <p class="mt-3 text-sm text-emerald-400">{uploadSuccess}</p>
        {/if}
      </div>

      <!-- Documents List -->
      <div class="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)]">
        <div class="border-b border-[var(--color-border)] px-6 py-4">
          <h2 class="text-sm font-semibold text-white">Uploaded Documents ({documents.length})</h2>
        </div>

        {#if documents.length === 0}
          <div class="px-6 py-12 text-center text-sm text-[var(--color-muted)]">
            No documents uploaded yet. Drag a file above to get started.
          </div>
        {:else}
          <div class="divide-y divide-[var(--color-border)]">
            {#each documents as entry (entry.id)}
              <div class="group flex items-start gap-4 px-6 py-4 hover:bg-[var(--color-surface-2)] transition">
                <div class="mt-0.5 flex h-8 w-8 shrink-0 items-center justify-center rounded-lg bg-blue-500/10">
                  <FileText class="h-4 w-4 text-blue-400" />
                </div>
                <div class="min-w-0 flex-1">
                  <p class="text-sm font-medium text-white">{entry.title}</p>
                  <p class="mt-1 text-xs text-[var(--color-muted)] line-clamp-1">{entry.content.slice(0, 200)}...</p>
                  <p class="mt-1 text-[10px] text-[var(--color-muted)]">
                    {entry.chunk_count} chunk{entry.chunk_count !== 1 ? 's' : ''} · {new Date(entry.created_at).toLocaleDateString()}
                  </p>
                </div>
                <button onclick={() => deleteEntry(entry.id)}
                  disabled={deletingId === entry.id}
                  class="shrink-0 rounded-md p-2 text-[var(--color-muted)] opacity-0 group-hover:opacity-100 hover:bg-red-500/10 hover:text-red-400 transition disabled:opacity-50">
                  {#if deletingId === entry.id}
                    <Loader2 class="h-4 w-4 animate-spin" />
                  {:else}
                    <Trash2 class="h-4 w-4" />
                  {/if}
                </button>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>
