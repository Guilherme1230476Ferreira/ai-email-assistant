# Backoffice Technical Documentation

## 1. Overview

The MailMate Backoffice is a **SvelteKit 2** web application that serves as the administrative dashboard for the AI email assistant. It provides a modern, responsive UI for managing users, roles, emails, LLM settings, the knowledge base, and monitoring RAG telemetry.

**Framework**: SvelteKit 2 with Svelte 5 (Runes)  
**Styling**: TailwindCSS 4 with custom design system  
**Dev Server**: `http://localhost:5173` (proxied to backend at `:3000`)  
**Production**: Built as Node.js app, served on port 3001 behind Nginx

---

## 2. Project Structure

```
backoffice/
├── src/
│   ├── app.html                  # HTML shell template
│   ├── app.css                   # Global styles + TailwindCSS imports
│   ├── app.d.ts                  # TypeScript ambient declarations
│   ├── hooks.server.ts           # Server hooks (cookie forwarding)
│   ├── lib/
│   │   ├── api.ts                # Centralized API client (typed fetch wrapper)
│   │   ├── token.ts              # JWT token utilities
│   │   ├── utils.ts              # Shared utility functions
│   │   ├── stores/
│   │   │   └── auth.ts           # Svelte store for auth token
│   │   ├── components/ui/        # Reusable UI components
│   │   │   ├── badge/            # Status badge component
│   │   │   ├── button/           # Button component with variants
│   │   │   ├── card/             # Card layout components
│   │   │   └── table/            # Data table components
│   │   └── assets/
│   │       └── favicon.svg       # Application favicon
│   └── routes/
│       ├── +layout.server.ts     # Root layout server load (auth + role resolution)
│       ├── +layout.svelte        # Root layout (sidebar + header + main)
│       ├── +page.server.ts       # Dashboard data loading
│       ├── +page.svelte          # Dashboard page
│       ├── login/                # Login page
│       ├── signup/               # Registration page
│       ├── logout/               # Logout action
│       ├── emails/               # Email management page
│       ├── users/                # User management page (admin)
│       ├── roles/                # Role management page (admin)
│       ├── settings/             # LLM settings page (admin)
│       ├── knowledge/            # Knowledge base management page (admin)
│       └── audit-logs/           # Audit logs viewer (admin)
├── static/                       # Static assets
├── svelte.config.js              # SvelteKit configuration
├── vite.config.ts                # Vite config with API proxy
├── tailwind.config.ts            # TailwindCSS configuration
└── package.json                  # Dependencies and scripts
```

---

## 3. Design System

### Color Palette
The backoffice uses a dark-mode-first design system with CSS custom properties:

| Token | Value | Usage |
|---|---|---|
| `--color-accent` | `#2dd4bf` (Teal) | Primary accent, active states, links |
| `--color-accent-hover` | `#14b8a6` | Hover states for accent elements |
| `--color-surface` | `#111113` | Card/panel backgrounds |
| `--color-surface-2` | `#1a1a1e` | Nested surface (inputs, hover states) |
| `--color-surface-3` | `#27272a` | Tertiary surface (avatars) |
| `--color-border` | `rgba(255,255,255,0.06)` | Subtle dividers and borders |
| `--color-muted` | `#52525b` | Muted text and icons |
| `--color-muted-foreground` | `#a1a1aa` | Secondary text |
| `--color-success` | `#10b981` | Success indicators |
| `--color-warning` | `#f59e0b` | Warning indicators |
| Background | `#09090b` (near-black) | Page background |

### Typography
- **Font**: Inter (Google Fonts)
- **Headings**: `font-semibold tracking-tight`
- **Body**: `text-sm` (14px)
- **Labels**: `text-xs uppercase tracking-wider`
- **Monospace**: System monospace for technical values

---

## 4. Routing & Pages

### Public Routes (No Auth Required)
| Route | Page | Description |
|---|---|---|
| `/login` | Login | Email/password login + Google OAuth button |
| `/signup` | Registration | New user registration form |

### Authenticated Routes (All Users)
| Route | Page | Description |
|---|---|---|
| `/` | Dashboard | Overview with stats, RAG telemetry, recent emails |
| `/emails` | Emails | Paginated list of generated emails with expand/copy/delete |

### Admin-Only Routes
| Route | Page | Description |
|---|---|---|
| `/users` | User Management | Create, list, update roles, delete users |
| `/roles` | Role Management | Create, list, delete roles |
| `/settings` | LLM Settings | Configure LLM provider URL, model, API key |
| `/knowledge` | Knowledge Base | Upload documents, add Q&A pairs, manage entries |
| `/audit-logs` | Audit Logs | View system audit trail (paginated) |

---

## 5. Layout & Navigation

### Collapsible Sidebar (`+layout.svelte`)
The application features a collapsible sidebar navigation:

- **Expanded Mode** (240px): Full icon + label navigation
- **Collapsed Mode** (68px): Icon-only with tooltip labels
- **Toggle**: Desktop-only chevron button at sidebar bottom
- **Mobile**: Slide-in overlay with backdrop, toggled by hamburger menu

### Role-Based Navigation
Navigation items are filtered based on the authenticated user's role:

| Item | Icon | Admin | User |
|---|---|---|---|
| Dashboard | `LayoutDashboard` | ✅ | ✅ |
| Emails | `Mail` | ✅ | ✅ |
| Users | `Users` | ✅ | ❌ |
| Roles | `Shield` | ✅ | ❌ |
| Settings | `Settings` | ✅ | ❌ |
| Knowledge Base | `BookOpen` | ✅ | ❌ |
| Audit Logs | `ShieldAlert` | ✅ | ❌ |

### Role Resolution
The layout server (`+layout.server.ts`) resolves the user's role name:
1. Fetches `/api/auth/me` to get `role_id`
2. Fetches `/api/admin/roles` to get all roles
3. Matches `role_id` to role name
4. Falls back to `'user'` if roles endpoint fails (non-admin users)

### Header Bar
- Role badge: Green "Admin" or neutral "User" indicator
- Connection status dot (green = connected)

---

## 6. Key Features

### Dashboard (`/`)
- **Stats Cards**: Emails generated, Users, Roles, LLM model (admin sees all, users see emails + model)
- **LLM Configuration Panel** (admin only): Model, base URL, API key status, embedding model
- **RAG Context Health**: Interactive radar chart showing context depth, similarity, and hallucination risk
- **AI Execution Telemetry**: Context rate, tokens processed, avg similarity, knowledge matches
- **Recent Generations**: Last 4 generated emails with quick links

### Email Management (`/emails`)
- **Search**: Filter emails by content or reply text
- **Generate Modal**: Paste incoming email → streams AI response in real-time terminal view
- **Expand View**: Click any email to see full original + generated reply
- **Copy Reply**: One-click copy button with confirmation
- **Delete**: Delete button with confirmation dialog and loading state
- **Pagination**: Page navigation for large email lists

### Knowledge Base (`/knowledge`)
- **Tabbed Interface**: "Q&A Pairs" and "Documents" tabs
- **Q&A Creation**: Question + answer text fields with instant creation
- **Document Upload**: Drag-and-drop zone supporting PDF, TXT, and Markdown files
- **File Validation**: Type checking and 10MB size limit
- **Entry List**: All knowledge entries with type badges (QA/Document), chunk counts, dates
- **Deletion**: Remove individual knowledge entries with confirmation

### User Management (`/users`) — Admin Only
- **User List**: Paginated table with email, role, creation date
- **Create User**: Email + password form
- **Role Assignment**: Inline role selector dropdown
- **Delete User**: Confirmation dialog with user email display

### Settings (`/settings`) — Admin Only
- **LLM Configuration**: Base URL, model name, API key (masked display)
- **Verify Connection**: Test button that pings the LLM provider
- **Save**: Updates are encrypted and stored in the database

---

## 7. API Client (`lib/api.ts`)

Centralized API client with typed methods:

```typescript
export const api = {
  // Auth
  me: () => request<ApiUser>('GET', '/auth/me'),
  
  // Users (Admin)
  getUsers: (page, limit) => request<PaginatedResponse>('GET', `/admin/users?page=${page}&limit=${limit}`),
  createUser: (email, password) => request<ApiUser>('POST', '/admin/users', { email, password }),
  deleteUser: (id) => request<void>('DELETE', `/admin/users/${id}`),
  updateUserRole: (userId, roleId) => request<ApiUser>('PUT', `/admin/users/${userId}/role`, { role_id: roleId }),
  
  // Roles
  getRoles: () => request<ApiRole[]>('GET', '/admin/roles'),
  createRole: (name) => request<ApiRole>('POST', '/admin/roles', { name }),
  deleteRole: (id) => request<void>('DELETE', `/admin/roles/${id}`),
  
  // Emails
  generateEmail: (prompt) => request<ApiEmail>('POST', '/emails/generate', { prompt }),
  deleteEmail: (id) => request<void>('DELETE', `/emails/${id}`),
  generateEmailStream: (prompt, onToken, onError, onDone) => { /* SSE streaming */ },
  
  // Settings
  updateSettings: (settings) => request<ApiSettings>('PUT', '/admin/settings', settings),
  verifySettings: (settings) => request<VerifyResult>('POST', '/admin/settings/verify', settings),
};
```

All requests include the JWT token from the auth store via `Authorization: Bearer <token>`.

---

## 8. Authentication Flow

### Login
1. User submits email/password on `/login`
2. Server action calls `POST /api/auth/login`
3. On success, JWT stored in HttpOnly cookie (`token`)
4. Redirect to dashboard

### Session Management
- Token is read from cookie in `+layout.server.ts` on every page load
- Token is validated by calling `/api/auth/me`
- If invalid/expired, user is redirected to `/login`
- In-memory auth store (`$lib/stores/auth.ts`) syncs token for client-side API calls

### Logout
- `POST /logout` server action clears the cookie
- Redirect to `/login`

---

## 9. Development

### Setup
```bash
cd backoffice
npm install
```

### Dev Server
```bash
npm run dev
# → http://localhost:5173
```

The Vite dev server proxies `/api/*` requests to `http://localhost:3000` (the Rust backend).

### Type Checking
```bash
npm run check
```

### Build
```bash
npm run build
# Output: build/ directory (Node.js adapter)
```

### Unit Tests
```bash
npm run test:unit
```
