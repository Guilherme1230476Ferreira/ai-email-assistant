<script lang="ts">
  import '../app.css';
  import { enhance } from '$app/forms';
  import { page } from '$app/state';
  import { authToken } from '$lib/stores/auth';
  import {
    LayoutDashboard, Mail, Users, Shield, Settings,
    LogOut, Sparkles, Menu, X, ShieldAlert, BookOpen,
    ChevronsLeft, ChevronsRight
  } from '@lucide/svelte';

  let { children, data } = $props();

  // Seed the in-memory token store whenever layout data refreshes
  $effect(() => {
    authToken.set(data.token ?? null);
  });

  let mobileSidebarOpen = $state(false);
  let sidebarCollapsed = $state(false);

  const userRole = $derived(data.user?.role || 'user');
  const isAdmin = $derived(userRole === 'admin');

  type NavItem = { label: string; href: string; icon: typeof LayoutDashboard; adminOnly?: boolean };

  const allNav: NavItem[] = [
    { label: 'Dashboard', href: '/', icon: LayoutDashboard },
    { label: 'Emails',    href: '/emails', icon: Mail },
    { label: 'Users',     href: '/users',  icon: Users,  adminOnly: true },
    { label: 'Roles',     href: '/roles',  icon: Shield, adminOnly: true },
    { label: 'Settings',  href: '/settings', icon: Settings, adminOnly: true },
    { label: 'Knowledge Base', href: '/knowledge', icon: BookOpen, adminOnly: true },
    { label: 'Audit Logs', href: '/audit-logs', icon: ShieldAlert, adminOnly: true }
  ];

  // Filter nav items based on user role
  let nav = $derived(allNav.filter(item => !item.adminOnly || isAdmin));

  function isActive(href: string) {
    if (href === '/') return page.url.pathname === '/';
    return page.url.pathname.startsWith(href);
  }

  function toggleCollapse() {
    sidebarCollapsed = !sidebarCollapsed;
  }
</script>

{#if page.url.pathname.startsWith('/login') || page.url.pathname.startsWith('/signup')}
  <div class="min-h-screen bg-black text-white">
    {@render children()}
  </div>
{:else}
<div class="flex min-h-screen w-full bg-black text-white">
  <!-- Mobile backdrop -->
  {#if mobileSidebarOpen}
    <button type="button" aria-label="Close menu"
      class="fixed inset-0 z-30 bg-black/70 backdrop-blur-sm lg:hidden"
      onclick={() => (mobileSidebarOpen = false)}></button>
  {/if}

  <!-- Sidebar -->
  <aside
    class="fixed inset-y-0 left-0 z-40 flex flex-col border-r border-[var(--color-border)] bg-[var(--color-surface)] transition-all duration-300 lg:translate-x-0
      {sidebarCollapsed ? 'w-[68px]' : 'w-60'}
      {mobileSidebarOpen ? 'translate-x-0' : '-translate-x-full lg:translate-x-0'}"
  >
    <!-- Logo row -->
    <div class="flex h-16 items-center border-b border-[var(--color-border)] {sidebarCollapsed ? 'justify-center px-2' : 'justify-between px-5'}">
      <a href="/" class="flex items-center gap-2.5">
        <div class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md bg-[var(--color-accent)] text-black">
          <Sparkles class="h-4 w-4" strokeWidth={2.5} />
        </div>
        {#if !sidebarCollapsed}
          <span class="text-sm font-semibold tracking-tight text-white">MailMate</span>
        {/if}
      </a>
      <!-- Mobile close -->
      {#if !sidebarCollapsed}
        <button type="button" aria-label="Close menu"
          class="rounded-md p-1.5 text-[var(--color-muted-foreground)] hover:bg-[var(--color-surface-2)] hover:text-white lg:hidden"
          onclick={() => (mobileSidebarOpen = false)}>
          <X class="h-5 w-5" />
        </button>
      {/if}
    </div>

    <!-- Navigation -->
    <nav class="flex-1 overflow-y-auto px-2 py-4">
      <ul class="space-y-0.5">
        {#each nav as item (item.href)}
          {@const active = isActive(item.href)}
          <li>
            <a href={item.href}
              title={sidebarCollapsed ? item.label : undefined}
              class="group relative flex items-center rounded-md transition
                {sidebarCollapsed ? 'justify-center px-2 py-2.5' : 'gap-3 px-3 py-2'}
                {active
                  ? 'bg-[var(--color-surface-2)] text-white'
                  : 'text-[var(--color-muted-foreground)] hover:bg-[var(--color-surface-2)] hover:text-white'}">
              {#if active}
                <span class="absolute left-0 top-1/2 h-4 w-0.5 -translate-y-1/2 rounded-r-full bg-[var(--color-accent)]"></span>
              {/if}
              <item.icon class="h-4 w-4 shrink-0 {active ? 'text-[var(--color-accent)]' : 'text-[var(--color-muted)] group-hover:text-white'}" />
              {#if !sidebarCollapsed}
                <span class="font-medium text-sm">{item.label}</span>
              {/if}
            </a>
          </li>
        {/each}
      </ul>
    </nav>

    <!-- Collapse toggle (desktop only) -->
    <div class="hidden lg:flex border-t border-[var(--color-border)] p-2 {sidebarCollapsed ? 'justify-center' : 'justify-end'}">
      <button type="button" aria-label={sidebarCollapsed ? 'Expand sidebar' : 'Collapse sidebar'}
        onclick={toggleCollapse}
        class="rounded-md p-1.5 text-[var(--color-muted)] hover:bg-[var(--color-surface-2)] hover:text-white transition-colors">
        {#if sidebarCollapsed}
          <ChevronsRight class="h-4 w-4" />
        {:else}
          <ChevronsLeft class="h-4 w-4" />
        {/if}
      </button>
    </div>

    <!-- User footer -->
    <div class="border-t border-[var(--color-border)] p-2">
      {#if data.user}
        <div class="flex items-center rounded-md {sidebarCollapsed ? 'justify-center px-1 py-2' : 'gap-3 px-2 py-2'}">
          <div class="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-[var(--color-surface-3)] text-[11px] font-semibold uppercase text-white">
            {data.user.initials}
          </div>
          {#if !sidebarCollapsed}
            <div class="min-w-0 flex-1">
              <p class="truncate text-xs font-medium text-white">{data.user.email}</p>
              <p class="truncate text-[10px] capitalize text-[var(--color-muted)]">{data.user.role || 'user'}</p>
            </div>
            <!-- Logout -->
            <form method="POST" action="/logout" use:enhance>
              <button type="submit" aria-label="Sign out"
                class="rounded-md p-1.5 text-[var(--color-muted)] hover:bg-[var(--color-surface-2)] hover:text-white">
                <LogOut class="h-4 w-4" />
              </button>
            </form>
          {/if}
        </div>
        {#if sidebarCollapsed}
          <form method="POST" action="/logout" use:enhance class="flex justify-center mt-1">
            <button type="submit" aria-label="Sign out"
              class="rounded-md p-1.5 text-[var(--color-muted)] hover:bg-[var(--color-surface-2)] hover:text-white">
              <LogOut class="h-4 w-4" />
            </button>
          </form>
        {/if}
      {:else}
        <div class="flex items-center justify-center rounded-md px-2 py-2">
          <a href="/login" class="text-xs text-white hover:text-[var(--color-accent)]">Sign In</a>
        </div>
      {/if}
    </div>
  </aside>

  <!-- Main content area — margin shifts based on sidebar width -->
  <div class="flex flex-1 flex-col transition-all duration-300 {sidebarCollapsed ? 'lg:pl-[68px]' : 'lg:pl-60'}">
    <header class="sticky top-0 z-20 flex h-16 items-center gap-3 border-b border-[var(--color-border)] bg-black/80 px-4 backdrop-blur-md sm:px-6 lg:px-8">
      <button type="button" aria-label="Open menu"
        class="rounded-md p-2 text-[var(--color-muted-foreground)] hover:bg-[var(--color-surface-2)] hover:text-white lg:hidden"
        onclick={() => (mobileSidebarOpen = !mobileSidebarOpen)}>
        <Menu class="h-5 w-5" />
      </button>
      <div class="flex-1"></div>
      <div class="flex items-center gap-2 text-xs text-[var(--color-muted)]">
        {#if !isAdmin}
          <span class="inline-flex items-center gap-1.5 rounded-full bg-[var(--color-surface-2)] border border-[var(--color-border)] px-2.5 py-1 text-[10px] font-medium text-[var(--color-muted-foreground)]">
            <Mail class="h-3 w-3" /> User
          </span>
        {:else}
          <span class="inline-flex items-center gap-1.5 rounded-full bg-[var(--color-accent)]/10 border border-[var(--color-accent)]/20 px-2.5 py-1 text-[10px] font-medium text-[var(--color-accent)]">
            <Shield class="h-3 w-3" /> Admin
          </span>
        {/if}
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
