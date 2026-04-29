# ── Stage 1: Build ────────────────────────────────────────────────────────────
# Installs all deps and builds the SvelteKit Node.js server bundle.
FROM node:22-alpine AS builder

WORKDIR /app

COPY package.json package-lock.json ./
RUN npm ci

COPY . .
RUN npm run build

# ── Stage 2: Runtime ──────────────────────────────────────────────────────────
# Only ships the compiled build output and production dependencies.
FROM node:22-alpine AS runtime

WORKDIR /app

COPY --from=builder /app/build ./build
COPY --from=builder /app/package.json ./
COPY --from=builder /app/package-lock.json ./

RUN npm ci --omit=dev

ENV PORT=3001
ENV HOST=0.0.0.0

EXPOSE 3001

CMD ["node", "build"]
