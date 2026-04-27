<script lang="ts">
  import '../app.css';
  import { page } from '$app/state';
  import {
    LayoutDashboard,
    Mail,
    Users,
    Shield,
    Settings,
    LogOut,
    Sparkles,
    Menu,
    X
  } from '@lucide/svelte';

  let { children, data } = $props();

  let sidebarOpen = $state(false);

  type NavItem = {
    label: string;
    href: string;
    icon: typeof LayoutDashboard;
  };

  const nav: NavItem[] = [
    { label: 'Dashboard', href: '/', icon: LayoutDashboard },
    { label: 'Emails', href: '/emails', icon: Mail },
    { label: 'Users', href: '/users', icon: Users },
    { label: 'Roles', href: '/roles', icon: Shield },
    { label: 'Settings', href: '/settings', icon: Settings }
  ];

  function isActive(href: string) {
    if (href === '/') return page.url.pathname === '/';
    return page.url.pathname.startsWith(href);
  }
</script>

{#if page.url.pathname.startsWith('/login') || page.url.pathname.startsWith('/signup')}
  <div class="min-h-screen bg-black text-white">
    {@render children()}
  </div>
{:else}
<div class="flex min-h-screen w-full bg-black text-white">
  {#if sidebarOpen}
    <button
      type="button"
      aria-label="Close menu"
      class="fixed inset-0 z-30 bg-black/70 backdrop-blur-sm lg:hidden"
      onclick={() => (sidebarOpen = false)}
    ></button>
  {/if}

  <aside
    class="fixed inset-y-0 left-0 z-40 flex w-60 flex-col border-r border-[var(--color-border)] bg-[var(--color-surface)] transition-transform duration-300 lg:translate-x-0 {sidebarOpen
      ? 'translate-x-0'
      : '-translate-x-full'}"
  >
    <div class="flex h-16 items-center justify-between border-b border-[var(--color-border)] px-5">
      <a href="/" class="flex items-center gap-2.5">
        <div
          class="flex h-8 w-8 items-center justify-center rounded-md bg-[var(--color-accent)] text-black"
        >
          <Sparkles class="h-4 w-4" strokeWidth={2.5} />
        </div>
        <span class="text-sm font-semibold tracking-tight text-white">Mailwise</span>
      </a>
      <button
        type="button"
        aria-label="Close menu"
        class="rounded-md p-1.5 text-[var(--color-muted-foreground)] hover:bg-[var(--color-surface-2)] hover:text-white lg:hidden"
        onclick={() => (sidebarOpen = false)}
      >
        <X class="h-5 w-5" />
      </button>
    </div>

    <nav class="flex-1 overflow-y-auto px-3 py-4">
      <ul class="space-y-0.5">
        {#each nav as item (item.href)}
          {@const active = isActive(item.href)}
          <li>
            <a
              href={item.href}
              class="group relative flex items-center gap-3 rounded-md px-3 py-2 text-sm transition {active
                ? 'bg-[var(--color-surface-2)] text-white'
                : 'text-[var(--color-muted-foreground)] hover:bg-[var(--color-surface-2)] hover:text-white'}"
            >
              {#if active}
                <span
                  class="absolute left-0 top-1/2 h-4 w-0.5 -translate-y-1/2 rounded-r-full bg-[var(--color-accent)]"
                ></span>
              {/if}
              <item.icon
                class="h-4 w-4 {active
                  ? 'text-[var(--color-accent)]'
                  : 'text-[var(--color-muted)] group-hover:text-white'}"
              />
              <span class="font-medium">{item.label}</span>
            </a>
          </li>
        {/each}
      </ul>
    </nav>

    <div class="border-t border-[var(--color-border)] p-3">
      {#if data.user}
      <div class="flex items-center gap-3 rounded-md px-2 py-2">
        <div
          class="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-[var(--color-surface-3)] text-[11px] font-semibold text-white uppercase"
        >
          {data.user.initials}
        </div>
        <div class="min-w-0 flex-1">
          <p class="truncate text-xs font-medium text-white">{data.user.email}</p>
          <p class="truncate text-[10px] text-[var(--color-muted)] capitalize">{data.user.role || 'user'}</p>
        </div>
        <button
          type="button"
          aria-label="Sign out"
          class="rounded-md p-1.5 text-[var(--color-muted)] hover:bg-[var(--color-surface-2)] hover:text-white"
          onclick={() => {
            document.cookie = 'token=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/;';
            window.location.href = '/login';
          }}
        >
          <LogOut class="h-4 w-4" />
        </button>
      </div>
      {:else}
      <div class="flex items-center justify-center rounded-md px-2 py-2">
        <a href="/login" class="text-xs text-white hover:text-[var(--color-accent)]">Sign In</a>
      </div>
      {/if}
    </div>
  </aside>

  <div class="flex flex-1 flex-col lg:pl-60">
    <header
      class="sticky top-0 z-20 flex h-16 items-center gap-3 border-b border-[var(--color-border)] bg-black/80 px-4 backdrop-blur-md sm:px-6 lg:px-8"
    >
      <button
        type="button"
        aria-label="Open menu"
        class="rounded-md p-2 text-[var(--color-muted-foreground)] hover:bg-[var(--color-surface-2)] hover:text-white lg:hidden"
        onclick={() => (sidebarOpen = !sidebarOpen)}
      >
        <Menu class="h-5 w-5" />
      </button>

      <div class="flex-1"></div>

      <div class="flex items-center gap-2 text-xs text-[var(--color-muted)]">
        <span class="inline-flex h-1.5 w-1.5 rounded-full bg-[var(--color-success)]"></span>
        <span>Connected</span>
      </div>
    </header>

    <main class="flex-1 px-4 py-8 sm:px-6 lg:px-8">
      {@render children()}
    </main>
  </div>
</div>
{/if}
