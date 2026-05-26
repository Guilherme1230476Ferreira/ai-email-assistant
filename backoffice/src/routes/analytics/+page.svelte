<script lang="ts">
	import { t, locale, type Locale } from '$lib/i18n';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	let currentLocale = $state<Locale>('en');
	locale.subscribe((v) => (currentLocale = v));

	// ── Types ──────────────────────────────────────────────────────────────────
	type KBStat = { id: string; title: string; entry_type: string; retrieval_count: number };
	type Bucket = { label: string; count: number };
	type HistPoint = { day: string; avg_similarity: number; kb_hit_rate: number };

	// ── Data (reactive via $derived so Svelte 5 tracks prop changes) ───────────
	let stats = $derived<KBStat[]>(data.knowledgeStats ?? []);
	let distribution = $derived<Bucket[]>(data.distribution ?? []);
	let history = $derived<HistPoint[]>(data.history ?? []);

	// ── Chart 1: KB Retrieval Frequency ───────────────────────────────────────
	const barH = 32;
	const barGap = 10;
	const labelW = 200;
	const barAreaW = 380;
	const countW = 60;
	const svgW = labelW + barAreaW + countW;
	const chartPadTop = 10;

	let maxCount = $derived(Math.max(...stats.map((s) => s.retrieval_count), 1));
	let chart1H = $derived(Math.max(stats.length * (barH + barGap) + chartPadTop + 20, 80));

	function barWidth(count: number, max: number) {
		return (count / max) * barAreaW;
	}
	function truncate(s: string, n = 26) {
		return s.length > n ? s.slice(0, n) + '…' : s;
	}

	// ── Chart 2: Similarity Histogram ─────────────────────────────────────────
	const histW = 560;
	const histH = 200;
	const histPad = { top: 20, right: 20, bottom: 36, left: 40 };
	const bW = Math.floor((histW - histPad.left - histPad.right) / 5) - 8;
	const histInnerH = histH - histPad.top - histPad.bottom;
	const bucketColors = ['#ef4444', '#f97316', '#eab308', '#22c55e', '#10b981'];

	let maxBucket = $derived(Math.max(...distribution.map((b) => b.count), 1));

	function bucketBarH(count: number, max: number) {
		return (count / max) * histInnerH;
	}
	function bucketX(i: number) {
		const spacing = (histW - histPad.left - histPad.right) / 5;
		return histPad.left + i * spacing + 4;
	}
	function bucketY(count: number, max: number) {
		return histPad.top + histInnerH - bucketBarH(count, max);
	}

	// ── Chart 3: RAG Metrics Over Time ────────────────────────────────────────
	const lineW = 560;
	const lineH = 240;
	const linePad = { top: 24, right: 24, bottom: 44, left: 48 };
	const lineInnerW = lineW - linePad.left - linePad.right;
	const lineInnerH = lineH - linePad.top - linePad.bottom;

	function px(i: number, total: number) {
		if (total < 2) return linePad.left + lineInnerW / 2;
		return linePad.left + (i / (total - 1)) * lineInnerW;
	}
	function py(v: number) {
		return linePad.top + (1 - Math.max(0, Math.min(1, v))) * lineInnerH;
	}
	function polyline(vals: number[], total: number) {
		return vals.map((v, i) => `${px(i, total)},${py(v)}`).join(' ');
	}
	function area(vals: number[], total: number) {
		if (vals.length === 0) return '';
		const pts = vals.map((v, i) => `${px(i, total)},${py(v)}`).join(' ');
		const base = `${px(vals.length - 1, total)},${py(0)} ${px(0, total)},${py(0)}`;
		return `M ${pts.split(' ')[0]} L ${pts} L ${base} Z`;
	}

	// X-axis tick indices (~6 evenly spaced)
	let xTicks = $derived(
		history.length > 1
			? [0, ...Array.from({ length: 4 }, (_, i) => Math.round(((i + 1) / 5) * (history.length - 1))), history.length - 1]
					.filter((v, i, a) => a.indexOf(v) === i)
			: history.length === 1 ? [0] : []
	);
</script>

<svelte:head>
	<title>RAG Analytics — MailMate</title>
	<meta name="description" content="View RAG knowledge base performance metrics and similarity analytics." />
</svelte:head>

<div class="space-y-8">
	<!-- Page header -->
	<div>
		<h1 class="text-2xl font-bold tracking-tight text-white">{t('analytics.title', currentLocale)}</h1>
		<p class="mt-1 text-sm text-[var(--color-muted)]">{t('analytics.subtitle', currentLocale)}</p>
	</div>

	<!-- ── Chart 1: KB Entry Retrieval Frequency ────────────────────────────── -->
	<div class="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-6">
		<div class="mb-5">
			<h2 class="text-base font-semibold text-white">{t('analytics.kb_frequency', currentLocale)}</h2>
			<p class="mt-0.5 text-xs text-[var(--color-muted)]">{t('analytics.kb_frequency_desc', currentLocale)}</p>
		</div>

		{#if stats.length === 0}
			<div class="flex h-24 items-center justify-center rounded-lg border border-dashed border-[var(--color-border)] text-sm text-[var(--color-muted)]">
				{t('analytics.no_retrievals', currentLocale)}
			</div>
		{:else}
			<div class="overflow-x-auto">
				<svg
					viewBox="0 0 {svgW} {chart1H}"
					width="100%"
					height={chart1H}
					role="img"
					aria-label="KB entry retrieval frequency bar chart"
					class="chart-svg"
				>
					<!-- Grid lines -->
					{#each [0.25, 0.5, 0.75, 1] as frac}
						<line
							x1={labelW + barAreaW * frac}
							y1={chartPadTop}
							x2={labelW + barAreaW * frac}
							y2={chart1H - 10}
							stroke="var(--color-border)"
							stroke-width="1"
							stroke-dasharray="3,3"
						/>
					{/each}

					{#each stats as stat, i}
						{@const y = chartPadTop + i * (barH + barGap)}
						{@const bw = barWidth(stat.retrieval_count, maxCount)}
						{@const isDoc = stat.entry_type === 'document'}

						<!-- Label -->
						<text
							x={labelW - 10}
							y={y + barH / 2 + 4}
							text-anchor="end"
							fill="var(--color-muted-foreground)"
							font-size="11"
							font-family="Inter, sans-serif"
						>
							{truncate(stat.title)}
						</text>

						<!-- Bar track -->
						<rect
							x={labelW}
							y={y}
							width={barAreaW}
							height={barH}
							rx="4"
							fill="var(--color-surface-2)"
						/>

						<!-- Bar fill with animation -->
						<rect
							x={labelW}
							y={y}
							width={bw}
							height={barH}
							rx="4"
							fill={isDoc ? 'var(--color-accent)' : '#818cf8'}
							opacity="0.85"
							class="bar-fill"
							style="transform-origin: {labelW}px {y + barH / 2}px"
						/>

						<!-- Type badge -->
						<text
							x={labelW + 8}
							y={y + barH / 2 + 4}
							fill={isDoc ? '#000' : '#fff'}
							font-size="9"
							font-family="Inter, sans-serif"
							font-weight="600"
							opacity={bw > 40 ? 1 : 0}
						>
							{isDoc ? t('analytics.document', currentLocale) : t('analytics.qa_pair', currentLocale)}
						</text>

						<!-- Count label -->
						<text
							x={labelW + bw + 8}
							y={y + barH / 2 + 4}
							fill="var(--color-muted-foreground)"
							font-size="11"
							font-family="Inter, sans-serif"
						>
							{stat.retrieval_count}
						</text>
					{/each}
				</svg>
			</div>

			<!-- Legend -->
			<div class="mt-3 flex items-center gap-4 text-xs text-[var(--color-muted)]">
				<span class="flex items-center gap-1.5">
					<span class="inline-block h-2.5 w-2.5 rounded-sm bg-[var(--color-accent)]"></span>
					{t('analytics.document', currentLocale)}
				</span>
				<span class="flex items-center gap-1.5">
					<span class="inline-block h-2.5 w-2.5 rounded-sm bg-indigo-400"></span>
					{t('analytics.qa_pair', currentLocale)}
				</span>
			</div>
		{/if}
	</div>

	<!-- ── Chart 2: Similarity Score Distribution ────────────────────────────── -->
	<div class="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-6">
		<div class="mb-5">
			<h2 class="text-base font-semibold text-white">{t('analytics.similarity_dist', currentLocale)}</h2>
			<p class="mt-0.5 text-xs text-[var(--color-muted)]">{t('analytics.similarity_dist_desc', currentLocale)}</p>
		</div>

		{#if distribution.length === 0 || distribution.every((b) => b.count === 0)}
			<div class="flex h-24 items-center justify-center rounded-lg border border-dashed border-[var(--color-border)] text-sm text-[var(--color-muted)]">
				{t('analytics.no_data', currentLocale)}
			</div>
		{:else}
			<div class="overflow-x-auto">
				<svg
					viewBox="0 0 {histW} {histH}"
					width="100%"
					height={histH}
					role="img"
					aria-label="Similarity score distribution histogram"
					class="chart-svg"
				>
					<!-- Y-axis grid lines -->
					{#each [0, 0.25, 0.5, 0.75, 1] as frac}
						{@const y = histPad.top + histInnerH * (1 - frac)}
						<line
							x1={histPad.left}
							y1={y}
							x2={histW - histPad.right}
							y2={y}
							stroke="var(--color-border)"
							stroke-width="1"
						/>
						<text
							x={histPad.left - 6}
							y={y + 4}
							text-anchor="end"
							fill="var(--color-muted)"
							font-size="9"
							font-family="Inter, sans-serif"
						>
							{Math.round(frac * maxBucket)}
						</text>
					{/each}

					<!-- Bars -->
					{#each distribution as bucket, i}
						{@const bh = bucketBarH(bucket.count, maxBucket)}
						{@const bx = bucketX(i)}
						{@const by = bucketY(bucket.count, maxBucket)}

						<!-- Bar -->
						<rect
							x={bx}
							y={by}
							width={bW}
							height={bh}
							rx="4"
							fill={bucketColors[i]}
							opacity="0.8"
							class="bar-fill"
						/>

						<!-- Count on top -->
						{#if bucket.count > 0}
							<text
								x={bx + bW / 2}
								y={by - 5}
								text-anchor="middle"
								fill="var(--color-muted-foreground)"
								font-size="10"
								font-family="Inter, sans-serif"
								font-weight="600"
							>
								{bucket.count}
							</text>
						{/if}

						<!-- Label below -->
						<text
							x={bx + bW / 2}
							y={histH - histPad.bottom + 16}
							text-anchor="middle"
							fill="var(--color-muted)"
							font-size="9"
							font-family="Inter, sans-serif"
						>
							{bucket.label}
						</text>
					{/each}

					<!-- X axis label -->
					<text
						x={histW / 2}
						y={histH - 2}
						text-anchor="middle"
						fill="var(--color-muted)"
						font-size="9"
						font-family="Inter, sans-serif"
					>
						Similarity Score
					</text>
				</svg>
			</div>
		{/if}
	</div>

	<!-- ── Chart 3: RAG Metrics Over Time ────────────────────────────────────── -->
	<div class="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-6">
		<div class="mb-5 flex items-start justify-between">
			<div>
				<h2 class="text-base font-semibold text-white">{t('analytics.metrics_over_time', currentLocale)}</h2>
				<p class="mt-0.5 text-xs text-[var(--color-muted)]">{t('analytics.metrics_over_time_desc', currentLocale)}</p>
			</div>
			<!-- Legend -->
			<div class="flex items-center gap-4 text-xs text-[var(--color-muted)]">
				<span class="flex items-center gap-1.5">
					<span class="inline-block h-0.5 w-5 rounded bg-cyan-400"></span>
					{t('analytics.avg_similarity', currentLocale)}
				</span>
				<span class="flex items-center gap-1.5">
					<span class="inline-block h-0.5 w-5 rounded bg-emerald-400"></span>
					{t('analytics.kb_hit_rate', currentLocale)}
				</span>
			</div>
		</div>

		{#if history.length === 0}
			<div class="flex h-24 items-center justify-center rounded-lg border border-dashed border-[var(--color-border)] text-sm text-[var(--color-muted)]">
				{t('analytics.no_data', currentLocale)}
			</div>
		{:else}
			<div class="overflow-x-auto">
				<svg
					viewBox="0 0 {lineW} {lineH}"
					width="100%"
					height={lineH}
					role="img"
					aria-label="RAG metrics over time line chart"
					class="chart-svg"
				>
					<!-- Y-axis grid + labels -->
					{#each [0, 0.25, 0.5, 0.75, 1] as v}
						{@const y = py(v)}
						<line
							x1={linePad.left}
							y1={y}
							x2={lineW - linePad.right}
							y2={y}
							stroke="var(--color-border)"
							stroke-width="1"
							stroke-dasharray={v === 0 || v === 1 ? '' : '3,3'}
						/>
						<text
							x={linePad.left - 6}
							y={y + 4}
							text-anchor="end"
							fill="var(--color-muted)"
							font-size="9"
							font-family="Inter, sans-serif"
						>
							{Math.round(v * 100)}%
						</text>
					{/each}

					<!-- Area fills (subtle) -->
					{#if history.length > 1}
						<path
							d={area(history.map((h) => h.avg_similarity), history.length)}
							fill="rgba(34,211,238,0.06)"
						/>
						<path
							d={area(history.map((h) => h.kb_hit_rate), history.length)}
							fill="rgba(16,185,129,0.06)"
						/>

						<!-- Lines -->
						<polyline
							points={polyline(history.map((h) => h.avg_similarity), history.length)}
							fill="none"
							stroke="#22d3ee"
							stroke-width="2"
							stroke-linejoin="round"
							stroke-linecap="round"
						/>
						<polyline
							points={polyline(history.map((h) => h.kb_hit_rate), history.length)}
							fill="none"
							stroke="#10b981"
							stroke-width="2"
							stroke-linejoin="round"
							stroke-linecap="round"
						/>

						<!-- Dots -->
						{#each history as pt, i}
							<circle cx={px(i, history.length)} cy={py(pt.avg_similarity)} r="3" fill="#22d3ee" />
							<circle cx={px(i, history.length)} cy={py(pt.kb_hit_rate)} r="3" fill="#10b981" />
						{/each}
					{:else if history.length === 1}
						<circle cx={px(0, 1)} cy={py(history[0].avg_similarity)} r="4" fill="#22d3ee" />
						<circle cx={px(0, 1)} cy={py(history[0].kb_hit_rate)} r="4" fill="#10b981" />
					{/if}

					<!-- X-axis tick labels -->
					{#each xTicks as idx}
						{@const label = history[idx]?.day?.slice(5) ?? ''}
						<text
							x={px(idx, history.length)}
							y={lineH - linePad.bottom + 14}
							text-anchor="middle"
							fill="var(--color-muted)"
							font-size="9"
							font-family="Inter, sans-serif"
						>
							{label}
						</text>
						<line
							x1={px(idx, history.length)}
							y1={lineH - linePad.bottom}
							x2={px(idx, history.length)}
							y2={lineH - linePad.bottom + 4}
							stroke="var(--color-border)"
							stroke-width="1"
						/>
					{/each}
				</svg>
			</div>
		{/if}
	</div>
</div>

<style>
	.chart-svg {
		display: block;
	}

	@keyframes bar-in {
		from { transform: scaleX(0); }
		to   { transform: scaleX(1); }
	}

	:global(.bar-fill) {
		animation: bar-in 0.5s cubic-bezier(0.16, 1, 0.3, 1) both;
	}
</style>
