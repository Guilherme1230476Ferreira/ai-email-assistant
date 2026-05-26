<script lang="ts">
  import { Download, Trash2, Shield, User, AlertTriangle, Loader2 } from '@lucide/svelte';
  import { api } from '$lib/api';
  import { goto } from '$app/navigation';
  import type { PageData } from './$types';

  let { data }: { data: PageData } = $props();

  let exportLoading = $state(false);
  let deleteLoading = $state(false);
  let confirmDelete = $state(false);
  let confirmText = $state('');
  let errorMsg = $state<string | null>(null);

  async function handleExport() {
    exportLoading = true;
    try {
      const res = await fetch('/api/auth/me/data', {
        headers: { Authorization: `Bearer ${data.token}` }
      });
      if (!res.ok) throw new Error(await res.text());
      const blob = await res.blob();
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = 'my-data.json';
      a.click();
      URL.revokeObjectURL(url);
    } catch (e: any) {
      errorMsg = e.message ?? 'Export failed';
    } finally {
      exportLoading = false;
    }
  }

  async function handleDelete() {
    if (confirmText !== 'DELETE') return;
    deleteLoading = true;
    errorMsg = null;
    try {
      const res = await fetch('/api/auth/me', {
        method: 'DELETE',
        headers: { Authorization: `Bearer ${data.token}` }
      });
      if (res.status !== 204) throw new Error(await res.text());
      // Account deleted — redirect to login
      await goto('/login?deleted=1');
    } catch (e: any) {
      errorMsg = e.message ?? 'Deletion failed';
      deleteLoading = false;
    }
  }
</script>

<svelte:head>
  <title>My Account — AI Email Assistant</title>
</svelte:head>

<div class="account-root">
  <div class="account-container">

    <!-- Header -->
    <div class="account-header">
      <div class="account-header-icon">
        <User class="w-6 h-6" />
      </div>
      <div>
        <h1 class="account-title">My Account</h1>
        <p class="account-subtitle">Manage your personal data — GDPR self-service</p>
      </div>
    </div>

    <!-- Profile info -->
    <section class="account-card">
      <div class="account-card-header">
        <Shield class="w-4 h-4 text-[var(--color-accent)]" />
        <h2 class="account-card-title">Account Details</h2>
      </div>
      <div class="account-card-body">
        <div class="account-info-row">
          <span class="account-info-label">Email</span>
          <span class="account-info-value">{data.user?.email ?? '—'}</span>
        </div>
        <div class="account-info-row">
          <span class="account-info-label">Role</span>
          <span class="account-info-value">{data.user?.role ?? '—'}</span>
        </div>
      </div>
    </section>

    <!-- Data export -->
    <section class="account-card">
      <div class="account-card-header">
        <Download class="w-4 h-4 text-blue-400" />
        <h2 class="account-card-title">Export My Data</h2>
        <span class="account-right-badge account-right-badge--blue">GDPR Art. 15</span>
      </div>
      <div class="account-card-body">
        <p class="account-card-desc">
          Download a JSON file containing all personal data we hold for your account:
          your profile, all emails you have generated, and metadata.
        </p>
        <button
          onclick={handleExport}
          disabled={exportLoading}
          class="account-btn account-btn--primary"
        >
          {#if exportLoading}
            <Loader2 class="w-4 h-4 animate-spin" />
            Preparing export…
          {:else}
            <Download class="w-4 h-4" />
            Download my data (JSON)
          {/if}
        </button>
      </div>
    </section>

    <!-- Account deletion -->
    <section class="account-card account-card--danger">
      <div class="account-card-header">
        <Trash2 class="w-4 h-4 text-red-400" />
        <h2 class="account-card-title account-card-title--danger">Delete My Account</h2>
        <span class="account-right-badge account-right-badge--red">GDPR Art. 17</span>
      </div>
      <div class="account-card-body">
        <div class="account-danger-warning">
          <AlertTriangle class="w-4 h-4 flex-shrink-0 mt-0.5" />
          <div>
            <p><strong>This is irreversible.</strong> Deleting your account permanently removes:</p>
            <ul>
              <li>Your profile and login credentials</li>
              <li>All emails you have generated and their AI embeddings</li>
            </ul>
            <p>Audit log entries are anonymised (your user ID is removed but the action record is kept for security purposes).</p>
          </div>
        </div>

        {#if !confirmDelete}
          <button
            onclick={() => { confirmDelete = true; }}
            class="account-btn account-btn--danger-outline"
          >
            <Trash2 class="w-4 h-4" />
            I want to delete my account
          </button>
        {:else}
          <div class="account-confirm-block">
            <p class="account-confirm-label">Type <strong>DELETE</strong> to confirm:</p>
            <input
              type="text"
              bind:value={confirmText}
              placeholder="DELETE"
              class="account-confirm-input"
            />
            <div class="account-confirm-actions">
              <button
                onclick={() => { confirmDelete = false; confirmText = ''; }}
                class="account-btn account-btn--ghost"
              >
                Cancel
              </button>
              <button
                onclick={handleDelete}
                disabled={deleteLoading || confirmText !== 'DELETE'}
                class="account-btn account-btn--danger"
              >
                {#if deleteLoading}
                  <Loader2 class="w-4 h-4 animate-spin" />
                  Deleting…
                {:else}
                  <Trash2 class="w-4 h-4" />
                  Permanently delete my account
                {/if}
              </button>
            </div>
          </div>
        {/if}

        {#if errorMsg}
          <p class="account-error">{errorMsg}</p>
        {/if}
      </div>
    </section>

    <!-- Privacy policy link -->
    <p class="account-footer-note">
      Read our full <a href="/privacy" class="account-footer-link">Privacy Policy</a>
      to understand how your data is used.
    </p>

  </div>
</div>

<style>
  .account-root {
    min-height: 100vh;
    padding: 2rem 1rem 4rem;
    background: var(--color-bg, #0a0a0f);
  }

  .account-container {
    max-width: 640px;
    margin: 0 auto;
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
  }

  /* Header */
  .account-header {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-bottom: 0.5rem;
  }

  .account-header-icon {
    width: 3rem;
    height: 3rem;
    border-radius: 0.75rem;
    background: var(--color-surface-2, rgba(255,255,255,0.05));
    border: 1px solid var(--color-border, rgba(255,255,255,0.08));
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-accent, #22d3ee);
    flex-shrink: 0;
  }

  .account-title {
    font-size: 1.5rem;
    font-weight: 800;
    color: white;
    margin: 0;
  }

  .account-subtitle {
    font-size: 0.85rem;
    color: var(--color-muted, #94a3b8);
    margin: 0.15rem 0 0;
  }

  /* Cards */
  .account-card {
    border-radius: 0.875rem;
    border: 1px solid var(--color-border, rgba(255,255,255,0.08));
    background: var(--color-surface, rgba(255,255,255,0.03));
    overflow: hidden;
  }

  .account-card--danger {
    border-color: rgba(239,68,68,0.2);
  }

  .account-card-header {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    padding: 1rem 1.25rem;
    border-bottom: 1px solid var(--color-border, rgba(255,255,255,0.08));
    background: var(--color-surface-2, rgba(255,255,255,0.02));
  }

  .account-card-title {
    font-size: 0.9rem;
    font-weight: 700;
    color: white;
    margin: 0;
    flex: 1;
  }

  .account-card-title--danger { color: #fca5a5; }

  .account-right-badge {
    font-size: 0.68rem;
    font-weight: 600;
    padding: 0.2rem 0.5rem;
    border-radius: 999px;
    border: 1px solid;
  }

  .account-right-badge--blue {
    color: #60a5fa;
    background: rgba(59,130,246,0.1);
    border-color: rgba(59,130,246,0.25);
  }

  .account-right-badge--red {
    color: #f87171;
    background: rgba(239,68,68,0.1);
    border-color: rgba(239,68,68,0.25);
  }

  .account-card-body {
    padding: 1.25rem;
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
  }

  .account-card-desc {
    font-size: 0.875rem;
    color: var(--color-muted-foreground, #cbd5e1);
    line-height: 1.6;
    margin: 0;
  }

  /* Info rows */
  .account-info-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.6rem 0;
    border-bottom: 1px solid rgba(255,255,255,0.04);
    font-size: 0.875rem;
  }

  .account-info-row:last-child { border-bottom: none; }

  .account-info-label {
    color: var(--color-muted, #94a3b8);
    font-weight: 500;
  }

  .account-info-value {
    color: white;
    font-weight: 600;
  }

  /* Danger warning */
  .account-danger-warning {
    display: flex;
    gap: 0.75rem;
    padding: 0.85rem 1rem;
    border-radius: 0.6rem;
    background: rgba(239,68,68,0.06);
    border: 1px solid rgba(239,68,68,0.15);
    color: #fca5a5;
    font-size: 0.825rem;
    line-height: 1.55;
  }

  .account-danger-warning ul {
    margin: 0.35rem 0;
    padding-left: 1.1rem;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .account-danger-warning p { margin: 0; }

  .account-danger-warning strong { color: #f87171; }

  /* Confirm block */
  .account-confirm-block {
    display: flex;
    flex-direction: column;
    gap: 0.65rem;
  }

  .account-confirm-label {
    font-size: 0.825rem;
    color: var(--color-muted-foreground, #cbd5e1);
    margin: 0;
  }

  .account-confirm-label strong { color: white; }

  .account-confirm-input {
    padding: 0.6rem 0.85rem;
    border-radius: 0.5rem;
    border: 1px solid rgba(239,68,68,0.3);
    background: rgba(239,68,68,0.05);
    color: white;
    font-size: 0.875rem;
    width: 100%;
    outline: none;
  }

  .account-confirm-input:focus {
    border-color: rgba(239,68,68,0.55);
    box-shadow: 0 0 0 2px rgba(239,68,68,0.1);
  }

  .account-confirm-actions {
    display: flex;
    gap: 0.65rem;
    justify-content: flex-end;
    flex-wrap: wrap;
  }

  /* Buttons */
  .account-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.6rem 1.1rem;
    border-radius: 0.5rem;
    font-size: 0.875rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s;
    border: 1px solid transparent;
  }

  .account-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .account-btn--primary {
    background: var(--color-accent, #22d3ee);
    color: #000;
  }

  .account-btn--primary:hover:not(:disabled) {
    filter: brightness(1.1);
  }

  .account-btn--danger {
    background: #dc2626;
    color: white;
  }

  .account-btn--danger:hover:not(:disabled) {
    background: #b91c1c;
  }

  .account-btn--danger-outline {
    background: transparent;
    border-color: rgba(239,68,68,0.35);
    color: #f87171;
  }

  .account-btn--danger-outline:hover {
    background: rgba(239,68,68,0.08);
    border-color: rgba(239,68,68,0.55);
  }

  .account-btn--ghost {
    background: transparent;
    border-color: var(--color-border, rgba(255,255,255,0.1));
    color: var(--color-muted-foreground, #cbd5e1);
  }

  .account-btn--ghost:hover {
    background: rgba(255,255,255,0.04);
  }

  .account-error {
    font-size: 0.8rem;
    color: #f87171;
    background: rgba(239,68,68,0.07);
    border: 1px solid rgba(239,68,68,0.2);
    border-radius: 0.4rem;
    padding: 0.5rem 0.75rem;
    margin: 0;
  }

  /* Footer note */
  .account-footer-note {
    font-size: 0.8rem;
    color: var(--color-muted, #94a3b8);
    text-align: center;
    margin-top: 0.5rem;
  }

  .account-footer-link {
    color: var(--color-accent, #22d3ee);
    text-decoration: none;
    border-bottom: 1px solid rgba(34,211,238,0.3);
  }

  .account-footer-link:hover {
    border-color: var(--color-accent, #22d3ee);
  }
</style>
