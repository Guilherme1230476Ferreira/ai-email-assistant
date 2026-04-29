#!/usr/bin/env bash
# =============================================================================
# ONE-TIME SETUP SCRIPT for vs224 (native deployment — no Docker)
# Run as root: bash /opt/ai-email-assistant/deploy/setup.sh
# =============================================================================
set -euo pipefail

REPO=/opt/ai-email-assistant
ENV_FILE=$REPO/.env

echo ""
echo "================================================="
echo " AI Email Assistant — One-Time Setup"
echo "================================================="
echo ""

# ── 1. System dependencies ────────────────────────────────────────────────────
echo "[1/7] Installing system packages..."
apt-get update -qq
apt-get install -y -qq \
    curl wget gnupg2 lsb-release ca-certificates \
    pkg-config libssl-dev build-essential \
    nginx

# ── 2. PostgreSQL 16 + pgvector ───────────────────────────────────────────────
echo "[2/7] Installing PostgreSQL 16 + pgvector..."
if ! command -v psql &>/dev/null; then
    curl -fsSL https://www.postgresql.org/media/keys/ACCC4CF8.asc \
        | gpg --dearmor -o /etc/apt/trusted.gpg.d/postgresql.gpg
    echo "deb https://apt.postgresql.org/pub/repos/apt $(lsb_release -cs)-pgdg main" \
        > /etc/apt/sources.list.d/pgdg.list
    apt-get update -qq
    apt-get install -y -qq postgresql-16
fi
apt-get install -y -qq postgresql-16-pgvector

systemctl enable --now postgresql

# ── 3. Database setup ─────────────────────────────────────────────────────────
echo "[3/7] Setting up database..."
# Read vars from .env
source <(grep -E '^(POSTGRES_USER|POSTGRES_PASSWORD|POSTGRES_DB)' "$ENV_FILE" | sed 's/^/export /')

sudo -u postgres psql -tc \
    "SELECT 1 FROM pg_roles WHERE rolname='${POSTGRES_USER}'" \
    | grep -q 1 || \
    sudo -u postgres psql -c \
    "CREATE USER ${POSTGRES_USER} WITH PASSWORD '${POSTGRES_PASSWORD}';"

sudo -u postgres psql -tc \
    "SELECT 1 FROM pg_database WHERE datname='${POSTGRES_DB}'" \
    | grep -q 1 || \
    sudo -u postgres psql -c \
    "CREATE DATABASE ${POSTGRES_DB} OWNER ${POSTGRES_USER};"

sudo -u postgres psql -d "${POSTGRES_DB}" \
    -c "CREATE EXTENSION IF NOT EXISTS vector;"

echo "    Database '${POSTGRES_DB}' ready with pgvector."

# ── 4. Rust toolchain ─────────────────────────────────────────────────────────
echo "[4/7] Ensuring Rust toolchain is installed..."
if ! command -v cargo &>/dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path
    source "$HOME/.cargo/env"
fi
# Make cargo available system-wide for the runner
ln -sf "$HOME/.cargo/bin/cargo" /usr/local/bin/cargo 2>/dev/null || true
ln -sf "$HOME/.cargo/bin/rustc" /usr/local/bin/rustc 2>/dev/null || true
echo "    Rust $(rustc --version)"

# ── 5. First build ────────────────────────────────────────────────────────────
echo "[5/7] Building backend and backoffice (first build — takes a few minutes)..."

# Backend
cd "$REPO/backend"
SQLX_OFFLINE=true cargo build --release
echo "    Backend compiled."

# Backoffice
cd "$REPO/backoffice"
npm ci --silent
npm run build --silent
echo "    Backoffice built."

# ── 6. Nginx ──────────────────────────────────────────────────────────────────
echo "[6/7] Configuring Nginx..."
cp "$REPO/nginx/nginx.conf" /etc/nginx/sites-available/ai-email-assistant
ln -sf /etc/nginx/sites-available/ai-email-assistant \
       /etc/nginx/sites-enabled/ai-email-assistant
rm -f /etc/nginx/sites-enabled/default
nginx -t
systemctl enable --now nginx
systemctl reload nginx

# ── 7. systemd services ───────────────────────────────────────────────────────
echo "[7/7] Installing and enabling systemd services..."
cp "$REPO/deploy/ai-backend.service"   /etc/systemd/system/
cp "$REPO/deploy/ai-backoffice.service" /etc/systemd/system/
systemctl daemon-reload
systemctl enable --now ai-backend
systemctl enable --now ai-backoffice

echo ""
echo "================================================="
echo " Setup complete!"
echo " Backend:   http://localhost:3000"
echo " Backoffice: http://localhost:3001"
echo " Public:     http://vs224.dei.isep.ipp.pt"
echo "================================================="
echo ""
echo "Check status:"
echo "  systemctl status ai-backend"
echo "  systemctl status ai-backoffice"
echo "  journalctl -u ai-backend -f"
echo ""
