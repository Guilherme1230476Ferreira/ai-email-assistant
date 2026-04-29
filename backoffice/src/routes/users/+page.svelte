<script lang="ts">
  import type { PageData } from './$types';
  import { invalidateAll, goto } from '$app/navigation';
  import { Plus, X, Loader2, Settings2, Trash2 } from '@lucide/svelte';
  import { api, type ApiUser, type ApiRole } from '$lib/api';

  let { data }: { data: PageData } = $props();

  // ── Create user modal ──────────────────────────────────────────────────────
  let isCreateOpen = $state(false);
  let newEmail = $state('');
  let newPassword = $state('');
  let isCreating = $state(false);
  let createError = $state<string | null>(null);

  function goToPage(page: number) {
      goto(`/users?page=${page}&limit=${data.pagination.limit}`);
  }

  async function handleCreateUser(e: Event) {
    e.preventDefault();
    createError = null;
    isCreating = true;
    const { error } = await api.createUser(newEmail, newPassword);
    isCreating = false;
    if (error) { createError = error; return; }
    newEmail = '';
    newPassword = '';
    isCreateOpen = false;
    await invalidateAll();
  }

  // ── Manage user modal ──────────────────────────────────────────────────────
  let managedUser = $state<ApiUser | null>(null);
  let selectedRoleId = $state('');
  let isSavingRole = $state(false);
  let manageError = $state<string | null>(null);

  function openManage(user: ApiUser) {
    managedUser = user;
    selectedRoleId = user.role_id;
    manageError = null;
  }

  async function handleSaveRole(e: Event) {
    e.preventDefault();
    if (!managedUser) return;
    manageError = null;
    isSavingRole = true;
    const { error } = await api.updateUserRole(managedUser.id, selectedRoleId);
    isSavingRole = false;
    if (error) { manageError = error; return; }
    managedUser = null;
    await invalidateAll();
  }

  // ── Delete user ────────────────────────────────────────────────────────────
  let deletingUserId = $state<string | null>(null);
  let deleteError = $state<string | null>(null);

  async function handleDeleteUser(userId: string) {
    deleteError = null;
    deletingUserId = userId;
    const { error } = await api.deleteUser(userId);
    deletingUserId = null;
    if (error) { deleteError = error; return; }
    await invalidateAll();
  }

  // ── Helpers ────────────────────────────────────────────────────────────────
  function roleName(roleId: string): string {
    const role = (data.roles as ApiRole[]).find((r) => r.id === roleId);
    return role ? role.name : roleId.substring(0, 8) + '…';
  }
</script>

<svelte:head>
  <title>Users · Mailwise</title>
</svelte:head>

<div class="mx-auto w-full max-w-5xl space-y-8">
  <header class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 mb-8">
    <div>
      <h1 class="text-2xl font-semibold tracking-tight text-white">Users</h1>
      <p class="text-sm text-[var(--color-muted-foreground)]">Manage system users.</p>
    </div>
    <button
      onclick={() => { isCreateOpen = true; createError = null; }}
      class="flex items-center gap-2 rounded-md bg-white px-4 py-2.5 text-sm font-semibold text-black transition hover:bg-gray-200"
    >
      <Plus class="h-4 w-4" />
      Add User
    </button>
  </header>

  <!-- Global delete error -->
  {#if deleteError}
    <div class="rounded-md border border-red-500/30 bg-red-500/8 px-4 py-3 text-sm text-red-300">
      {deleteError}
    </div>
  {/if}

  <div class="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)]">
    <ul class="divide-y divide-[var(--color-border)]">
      {#each data.users || [] as user (user.id)}
        <li class="flex items-center justify-between px-5 py-4 gap-4">
          <div class="min-w-0 flex-1">
            <p class="text-sm font-medium text-white truncate">{user.email}</p>
            <p class="text-xs text-[var(--color-muted-foreground)] capitalize mt-0.5">
              Role: <span class="text-[var(--color-accent)]">{roleName(user.role_id)}</span>
            </p>
          </div>
          <div class="flex items-center gap-2 shrink-0">
            <button
              onclick={() => openManage(user)}
              class="flex items-center gap-1.5 rounded-md border border-[var(--color-border)] px-3 py-1.5 text-xs font-medium text-white transition hover:border-[var(--color-accent)]/50 hover:text-[var(--color-accent)]"
            >
              <Settings2 class="h-3.5 w-3.5" />
              Manage
            </button>
            <button
              onclick={() => handleDeleteUser(user.id)}
              disabled={deletingUserId === user.id}
              class="flex items-center justify-center rounded-md border border-red-500/30 p-1.5 text-red-400 transition hover:bg-red-500/10 disabled:opacity-50"
              aria-label="Delete user"
            >
              {#if deletingUserId === user.id}
                <Loader2 class="h-3.5 w-3.5 animate-spin" />
              {:else}
                <Trash2 class="h-3.5 w-3.5" />
              {/if}
            </button>
          </div>
        </li>
      {:else}
        <li class="px-5 py-10 text-center text-sm text-[var(--color-muted-foreground)]">No users found.</li>
      {/each}
    </ul>
    
    <!-- Pagination -->
    {#if data.pagination && data.pagination.total > data.pagination.limit}
        <div class="px-5 py-4 border-t border-[var(--color-border)] flex items-center justify-between bg-[var(--color-surface-2)]/30">
            <span class="text-sm text-[var(--color-muted-foreground)]">
                Showing {((data.pagination.page - 1) * data.pagination.limit) + 1} to {Math.min(data.pagination.page * data.pagination.limit, data.pagination.total)} of {data.pagination.total} users
            </span>
            <div class="flex gap-2">
                <button 
                    disabled={data.pagination.page <= 1}
                    onclick={() => goToPage(data.pagination.page - 1)}
                    class="px-3 py-1.5 rounded-md text-sm font-medium border border-[var(--color-border)] bg-[var(--color-surface)] text-white hover:bg-[var(--color-surface-2)] disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                >
                    Previous
                </button>
                <button 
                    disabled={data.pagination.page * data.pagination.limit >= data.pagination.total}
                    onclick={() => goToPage(data.pagination.page + 1)}
                    class="px-3 py-1.5 rounded-md text-sm font-medium border border-[var(--color-border)] bg-[var(--color-surface)] text-white hover:bg-[var(--color-surface-2)] disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                >
                    Next
                </button>
            </div>
        </div>
    {/if}
  </div>
</div>

<!-- ── Create user modal ─────────────────────────────────────────────────── -->
{#if isCreateOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-sm p-4">
    <div class="w-full max-w-md rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-6 shadow-2xl">
      <div class="flex items-center justify-between mb-4">
        <h2 class="text-lg font-semibold text-white">Add New User</h2>
        <button onclick={() => (isCreateOpen = false)} class="text-[var(--color-muted)] hover:text-white">
          <X class="h-5 w-5" />
        </button>
      </div>

      {#if createError}
        <div class="mb-4 rounded-md border border-red-500/30 bg-red-500/8 px-4 py-3 text-sm text-red-300">
          {createError}
        </div>
      {/if}

      <form onsubmit={handleCreateUser}>
        <div class="space-y-4">
          <div>
            <label for="email" class="block text-sm font-medium text-[var(--color-muted-foreground)] mb-1">Email Address</label>
            <input id="email" type="email" required bind:value={newEmail}
              class="w-full rounded-md border border-[var(--color-border)] bg-[var(--color-surface-2)] px-3 py-2 text-sm text-white focus:border-[var(--color-accent)] focus:outline-none"
              placeholder="user@example.com" />
          </div>
          <div>
            <label for="password" class="block text-sm font-medium text-[var(--color-muted-foreground)] mb-1">Password</label>
            <input id="password" type="password" required bind:value={newPassword}
              class="w-full rounded-md border border-[var(--color-border)] bg-[var(--color-surface-2)] px-3 py-2 text-sm text-white focus:border-[var(--color-accent)] focus:outline-none"
              placeholder="••••••••" />
          </div>
          <div class="flex justify-end gap-3 pt-2">
            <button type="button" onclick={() => (isCreateOpen = false)}
              class="rounded-md px-4 py-2 text-sm font-medium text-white hover:bg-[var(--color-surface-2)]">
              Cancel
            </button>
            <button type="submit" disabled={isCreating}
              class="flex items-center justify-center gap-2 rounded-md bg-white px-4 py-2 text-sm font-semibold text-black transition hover:bg-gray-200 disabled:opacity-50">
              {#if isCreating}
                <Loader2 class="h-4 w-4 animate-spin" /> Adding...
              {:else}
                <Plus class="h-4 w-4" /> Add User
              {/if}
            </button>
          </div>
        </div>
      </form>
    </div>
  </div>
{/if}

<!-- ── Manage user modal ──────────────────────────────────────────────────── -->
{#if managedUser}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-sm p-4">
    <div class="w-full max-w-md rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-6 shadow-2xl">
      <div class="flex items-center justify-between mb-1">
        <h2 class="text-lg font-semibold text-white">Manage User</h2>
        <button onclick={() => (managedUser = null)} class="text-[var(--color-muted)] hover:text-white">
          <X class="h-5 w-5" />
        </button>
      </div>
      <p class="text-xs text-[var(--color-muted-foreground)] mb-5 truncate">{managedUser.email}</p>

      {#if manageError}
        <div class="mb-4 rounded-md border border-red-500/30 bg-red-500/8 px-4 py-3 text-sm text-red-300">
          {manageError}
        </div>
      {/if}

      <form onsubmit={handleSaveRole}>
        <p class="text-xs font-medium text-[var(--color-muted-foreground)] uppercase tracking-wider mb-3">Assign Role</p>
        <div class="space-y-2 mb-6">
          {#each data.roles as role (role.id)}
            {@const checked = selectedRoleId === role.id}
            <label
              class="flex items-center gap-3 rounded-lg border px-4 py-3 cursor-pointer transition
                {checked
                  ? 'border-[var(--color-accent)]/60 bg-[var(--color-accent)]/8'
                  : 'border-[var(--color-border)] hover:border-[var(--color-border-strong)]'}"
            >
              <div class="relative flex items-center justify-center">
                <input
                  type="radio"
                  name="role"
                  value={role.id}
                  bind:group={selectedRoleId}
                  class="sr-only"
                />
                <div class="h-4 w-4 rounded-full border-2 {checked ? 'border-[var(--color-accent)]' : 'border-[var(--color-muted)]'} flex items-center justify-center">
                  {#if checked}
                    <div class="h-2 w-2 rounded-full bg-[var(--color-accent)]"></div>
                  {/if}
                </div>
              </div>
              <span class="text-sm font-medium capitalize {checked ? 'text-white' : 'text-[var(--color-muted-foreground)]'}">{role.name}</span>
              {#if checked}
                <span class="ml-auto text-[10px] uppercase tracking-wider text-[var(--color-accent)] font-semibold">Current</span>
              {/if}
            </label>
          {/each}
        </div>

        <div class="flex justify-end gap-3">
          <button type="button" onclick={() => (managedUser = null)}
            class="rounded-md px-4 py-2 text-sm font-medium text-white hover:bg-[var(--color-surface-2)]">
            Cancel
          </button>
          <button type="submit" disabled={isSavingRole || selectedRoleId === managedUser.role_id}
            class="flex items-center justify-center gap-2 rounded-md bg-[var(--color-accent)] px-4 py-2 text-sm font-semibold text-black transition hover:bg-[var(--color-accent-hover)] disabled:opacity-50">
            {#if isSavingRole}
              <Loader2 class="h-4 w-4 animate-spin" /> Saving...
            {:else}
              Save Role
            {/if}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}