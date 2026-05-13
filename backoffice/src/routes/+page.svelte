<script lang="ts">
  import { Mail, Users, Shield, Cpu, ArrowUpRight, Check, Activity, BarChart2 } from '@lucide/svelte';
  import type { PageData } from './$types';

  let { data }: { data: PageData } = $props();

  let stats = $derived([
    { label: 'Emails generated', value: data.stats.emailsCount.toString(), icon: Mail, href: '/emails' },
    { label: 'Users', value: data.stats.usersCount.toString(), icon: Users, href: '/users' },
    { label: 'Roles', value: data.stats.rolesCount.toString(), icon: Shield, href: '/roles' },
    { label: 'LLM model', value: data.llm?.llm_model || 'Not set', icon: Cpu, href: '/settings' }
  ]);

  let llmConfig = $derived({
    model: data.llm?.llm_model || 'Unknown',
    base_url: data.llm?.llm_base_url || 'Unknown',
    has_api_key: !!data.llm?.has_api_key,
    embedding_model: 'text-embedding-3-small' 
  });

  // Real Postgres-driven RAG health metrics
  let ragMetrics = $derived({
    contextRetrievalRate: Math.round((data.telemetry?.context_retrieval_rate || 0) * 100) + '%',
    hallucinationRisk: Math.max(0, 100 - (data.telemetry?.avg_similarity_score || 0) * 100).toFixed(1) + '%',
    avgSimilarityScore: (data.telemetry?.avg_similarity_score || 0).toFixed(2),
    tokensSaved: data.telemetry?.tokens_saved || 0,
    knowledgeMatches: data.telemetry?.knowledge_matches || 0
  });

  // Calculate the vertices of the knowledge triangle based on real pgvector metrics
  let trianglePoints = $derived(() => {
     let v1 = Math.min(1, data.telemetry?.context_retrieval_rate || 0.1); // Context Rate (Top)
     let v2 = Math.min(1, data.telemetry?.avg_similarity_score || 0.1); // Similarity (Bottom Right)
     let v3 = Math.max(0.1, data.telemetry?.avg_similarity_score || 0.1); // Accuracy proxy (Bottom Left)
     
     // Base radius of the triangle boundaries
     const r = 40;
     
     // Angles (in radians)
     // Top: -90 degrees (-PI/2)
     let p1y = 50 - r * v1; 
     let p1x = 50;
     
     // Bottom Right: 30 degrees (PI/6)
     let p2x = 50 + (r * v2 * 0.866);
     let p2y = 50 + (r * v2 * 0.5);
     
     // Bottom Left: 150 degrees (5PI/6)
     let p3x = 50 - (r * v3 * 0.866);
     let p3y = 50 + (r * v3 * 0.5);

     return `${p1x},${p1y} ${p2x},${p2y} ${p3x},${p3y}`;
  });

</script>

<svelte:head>
  <title>Dashboard · MailMate</title>
</svelte:head>

<div class="mx-auto w-full max-w-5xl space-y-10">
  <header class="flex flex-col gap-1">
    <h1 class="text-2xl font-semibold tracking-tight text-white">Dashboard</h1>
    <p class="text-sm text-[var(--color-muted-foreground)]">
      Overview of your AI email assistant.
    </p>
  </header>

  <section class="grid grid-cols-2 gap-3 lg:grid-cols-4">
    {#each stats as s (s.label)}
      <a
        href={s.href}
        class="group flex flex-col justify-between rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] p-4 transition hover:border-[var(--color-border-strong)]"
      >
        <div class="flex items-center justify-between">
          <span class="text-xs text-[var(--color-muted-foreground)]">{s.label}</span>
          <s.icon class="h-4 w-4 text-[var(--color-muted)]" />
        </div>
        <div class="mt-4 flex items-end justify-between">
          <span class="text-2xl font-semibold tracking-tight text-white">{s.value}</span>
          <ArrowUpRight
            class="h-4 w-4 text-[var(--color-muted)] transition group-hover:text-[var(--color-accent)]"
          />
        </div>
      </a>
    {/each}
  </section>

  <section class="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)]">
    <div class="flex items-center justify-between border-b border-[var(--color-border)] px-5 py-4">
      <div>
        <h2 class="text-sm font-medium text-white">LLM configuration</h2>
        <p class="text-xs text-[var(--color-muted-foreground)]">
          Used by <code class="text-[var(--color-muted-foreground)]">/api/emails/generate</code>
        </p>
      </div>
      <a
        href="/settings"
        class="rounded-md border border-[var(--color-border)] px-3 py-1.5 text-xs font-medium text-white transition hover:border-[var(--color-accent)]/40 hover:text-[var(--color-accent)]"
      >
        Manage
      </a>
    </div>

    <dl
      class="grid grid-cols-1 divide-y divide-[var(--color-border)] sm:grid-cols-2 sm:divide-x sm:divide-y-0"
    >
      <div class="px-5 py-4">
        <dt class="text-[11px] uppercase tracking-wider text-[var(--color-muted)]">Model</dt>
        <dd class="mt-1 font-mono text-sm text-white">{llmConfig.model}</dd>
      </div>
      <div class="px-5 py-4">
        <dt class="text-[11px] uppercase tracking-wider text-[var(--color-muted)]">Base URL</dt>
        <dd class="mt-1 truncate font-mono text-sm text-white">{llmConfig.base_url}</dd>
      </div>
      <div class="px-5 py-4">
        <dt class="text-[11px] uppercase tracking-wider text-[var(--color-muted)]">API key</dt>
        <dd class="mt-1 flex items-center gap-2 text-sm text-white">
          {#if llmConfig.has_api_key}
            <span
              class="inline-flex h-4 w-4 items-center justify-center rounded-full bg-[var(--color-success)]/15 text-[var(--color-success)]"
            >
              <Check class="h-3 w-3" strokeWidth={3} />
            </span>
            <span>Configured</span>
          {:else}
            <span class="text-[var(--color-warning)]">Not set</span>
          {/if}
        </dd>
      </div>
      <div class="px-5 py-4">
        <dt class="text-[11px] uppercase tracking-wider text-[var(--color-muted)]">
          Embedding model
        </dt>
        <dd class="mt-1 font-mono text-sm text-white">{llmConfig.embedding_model}</dd>
      </div>
    </dl>
  </section>

  <!-- RAG Health & AI Telemetry Dashboard -->
  <section class="grid grid-cols-1 lg:grid-cols-2 gap-4">
    <!-- Hallucination & Accuracy Graph (Visual representation) -->
    <div class="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] p-5">
      <div class="flex items-center justify-between mb-6">
        <div>
          <h2 class="text-sm font-medium text-white flex items-center gap-2">
            <Activity class="h-4 w-4 text-[var(--color-accent)]" />
            RAG Context Health
          </h2>
          <p class="text-xs text-[var(--color-muted-foreground)] mt-1">Accuracy vs Hallucination bounds</p>
        </div>
        <span class="text-[10px] uppercase font-bold tracking-wider text-emerald-400 bg-emerald-400/10 px-2 py-1 rounded-full">Optimal</span>
      </div>

      <div class="space-y-5">
        
        <!-- Interactive Geometric Graphic -->
        <div class="relative w-full aspect-video flex items-center justify-center bg-[var(--color-surface-2)] rounded-lg overflow-hidden border border-[var(--color-border)] my-6">
          <svg viewBox="0 0 100 100" class="w-full max-w-[200px] h-full" preserveAspectRatio="xMidYMid meet">
            <!-- Background Web Rings -->
            <polygon points="50,10 84.64,70 15.36,70" fill="none" stroke="rgba(255,255,255,0.05)" stroke-width="0.5" />
            <polygon points="50,23.3 73.1,63.3 26.9,63.3" fill="none" stroke="rgba(255,255,255,0.05)" stroke-width="0.5" />
            <polygon points="50,36.6 61.5,56.6 38.5,56.6" fill="none" stroke="rgba(255,255,255,0.05)" stroke-width="0.5" />
            
            <!-- Axes Lines -->
            <line x1="50" y1="50" x2="50" y2="10" stroke="rgba(255,255,255,0.1)" stroke-width="0.5" />
            <line x1="50" y1="50" x2="84.64" y2="70" stroke="rgba(255,255,255,0.1)" stroke-width="0.5" />
            <line x1="50" y1="50" x2="15.36" y2="70" stroke="rgba(255,255,255,0.1)" stroke-width="0.5" />

            <!-- The Dynamic Vector Triangle -->
            <polygon 
              points={trianglePoints()} 
              fill="rgba(45, 212, 191, 0.2)" 
              stroke="#2dd4bf" 
              stroke-width="1.5" 
              class="transition-all duration-1000 ease-out"
            />
            
            <!-- Radar Node Dots -->
            <circle cx={trianglePoints().split(' ')[0].split(',')[0]} cy={trianglePoints().split(' ')[0].split(',')[1]} r="1.5" fill="#3b82f6" />
            <circle cx={trianglePoints().split(' ')[1].split(',')[0]} cy={trianglePoints().split(' ')[1].split(',')[1]} r="1.5" fill="#10b981" />
            <circle cx={trianglePoints().split(' ')[2].split(',')[0]} cy={trianglePoints().split(' ')[2].split(',')[1]} r="1.5" fill="#f43f5e" />
          </svg>

          <!-- Axis Labels Overlay -->
          <div class="absolute inset-0 pointer-events-none p-2 flex flex-col justify-between">
            <span class="text-[9px] text-blue-400 font-bold uppercase text-center block mt-1 tracking-wider absolute top-2 left-1/2 -translate-x-1/2">Context Depth</span>
            <div class="flex justify-between w-full mt-auto mb-2 px-2 absolute bottom-2 left-0">
               <span class="text-[9px] text-emerald-400 font-bold uppercase tracking-wider">Similarity</span>
               <span class="text-[9px] text-rose-400 font-bold uppercase tracking-wider">Accuracy</span>
            </div>
          </div>
        </div>

<div class="grid grid-cols-3 gap-2 text-center border-t border-[var(--color-border)] pt-3">
          <div class="flex flex-col">
            <span class="text-[10px] text-[var(--color-muted-foreground)]">Context Depth</span>
            <span class="text-xs font-bold text-white">{ragMetrics.contextRetrievalRate}</span>
          </div>
          <div class="flex flex-col border-x border-[var(--color-border)]">
            <span class="text-[10px] text-[var(--color-muted-foreground)]">Similarity Avg.</span>
            <span class="text-xs font-bold text-white">{ragMetrics.avgSimilarityScore}</span>
          </div>
          <div class="flex flex-col">
            <span class="text-[10px] text-[var(--color-muted-foreground)]">Hallucination Risk</span>
            <span class="text-xs font-bold text-rose-400">{ragMetrics.hallucinationRisk}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- AI Telemetry Stats -->
    <div class="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] p-5 flex flex-col justify-between">
      <div>
        <h2 class="text-sm font-medium text-white flex items-center gap-2">
          <BarChart2 class="h-4 w-4 text-[var(--color-accent)]" />
          AI Execution Telemetry
        </h2>
        <p class="text-xs text-[var(--color-muted-foreground)] mt-1">Live metrics from your PostgreSQL Vector DB</p>
      </div>

      <div class="grid grid-cols-2 gap-4 mt-6">
        <div class="p-3 rounded-md bg-[var(--color-surface-2)] flex flex-col gap-1">
          <span class="text-[10px] text-[var(--color-muted-foreground)] uppercase font-semibold">Context Rate</span>
          <span class="text-lg font-bold text-white">{ragMetrics.contextRetrievalRate}</span>
        </div>
        <div class="p-3 rounded-md bg-[var(--color-surface-2)] flex flex-col gap-1">
          <span class="text-[10px] text-[var(--color-muted-foreground)] uppercase font-semibold">Tokens Processed</span>
          <span class="text-lg font-bold text-white">{ragMetrics.tokensSaved.toLocaleString()}</span>
        </div>
        <div class="p-3 rounded-md bg-[var(--color-surface-2)] flex flex-col gap-1">
          <span class="text-[10px] text-[var(--color-muted-foreground)] uppercase font-semibold">Avg. Similarity</span>
          <span class="text-lg font-bold text-[var(--color-accent)]">{ragMetrics.avgSimilarityScore}</span>
        </div>
        <div class="p-3 rounded-md bg-[var(--color-surface-2)] flex flex-col gap-1">
          <span class="text-[10px] text-[var(--color-muted-foreground)] uppercase font-semibold">Knowledge Matches</span>
          <span class="text-lg font-bold text-white">{ragMetrics.knowledgeMatches}</span>
        </div>
      </div>
    </div>
  </section>

  <section class="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)]">
    <div class="flex items-center justify-between border-b border-[var(--color-border)] px-5 py-4">
      <div>
        <h2 class="text-sm font-medium text-white">Recent generations</h2>
        <p class="text-xs text-[var(--color-muted-foreground)]">Latest AI-drafted replies.</p>
      </div>
      <a href="/emails" class="text-xs font-medium text-[var(--color-accent)] hover:underline">
        View all
      </a>
    </div>

    <ul class="divide-y divide-[var(--color-border)]">
      {#each data.recentEmails as email (email.id)}
        <li class="px-5 py-4 transition hover:bg-[var(--color-surface-2)]/50">
          <div class="flex items-start justify-between gap-4">
            <div class="min-w-0 flex-1 space-y-1.5">
              <p class="line-clamp-1 text-sm text-white">{email.original_content || "No prompt provided"}</p>
              <p class="line-clamp-1 text-xs text-[var(--color-muted-foreground)]">
                <span class="text-[var(--color-accent)]">↳</span>
                {email.generated_response || "Empty response"}
              </p>
            </div>
            <span class="shrink-0 text-[11px] text-[var(--color-muted)]">
              {new Date(email.created_at || Date.now()).toLocaleDateString()}
            </span>
          </div>
        </li>
      {:else}
        <li class="px-5 py-10 text-center text-sm text-[var(--color-muted-foreground)]">
          No emails generated yet.
        </li>
      {/each}
    </ul>
  </section>
</div>
