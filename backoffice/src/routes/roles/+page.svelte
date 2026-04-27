<script lang="ts">
  import type { PageData } from './$types';
  import { invalidateAll } from '$app/navigation';
  import { Plus, X, Loader2 } from '@lucide/svelte';

  let { data }: { data: PageData } = $props();

  let isModalOpen = $state(false);
  let newRoleName = $state('');
  let isCreating = $state(false);

  function getToken() {
    const match = document.cookie.match(/(^| )token=([^;]+)/);
    return match ? match[2] : null;
  }

  async function handleCreateRole(e: Event) {
    e.preventDefault();
    if (!newRoleName.trim()) return;

    isCreating = true;
    try {
      const res = await fetch('/api/admin/roles', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${getToken()}`
        },
        body: JSON.stringify({ name: newRoleName })
      });

      if (res.ok) {
        newRoleName = '';
        isModalOpen = false;
        await invalidateAll();
      } else {
        console.error('Failed to create role');
      }
    } catch (err) {
      console.error(err);
    } finally {
      isCreating = false;
    }
  }
</script>

<svelte:head>
  <title>Roles · Mailwise</title>
</svelte:head>

<div class="mx-auto w-full max-w-5xl space-y-8">
  <header class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 mb-8">
    <div>
      <h1 class="text-2xl font-semibold tracking-tight text-white">Roles</h1>
      <p class="text-sm text-[var(--color-muted-foreground)]">Manage system permissions and roles.</p>
    </div>
    <button
      onclick={() => (isModalOpen = true)}
      class="flex items-center gap-2 rounded-md bg-white px-4 py-2.5 text-sm font-semibold text-black transition hover:bg-gray-200"
    >
      <Plus class="h-4 w-4" />
      New Role
    </button>
  </header>

  <div class="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)]">
    <ul class="divide-y divide-[var(--color-border)]">
      {#each data.roles || [] as role (role.id)}
        <li class="flex items-center justify-between px-5 py-4">
          <p class="text-sm font-medium text-white uppercase">{role.name}</p>
        </li>
      {:else}
        <li class="px-5 py-10 text-center text-sm text-[var(--color-muted-foreground)]">No roles found.</li>
      {/each}
    </ul>
  </div>
</div>

{#if isModalOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-sm p-4">
    <div class="w-full max-w-md rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-6 shadow-2xl">
      <div class="flex items-center justify-between mb-4">
        <h2 class="text-lg font-semibold text-white">Add New Role</h2>
        <button onclick={() => (isModalOpen = false)} class="text-[var(--color-muted)] hover:text-white">
          <X class="h-5 w-5" />
        </button>
      </div>
      <form onsubmit={handleCreateRole}>
        <div class="space-y-4">
          <div>
            <label for="roleName" class="block text-sm font-medium text-[var(--color-muted-foreground)] mb-1">
              Role Name
            </label>
            <input
              id="roleName"
              type="text"
              required
              bind:value={newRoleName}
              class="w-full rounded-md border border-[var(--color-border)] bg-[var(--color-surface-2)] px-3 py-2 text-sm text-white focus:border-white focus:outline-none focus:ring-1 focus:ring-white"
              placeholder="E.g., editor, manager..."
            />
          </div>
          <div class="flex justify-end gap-3 pt-4">
            <button
              type="button"
              onclick={() => (isModalOpen = false)}
              class="rounded-md px-4 py-2 text-sm font-medium text-white hover:bg-[var(--color-surface-2)]"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={isCreating}
              class="flex items-center justify-center gap-2 rounded-md bg-white px-4 py-2 text-sm font-semibold text-black transition hover:bg-gray-200 disabled:opacity-50"
            >
              {#if isCreating}
                <Loader2 class="h-4 w-4 animate-spin" />
                Creating...
              {:else}
                <Plus class="h-4 w-4" />
                Create Role
              {/if}
            </button>
          </div>
        </div>
      </form>
    </div>
  </div>
{/if}