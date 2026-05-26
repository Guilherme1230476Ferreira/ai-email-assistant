<script lang="ts">
  import { Sparkles, Shield, Database, Globe, Trash2, Download, Mail } from '@lucide/svelte';
</script>

<svelte:head>
  <title>Privacy Policy — AI Email Assistant</title>
  <meta name="description" content="Privacy Policy for AI Email Assistant — how we collect, use and protect your personal data under GDPR." />
</svelte:head>

<div class="privacy-root">
  <!-- Background glow -->
  <div class="privacy-bg-glow"></div>

  <div class="privacy-container">
    <!-- Header -->
    <header class="privacy-header">
      <a href="/login" class="privacy-logo-link">
        <div class="privacy-logo">
          <Sparkles class="privacy-logo-icon" strokeWidth={2.5} />
        </div>
        <span class="privacy-logo-name">AI Email Assistant</span>
      </a>
      <div class="privacy-badge">
        <Shield class="privacy-badge-icon" />
        GDPR Compliant
      </div>
    </header>

    <!-- Hero -->
    <section class="privacy-hero">
      <h1 class="privacy-title">Privacy Policy</h1>
      <p class="privacy-subtitle">
        Last updated: <strong>May 26, 2026</strong>
      </p>
      <p class="privacy-intro">
        This Privacy Policy explains how <strong>AI Email Assistant</strong> ("we", "our", "the service")
        collects, uses, stores and protects your personal data in accordance with the
        <strong>EU General Data Protection Regulation (GDPR)</strong> — Regulation (EU) 2016/679 —
        and the Portuguese data protection law <strong>Lei n.º 58/2019</strong>.
      </p>
    </section>

    <!-- Cards -->
    <div class="privacy-sections">

      <!-- 1. Controller -->
      <section class="privacy-card">
        <div class="privacy-card-header">
          <div class="privacy-card-icon-wrap privacy-card-icon--blue">
            <Globe class="privacy-card-icon" />
          </div>
          <h2 class="privacy-card-title">1. Data Controller</h2>
        </div>
        <div class="privacy-card-body">
          <p>The data controller is the operator of this service, deployed at ISEP — Instituto Superior de Engenharia do Porto, Portugal.</p>
          <p>For any data protection enquiries contact: <a href="mailto:admin@isep.ipp.pt" class="privacy-link">admin@isep.ipp.pt</a></p>
        </div>
      </section>

      <!-- 2. Data Collected -->
      <section class="privacy-card">
        <div class="privacy-card-header">
          <div class="privacy-card-icon-wrap privacy-card-icon--cyan">
            <Database class="privacy-card-icon" />
          </div>
          <h2 class="privacy-card-title">2. Personal Data We Collect</h2>
        </div>
        <div class="privacy-card-body">
          <table class="privacy-table">
            <thead>
              <tr>
                <th>Category</th>
                <th>Examples</th>
                <th>Purpose</th>
                <th>Legal Basis (GDPR Art. 6)</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td>Account data</td>
                <td>E-mail address, hashed password</td>
                <td>Authentication &amp; account management</td>
                <td>Art. 6(1)(b) — contract performance</td>
              </tr>
              <tr>
                <td>Email content</td>
                <td>Original messages you paste, AI-generated replies</td>
                <td>Providing the AI reply service</td>
                <td>Art. 6(1)(b) — contract performance</td>
              </tr>
              <tr>
                <td>Email embeddings</td>
                <td>Vector representations of email text</td>
                <td>Improving reply relevance (RAG retrieval)</td>
                <td>Art. 6(1)(b) — contract performance</td>
              </tr>
              <tr>
                <td>Audit logs</td>
                <td>Actions performed in the system (anonymised on account deletion)</td>
                <td>Security &amp; accountability</td>
                <td>Art. 6(1)(f) — legitimate interests</td>
              </tr>
            </tbody>
          </table>
        </div>
      </section>

      <!-- 3. Third-party processors -->
      <section class="privacy-card">
        <div class="privacy-card-header">
          <div class="privacy-card-icon-wrap privacy-card-icon--amber">
            <Globe class="privacy-card-icon" />
          </div>
          <h2 class="privacy-card-title">3. Third-Party Data Processors</h2>
        </div>
        <div class="privacy-card-body">
          <p class="privacy-warning-note">
            ⚠️ Your email content is sent to external AI providers to generate replies and create embeddings.
            By using this service you acknowledge this transfer.
          </p>
          <table class="privacy-table">
            <thead>
              <tr>
                <th>Processor</th>
                <th>Purpose</th>
                <th>Data transferred</th>
                <th>Privacy Policy</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td><strong>Groq, Inc.</strong></td>
                <td>AI reply generation (LLM inference)</td>
                <td>Email content &amp; context</td>
                <td><a href="https://groq.com/privacy-policy/" class="privacy-link" target="_blank" rel="noopener">groq.com/privacy-policy</a></td>
              </tr>
              <tr>
                <td><strong>Google LLC</strong></td>
                <td>Text embeddings via Gemini API</td>
                <td>Email &amp; document text</td>
                <td><a href="https://policies.google.com/privacy" class="privacy-link" target="_blank" rel="noopener">policies.google.com/privacy</a></td>
              </tr>
              <tr>
                <td><strong>Google LLC</strong></td>
                <td>Google Sign-In (OAuth)</td>
                <td>Email address &amp; profile</td>
                <td><a href="https://policies.google.com/privacy" class="privacy-link" target="_blank" rel="noopener">policies.google.com/privacy</a></td>
              </tr>
            </tbody>
          </table>
          <p>All providers are based in the USA; transfers are covered under Standard Contractual Clauses (SCCs) per GDPR Chapter V.</p>
        </div>
      </section>

      <!-- 4. Retention -->
      <section class="privacy-card">
        <div class="privacy-card-header">
          <div class="privacy-card-icon-wrap privacy-card-icon--purple">
            <Database class="privacy-card-icon" />
          </div>
          <h2 class="privacy-card-title">4. Data Retention</h2>
        </div>
        <div class="privacy-card-body">
          <ul class="privacy-list">
            <li><strong>Account data &amp; emails</strong> — retained for the lifetime of your account.</li>
            <li><strong>Email embeddings</strong> — retained for the lifetime of your account.</li>
            <li><strong>Audit logs</strong> — retained for 12 months; your user reference is anonymised upon account deletion.</li>
            <li>You may delete your account and all associated data at any time (see Section 5).</li>
          </ul>
        </div>
      </section>

      <!-- 5. Your Rights -->
      <section class="privacy-card">
        <div class="privacy-card-header">
          <div class="privacy-card-icon-wrap privacy-card-icon--green">
            <Shield class="privacy-card-icon" />
          </div>
          <h2 class="privacy-card-title">5. Your Rights Under GDPR</h2>
        </div>
        <div class="privacy-card-body">
          <div class="privacy-rights-grid">
            <div class="privacy-right-item">
              <div class="privacy-right-label">Right of access (Art. 15)</div>
              <p>Export all your data as JSON from your account page.</p>
              <div class="privacy-right-endpoint"><Download class="privacy-right-endpoint-icon" /> GET /api/auth/me/data</div>
            </div>
            <div class="privacy-right-item">
              <div class="privacy-right-label">Right to erasure (Art. 17)</div>
              <p>Permanently delete your account and all associated personal data.</p>
              <div class="privacy-right-endpoint"><Trash2 class="privacy-right-endpoint-icon" /> DELETE /api/auth/me</div>
            </div>
            <div class="privacy-right-item">
              <div class="privacy-right-label">Right to rectification (Art. 16)</div>
              <p>Contact the administrator to correct inaccurate data.</p>
            </div>
            <div class="privacy-right-item">
              <div class="privacy-right-label">Right to object (Art. 21)</div>
              <p>You may object to processing based on legitimate interests by contacting us.</p>
            </div>
            <div class="privacy-right-item">
              <div class="privacy-right-label">Right to lodge a complaint</div>
              <p>You may lodge a complaint with the Portuguese data protection authority: <a href="https://www.cnpd.pt" class="privacy-link" target="_blank" rel="noopener">CNPD — cnpd.pt</a></p>
            </div>
          </div>
        </div>
      </section>

      <!-- 6. AI Act disclosure -->
      <section class="privacy-card">
        <div class="privacy-card-header">
          <div class="privacy-card-icon-wrap privacy-card-icon--cyan">
            <Sparkles class="privacy-card-icon" />
          </div>
          <h2 class="privacy-card-title">6. AI-Generated Content — EU AI Act Disclosure</h2>
        </div>
        <div class="privacy-card-body">
          <p>
            In accordance with <strong>Article 50 of the EU AI Act</strong> (Regulation (EU) 2024/1689),
            this service discloses that all email replies shown in the application are
            <strong>generated by an artificial intelligence system</strong> (large language model).
          </p>
          <p>
            AI-generated replies are clearly labelled with an <span class="privacy-ai-badge">✦ AI Generated</span> badge
            throughout the interface. You should review every reply before sending it.
          </p>
        </div>
      </section>

      <!-- 7. Security -->
      <section class="privacy-card">
        <div class="privacy-card-header">
          <div class="privacy-card-icon-wrap privacy-card-icon--purple">
            <Shield class="privacy-card-icon" />
          </div>
          <h2 class="privacy-card-title">7. Security Measures</h2>
        </div>
        <div class="privacy-card-body">
          <ul class="privacy-list">
            <li>Passwords are stored as <strong>bcrypt hashes</strong> (cost factor 12) — never in plaintext.</li>
            <li>Authentication uses short-lived <strong>JSON Web Tokens (JWT)</strong>.</li>
            <li>All API traffic is served over <strong>HTTPS/TLS</strong>.</li>
            <li>Role-based access control (RBAC) limits data access to authorised users.</li>
          </ul>
        </div>
      </section>

      <!-- 8. Changes -->
      <section class="privacy-card">
        <div class="privacy-card-header">
          <div class="privacy-card-icon-wrap privacy-card-icon--amber">
            <Mail class="privacy-card-icon" />
          </div>
          <h2 class="privacy-card-title">8. Changes to This Policy</h2>
        </div>
        <div class="privacy-card-body">
          <p>We may update this policy to reflect changes in the service or applicable law. Material changes will be communicated via the application. The "Last updated" date at the top of this page will always reflect the most recent revision.</p>
        </div>
      </section>

    </div>

    <!-- Footer -->
    <footer class="privacy-footer">
      <a href="/login" class="privacy-back-link">← Back to Sign In</a>
      <span class="privacy-footer-copy">© 2026 AI Email Assistant — ISEP</span>
    </footer>
  </div>
</div>

<style>
  .privacy-root {
    min-height: 100vh;
    background: var(--color-bg, #0a0a0f);
    color: var(--color-text, #e2e8f0);
    font-family: 'Inter', 'Outfit', system-ui, sans-serif;
    position: relative;
    overflow-x: hidden;
  }

  .privacy-bg-glow {
    pointer-events: none;
    position: fixed;
    inset: 0;
    background: radial-gradient(ellipse at top left, rgba(34,211,238,0.06), transparent 55%),
                radial-gradient(ellipse at bottom right, rgba(139,92,246,0.05), transparent 55%);
    z-index: 0;
  }

  .privacy-container {
    position: relative;
    z-index: 1;
    max-width: 860px;
    margin: 0 auto;
    padding: 2rem 1.5rem 4rem;
  }

  /* Header */
  .privacy-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 3rem;
    padding-bottom: 1.5rem;
    border-bottom: 1px solid var(--color-border, rgba(255,255,255,0.08));
  }

  .privacy-logo-link {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    text-decoration: none;
  }

  .privacy-logo {
    width: 2.5rem;
    height: 2.5rem;
    border-radius: 0.5rem;
    background: var(--color-accent, #22d3ee);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  :global(.privacy-logo-icon) {
    width: 1.25rem;
    height: 1.25rem;
    color: #000;
  }

  .privacy-logo-name {
    font-size: 1rem;
    font-weight: 700;
    color: white;
  }

  .privacy-badge {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.75rem;
    font-weight: 600;
    color: #4ade80;
    background: rgba(74,222,128,0.1);
    border: 1px solid rgba(74,222,128,0.25);
    border-radius: 999px;
    padding: 0.35rem 0.85rem;
  }

  :global(.privacy-badge-icon) {
    width: 0.85rem;
    height: 0.85rem;
  }

  /* Hero */
  .privacy-hero {
    margin-bottom: 2.5rem;
  }

  .privacy-title {
    font-size: clamp(2rem, 5vw, 3rem);
    font-weight: 800;
    letter-spacing: -0.03em;
    background: linear-gradient(135deg, #fff 30%, var(--color-accent, #22d3ee));
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
    margin: 0 0 0.5rem;
    line-height: 1.1;
  }

  .privacy-subtitle {
    font-size: 0.875rem;
    color: var(--color-muted, #94a3b8);
    margin: 0 0 1.25rem;
  }

  .privacy-subtitle strong { color: white; }

  .privacy-intro {
    font-size: 0.95rem;
    color: var(--color-muted-foreground, #cbd5e1);
    line-height: 1.7;
    max-width: 720px;
  }

  .privacy-intro strong { color: white; }

  /* Cards */
  .privacy-sections {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
  }

  .privacy-card {
    border-radius: 1rem;
    border: 1px solid var(--color-border, rgba(255,255,255,0.08));
    background: var(--color-surface, rgba(255,255,255,0.03));
    overflow: hidden;
    transition: border-color 0.2s;
  }

  .privacy-card:hover {
    border-color: rgba(255,255,255,0.14);
  }

  .privacy-card-header {
    display: flex;
    align-items: center;
    gap: 0.9rem;
    padding: 1.25rem 1.5rem;
    border-bottom: 1px solid var(--color-border, rgba(255,255,255,0.08));
    background: var(--color-surface-2, rgba(255,255,255,0.02));
  }

  .privacy-card-icon-wrap {
    width: 2.25rem;
    height: 2.25rem;
    border-radius: 0.5rem;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .privacy-card-icon--blue  { background: rgba(59,130,246,0.15); }
  .privacy-card-icon--cyan  { background: rgba(34,211,238,0.15); }
  .privacy-card-icon--amber { background: rgba(251,191,36,0.15); }
  .privacy-card-icon--purple{ background: rgba(139,92,246,0.15); }
  .privacy-card-icon--green { background: rgba(74,222,128,0.15); }

  :global(.privacy-card-icon) {
    width: 1.1rem;
    height: 1.1rem;
    color: var(--color-accent, #22d3ee);
  }



  .privacy-card-title {
    font-size: 1rem;
    font-weight: 700;
    color: white;
    margin: 0;
  }

  .privacy-card-body {
    padding: 1.25rem 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
    font-size: 0.9rem;
    color: var(--color-muted-foreground, #cbd5e1);
    line-height: 1.65;
  }

  .privacy-card-body p { margin: 0; }
  .privacy-card-body strong { color: white; }

  /* Table */
  .privacy-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.825rem;
    overflow-x: auto;
    display: block;
  }

  .privacy-table th {
    text-align: left;
    font-size: 0.7rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-muted, #94a3b8);
    padding: 0.6rem 0.85rem;
    border-bottom: 1px solid var(--color-border, rgba(255,255,255,0.08));
    white-space: nowrap;
  }

  .privacy-table td {
    padding: 0.65rem 0.85rem;
    border-bottom: 1px solid rgba(255,255,255,0.04);
    vertical-align: top;
    color: var(--color-muted-foreground, #cbd5e1);
  }

  .privacy-table td strong { color: white; }

  .privacy-table tr:last-child td { border-bottom: none; }

  /* Warning note */
  .privacy-warning-note {
    padding: 0.75rem 1rem;
    border-radius: 0.5rem;
    background: rgba(251,191,36,0.07);
    border: 1px solid rgba(251,191,36,0.2);
    color: #fbbf24;
    font-size: 0.85rem;
  }

  /* List */
  .privacy-list {
    margin: 0;
    padding-left: 1.25rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .privacy-list li strong { color: white; }

  /* Rights grid */
  .privacy-rights-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
    gap: 0.85rem;
  }

  .privacy-right-item {
    background: rgba(255,255,255,0.02);
    border: 1px solid rgba(255,255,255,0.06);
    border-radius: 0.6rem;
    padding: 0.9rem 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .privacy-right-label {
    font-size: 0.8rem;
    font-weight: 700;
    color: var(--color-accent, #22d3ee);
    margin-bottom: 0.15rem;
  }

  .privacy-right-endpoint {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    font-size: 0.72rem;
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
    color: var(--color-muted, #94a3b8);
    background: rgba(255,255,255,0.04);
    border: 1px solid rgba(255,255,255,0.07);
    border-radius: 0.35rem;
    padding: 0.25rem 0.5rem;
    margin-top: 0.25rem;
    width: fit-content;
  }

  :global(.privacy-right-endpoint-icon) {
    width: 0.75rem;
    height: 0.75rem;
  }

  /* AI badge */
  .privacy-ai-badge {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    font-size: 0.72rem;
    font-weight: 700;
    color: var(--color-accent, #22d3ee);
    background: rgba(34,211,238,0.1);
    border: 1px solid rgba(34,211,238,0.25);
    border-radius: 999px;
    padding: 0.2rem 0.55rem;
    vertical-align: middle;
  }

  /* Links */
  .privacy-link {
    color: var(--color-accent, #22d3ee);
    text-decoration: none;
    border-bottom: 1px solid rgba(34,211,238,0.3);
    transition: border-color 0.15s;
  }

  .privacy-link:hover {
    border-color: var(--color-accent, #22d3ee);
  }

  /* Footer */
  .privacy-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-top: 3rem;
    padding-top: 1.5rem;
    border-top: 1px solid var(--color-border, rgba(255,255,255,0.08));
    flex-wrap: wrap;
    gap: 1rem;
  }

  .privacy-back-link {
    font-size: 0.875rem;
    color: var(--color-accent, #22d3ee);
    text-decoration: none;
    transition: opacity 0.15s;
  }

  .privacy-back-link:hover { opacity: 0.75; }

  .privacy-footer-copy {
    font-size: 0.8rem;
    color: var(--color-muted, #94a3b8);
  }
</style>
