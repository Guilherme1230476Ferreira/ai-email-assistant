# Privacy & GDPR Compliance — AI Email Assistant

## Legal Framework

| Regulation | Applicability |
|---|---|
| **GDPR** — Regulation (EU) 2016/679 | Processing personal data of EU residents |
| **Lei n.º 58/2019** | Portuguese national GDPR implementation |
| **EU AI Act** — Regulation (EU) 2024/1689 | AI-generated content transparency obligations |

---

## Data Controller

The data controller is the operator of the AI Email Assistant service, deployed at **ISEP — Instituto Superior de Engenharia do Porto**, Portugal.

Contact for data protection enquiries: `admin@isep.ipp.pt`

---

## Personal Data Processed

| Data Category | Examples | Legal Basis (GDPR Art.) | Retention |
|---|---|---|---|
| Account data | Email address, password hash, role | Art. 6(1)(b) — contract | Until account deletion |
| Email content | Incoming emails, AI-generated replies | Art. 6(1)(b) — contract | Until deleted by user or account erasure |
| Email embeddings | Vector representations of email text | Art. 6(1)(b) — contract | Cascade-deleted with email |
| Knowledge base | Uploaded documents and their chunk vectors | Art. 6(1)(b) — contract | Until deleted by admin |
| Audit logs | Admin actions with timestamp and user ID | Art. 6(1)(c) — legal obligation | 90 days |
| Authentication tokens | JWT (stored client-side in cookie, HttpOnly) | Art. 6(1)(b) — contract | Session duration (24h default) |

### Data NOT collected

- No tracking pixels or advertising cookies
- No third-party analytics (no Google Analytics, Mixpanel, etc.)
- No device fingerprinting
- No IP address logging beyond access logs

---

## AI Act Compliance (Article 50)

The AI Email Assistant generates content using Large Language Models (LLMs). The following measures are implemented per **EU AI Act Art. 50 — Transparency obligations for providers of AI systems**:

### AI-Generated Content Labelling

Every AI-generated email reply displayed in the UI carries an explicit **✦ AI Generated** badge. This badge is:

- Visually prominent (cyan accent colour, bold font)
- Present on every AI-generated reply without exception
- Rendered before the content, not hidden below the fold

### Implementation

**File:** `backoffice/src/routes/emails/+page.svelte`

```html
<span class="ai-badge">✦ AI Generated</span>
```

---

## GDPR Data Subject Rights — Implementation

All rights are exercisable by authenticated users via the **My Account** page (`/account`).

| Right | GDPR Article | Implementation |
|---|---|---|
| **Right of Access** | Art. 15 | `GET /api/auth/me/data` — exports all personal data as JSON |
| **Right to Erasure** | Art. 17 | `DELETE /api/auth/me` — deletes account + all associated data via DB cascade |
| **Right to Rectification** | Art. 16 | Email address update via account settings |
| **Right to Object** | Art. 21 | Users may stop using the service and request erasure at any time |
| **Right to Information** | Art. 13 | Privacy Policy linked on login page at point of data collection |

### Cascade Deletion (Art. 17 Technical Implementation)

When a user account is deleted (`DELETE /api/auth/me`), the following data is automatically removed via PostgreSQL `ON DELETE CASCADE` foreign key constraints:

```
users
 └── emails (ON DELETE CASCADE)
      └── email_embeddings (ON DELETE CASCADE)
```

Knowledge base entries are admin-managed resources and are not user-specific; they are not deleted on user erasure.

### Data Export Format

`GET /api/auth/me/data` returns:

```json
{
  "user": {
    "id": "...",
    "email": "...",
    "created_at": "..."
  },
  "emails": [
    {
      "id": "...",
      "subject": "...",
      "body": "...",
      "generated_reply": "...",
      "created_at": "..."
    }
  ]
}
```

---

## Security Measures

| Measure | Implementation |
|---|---|
| Password hashing | Argon2id (memory-hard, resistant to GPU cracking) |
| API keys at rest | AES-256-GCM encrypted in database |
| Authentication | JWT (HS256), HttpOnly cookie, 24h expiry |
| Transport | HTTPS enforced via nginx (TLS termination) |
| CSRF protection | SvelteKit CSRF origin checking enabled |
| Rate limiting | Per-IP rate limiting on auth endpoints |
| SQL injection | All queries use parameterised statements via `sqlx` |

---

## Third-Party Data Processors

| Processor | Data Shared | Purpose | Privacy Policy |
|---|---|---|---|
| **Jina AI** | Text chunks from uploaded documents | Embedding generation | [jina.ai/legal](https://jina.ai/legal/) |
| **Google (Gemini API)** | Email content + context for generation | LLM inference | [policies.google.com](https://policies.google.com/privacy) |
| **ISEP Server Infrastructure** | All data (self-hosted) | Hosting | ISEP IT Policy |

> ⚠️ **Note:** Text content sent to Jina AI and Google Gemini is processed outside the EU. Both providers offer Data Processing Agreements (DPAs) under GDPR Art. 28. Ensure a DPA is in place if processing sensitive personal data.

---

## Privacy Policy UI

The privacy policy is publicly accessible at `/privacy` without authentication.

It is linked from:
- The login page footer (point of data collection — GDPR Art. 13 compliance)
- The My Account page

**File:** `backoffice/src/routes/privacy/+page.svelte`

The page:
- Requires no login
- Has no sidebar (standalone public page)
- Is themed consistently with the main application
- Last updated: May 26, 2026

---

## Cookie Usage

| Cookie | Type | Purpose | Expiry |
|---|---|---|---|
| `token` | HttpOnly, Secure, SameSite=Strict | JWT authentication | 24 hours |

No marketing, tracking, or analytics cookies are used.

---

## Data Breach Procedure

In the event of a personal data breach, the data controller must:

1. Assess severity within **24 hours** of discovery
2. Notify the **CNPD** (Comissão Nacional de Proteção de Dados) within **72 hours** if the breach poses a risk to individuals (GDPR Art. 33)
3. Notify affected users without undue delay if the breach is likely to result in a **high risk** to their rights and freedoms (GDPR Art. 34)

Contact CNPD: [cnpd.pt](https://www.cnpd.pt)

---

## Changelog

| Date | Change |
|---|---|
| 2026-05-26 | Initial GDPR compliance implementation: data export, account deletion, privacy policy page, AI-generated content badges |
| 2026-05-26 | Added EU AI Act Art. 50 compliance: AI-generated content labelling |
