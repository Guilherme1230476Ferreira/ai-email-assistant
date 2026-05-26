<script lang="ts">
  import '../app.css';
  import { enhance } from '$app/forms';
  import { page } from '$app/state';
  import { authToken } from '$lib/stores/auth';
  import { locale, t, type Locale } from '$lib/i18n';
  import {
    LayoutDashboard, Mail, Users, Shield, Settings,
    LogOut, Sparkles, Menu, X, ShieldAlert, BookOpen,
    ChevronsLeft, ChevronsRight, Globe, Puzzle
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

  // Current locale for reactive translations
  let currentLocale = $state<Locale>($locale);
  locale.subscribe((val) => (currentLocale = val));

  function toggleLocale() {
    locale.set(currentLocale === 'en' ? 'pt' : 'en');
  }

  type NavItem = { key: string; href: string; icon: typeof LayoutDashboard; adminOnly?: boolean };

  const allNav: NavItem[] = [
    { key: 'nav.dashboard', href: '/', icon: LayoutDashboard },
    { key: 'nav.emails',    href: '/emails', icon: Mail },
    { key: 'nav.users',     href: '/users',  icon: Users,  adminOnly: true },
    { key: 'nav.roles',     href: '/roles',  icon: Shield, adminOnly: true },
    { key: 'nav.settings',  href: '/settings', icon: Settings, adminOnly: true },
    { key: 'nav.knowledge', href: '/knowledge', icon: BookOpen, adminOnly: true },
    { key: 'nav.audit',     href: '/audit-logs', icon: ShieldAlert, adminOnly: true }
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

  // Extension Status Widget
  let extensionConnected = $state(false);

  async function checkExtensionStatus() {
    try {
      const { get } = await import('svelte/store');
      const { authToken } = await import('$lib/stores/auth');
      const tok = get(authToken);
      if (!tok) { extensionConnected = false; return; }
      const res = await fetch('/api/extension/status', {
        headers: { Authorization: `Bearer ${tok}` }
      });
      if (res.ok) {
        const json = await res.json();
        extensionConnected = json.connected ?? false;
      }
    } catch {
      extensionConnected = false;
    }
  }

  $effect(() => {
    checkExtensionStatus();
    const interval = setInterval(checkExtensionStatus, 30_000);
    return () => clearInterval(interval);
  });
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
              title={sidebarCollapsed ? t(item.key, currentLocale) : undefined}
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
                <span class="font-medium text-sm">{t(item.key, currentLocale)}</span>
              {/if}
            </a>
          </li>
        {/each}
      </ul>
    </nav>

    <!-- Collapse toggle + Language toggle (desktop only) -->
    <div class="hidden lg:flex items-center border-t border-[var(--color-border)] p-2 {sidebarCollapsed ? 'flex-col gap-1' : 'justify-between'}">
      <!-- Language toggle -->
      <button type="button" aria-label="Toggle language"
        onclick={toggleLocale}
        title={currentLocale === 'en' ? 'Switch to Portuguese' : 'Mudar para Inglês'}
        class="flex items-center gap-1.5 rounded-md px-2 py-1.5 text-[var(--color-muted)] hover:bg-[var(--color-surface-2)] hover:text-white transition-colors">
        <Globe class="h-3.5 w-3.5" />
        {#if !sidebarCollapsed}
          <span class="text-[11px] font-bold tracking-wider">
            <span class="{currentLocale === 'en' ? 'text-[var(--color-accent)]' : 'text-[var(--color-muted)]'}">EN</span>
            <span class="text-[var(--color-muted)] mx-0.5">|</span>
            <span class="{currentLocale === 'pt' ? 'text-[var(--color-accent)]' : 'text-[var(--color-muted)]'}">PT</span>
          </span>
        {:else}
          <span class="text-[10px] font-bold text-[var(--color-accent)]">{currentLocale.toUpperCase()}</span>
        {/if}
      </button>

      <!-- Collapse toggle -->
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
              <button type="submit" aria-label={t('nav.signout', currentLocale)}
                class="rounded-md p-1.5 text-[var(--color-muted)] hover:bg-[var(--color-surface-2)] hover:text-white">
                <LogOut class="h-4 w-4" />
              </button>
            </form>
          {/if}
        </div>
        {#if sidebarCollapsed}
          <form method="POST" action="/logout" use:enhance class="flex justify-center mt-1">
            <button type="submit" aria-label={t('nav.signout', currentLocale)}
              class="rounded-md p-1.5 text-[var(--color-muted)] hover:bg-[var(--color-surface-2)] hover:text-white">
              <LogOut class="h-4 w-4" />
            </button>
          </form>
        {/if}
      {:else}
        <div class="flex items-center justify-center rounded-md px-2 py-2">
          <a href="/login" class="text-xs text-white hover:text-[var(--color-accent)]">{t('nav.signin', currentLocale)}</a>
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

      <!-- Mobile language toggle -->
      <button type="button" aria-label="Toggle language"
        onclick={toggleLocale}
        class="flex items-center gap-1 rounded-md px-2 py-1 text-[var(--color-muted)] hover:bg-[var(--color-surface-2)] hover:text-white transition-colors lg:hidden">
        <Globe class="h-3.5 w-3.5" />
        <span class="text-[11px] font-bold">
          <span class="{currentLocale === 'en' ? 'text-[var(--color-accent)]' : ''}">EN</span>
          <span class="mx-0.5">|</span>
          <span class="{currentLocale === 'pt' ? 'text-[var(--color-accent)]' : ''}">PT</span>
        </span>
      </button>

      <div class="flex-1"></div>
      <div class="flex items-center gap-2 text-xs text-[var(--color-muted)]">
        {#if !isAdmin}
          <span class="inline-flex items-center gap-1.5 rounded-full bg-[var(--color-surface-2)] border border-[var(--color-border)] px-2.5 py-1 text-[10px] font-medium text-[var(--color-muted-foreground)]">
            <Mail class="h-3 w-3" /> {t('header.user', currentLocale)}
          </span>
        {:else}
          <span class="inline-flex items-center gap-1.5 rounded-full bg-[var(--color-accent)]/10 border border-[var(--color-accent)]/20 px-2.5 py-1 text-[10px] font-medium text-[var(--color-accent)]">
            <Shield class="h-3 w-3" /> {t('header.admin', currentLocale)}
          </span>
        {/if}

        <!-- Extension Status Widget -->
        <span
          title={extensionConnected ? t('ext.tooltip_on', currentLocale) : t('ext.tooltip_off', currentLocale)}
          class="inline-flex items-center gap-1.5 rounded-full px-2.5 py-1 text-[10px] font-medium border transition-all duration-300
            {extensionConnected
              ? 'bg-emerald-500/10 border-emerald-500/30 text-emerald-400'
              : 'bg-[var(--color-surface-2)] border-[var(--color-border)] text-[var(--color-muted)]'}">
          <span class="relative flex h-1.5 w-1.5">
            {#if extensionConnected}
              <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
            {/if}
            <span class="relative inline-flex rounded-full h-1.5 w-1.5 {extensionConnected ? 'bg-emerald-400' : 'bg-[var(--color-muted)]'}"></span>
          </span>
          <Puzzle class="h-3 w-3" />
          {extensionConnected ? t('ext.connected', currentLocale) : t('ext.disconnected', currentLocale)}
        </span>

        <span class="inline-flex h-1.5 w-1.5 rounded-full bg-[var(--color-success)]"></span>
        <span>{t('header.connected', currentLocale)}</span>
      </div>
    </header>

    <main class="flex-1 px-4 py-8 sm:px-6 lg:px-8">
      {@render children()}
    </main>
  </div>

</div>
{/if}
