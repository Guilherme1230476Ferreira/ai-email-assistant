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
echo "[1/8] Installing system packages..."
apt-get update -qq
apt-get install -y -qq \
    curl wget gnupg2 lsb-release ca-certificates \
    pkg-config libssl-dev build-essential \
    nginx git

# ── 2. PostgreSQL 16 + pgvector ───────────────────────────────────────────────
echo "[2/8] Installing PostgreSQL 16 + pgvector..."
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

# ── 2b. Configure PostgreSQL auth for TCP connections ─────────────────────────
echo "    Configuring PostgreSQL authentication..."
PG_HBA=$(find /etc/postgresql -name pg_hba.conf 2>/dev/null | head -1)
if [ -n "$PG_HBA" ]; then
    # Check if we already added md5 auth for localhost TCP
    if ! grep -q "host.*all.*all.*127.0.0.1/32.*md5" "$PG_HBA" 2>/dev/null; then
        # Backup original
        cp "$PG_HBA" "${PG_HBA}.bak.$(date +%s)"
        # Add md5 auth for TCP connections on localhost (before any existing host lines)
        # This allows the backend to connect via DATABASE_URL=postgresql://...@localhost:5432/...
        sed -i '/^# IPv4 local connections:/a host    all             all             127.0.0.1/32            md5' "$PG_HBA" 2>/dev/null || \
            echo "host    all             all             127.0.0.1/32            md5" >> "$PG_HBA"
        systemctl restart postgresql
        echo "    pg_hba.conf updated for md5 auth on localhost TCP."
    else
        echo "    pg_hba.conf already configured."
    fi
else
    echo "    ⚠ Could not find pg_hba.conf — check PostgreSQL installation."
fi

# ── 3. Database setup ─────────────────────────────────────────────────────────
echo "[3/8] Setting up database..."

# Create .env if it doesn't exist
if [ ! -f "$ENV_FILE" ]; then
    echo "    Creating $ENV_FILE with default values..."
    cat > "$ENV_FILE" << 'ENVEOF'
# ─── PostgreSQL ──────────────────────────────────────────────────
POSTGRES_USER=admin
POSTGRES_PASSWORD=admin
POSTGRES_DB=ai_email_assistant
DATABASE_URL=postgresql://admin:admin@localhost:5432/ai_email_assistant

# ─── LLM API Keys ───────────────────────────────────────────────
OPENAI_API_KEY=
GROK_API_KEY=
EMBEDDING_API_URL=https://generativelanguage.googleapis.com/v1beta/openai
EMBEDDING_MODEL=gemini-embedding-001
EMBEDDING_API_KEY=

# ─── Security ───────────────────────────────────────────────────
JWT_SECRET=CHANGE_ME_TO_A_RANDOM_STRING
JWT_EXPIRATION_HOURS=24
ENCRYPTION_KEY=CHANGE_ME_TO_A_RANDOM_KEY_32CHAR

# ─── Google OAuth ───────────────────────────────────────────────
GOOGLE_CLIENT_ID=
GOOGLE_CLIENT_SECRET=

# ─── Server ─────────────────────────────────────────────────────
PORT=3000
ENVEOF
    chmod 600 "$ENV_FILE"
    echo ""
    echo "    ╔══════════════════════════════════════════════════════════╗"
    echo "    ║  ⚠  IMPORTANT: Edit /opt/ai-email-assistant/.env       ║"
    echo "    ║     Fill in your actual API keys and secrets!           ║"
    echo "    ╚══════════════════════════════════════════════════════════╝"
    echo ""
fi

# Read DB vars from .env
source <(grep -E '^(POSTGRES_USER|POSTGRES_PASSWORD|POSTGRES_DB|DATABASE_URL)' "$ENV_FILE" | sed 's/^/export /')

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

# Grant necessary privileges
sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE ${POSTGRES_DB} TO ${POSTGRES_USER};"
sudo -u postgres psql -d "${POSTGRES_DB}" -c "GRANT ALL ON SCHEMA public TO ${POSTGRES_USER};"

sudo -u postgres psql -d "${POSTGRES_DB}" \
    -c "CREATE EXTENSION IF NOT EXISTS vector;"

echo "    Database '${POSTGRES_DB}' ready with pgvector."

# ── 4. Node.js (LTS) ──────────────────────────────────────────────────────────
echo "[4/8] Ensuring Node.js is installed..."
if ! command -v node &>/dev/null; then
    echo "    Installing Node.js LTS..."
    curl -fsSL https://deb.nodesource.com/setup_lts.x | bash -
    apt-get install -y -qq nodejs
fi
echo "    Node.js $(node --version), npm $(npm --version)"

# ── 5. Rust toolchain ─────────────────────────────────────────────────────────
echo "[5/8] Ensuring Rust toolchain is installed..."
if ! command -v cargo &>/dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path
    source "$HOME/.cargo/env"
fi
# Make cargo available system-wide for the runner
ln -sf "$HOME/.cargo/bin/cargo" /usr/local/bin/cargo 2>/dev/null || true
ln -sf "$HOME/.cargo/bin/rustc" /usr/local/bin/rustc 2>/dev/null || true
echo "    Rust $(rustc --version)"

# Install sqlx-cli for migrations
if ! command -v sqlx &>/dev/null; then
    echo "    Installing sqlx-cli..."
    cargo install sqlx-cli --no-default-features --features postgres
    ln -sf "$HOME/.cargo/bin/sqlx" /usr/local/bin/sqlx 2>/dev/null || true
fi

# ── 6. Run database migrations ────────────────────────────────────────────────
echo "[6/8] Running database migrations..."
cd "$REPO/backend"
export DATABASE_URL
sqlx migrate run
echo "    Migrations applied."

# ── 7. First build ────────────────────────────────────────────────────────────
echo "[7/8] Building backend and backoffice (first build — takes a few minutes)..."

# Backend
cd "$REPO/backend"
SQLX_OFFLINE=true cargo build --release
echo "    Backend compiled."

# Backoffice
cd "$REPO/backoffice"
npm ci --silent
npm run build --silent
echo "    Backoffice built."

# ── 8. Nginx + systemd services ──────────────────────────────────────────────
echo "[8/8] Configuring Nginx and systemd services..."

# Nginx
cp "$REPO/nginx/nginx.conf" /etc/nginx/sites-available/ai-email-assistant
ln -sf /etc/nginx/sites-available/ai-email-assistant \
       /etc/nginx/sites-enabled/ai-email-assistant
rm -f /etc/nginx/sites-enabled/default
nginx -t
systemctl enable --now nginx
systemctl reload nginx

# Systemd services
cp "$REPO/deploy/ai-backend.service"   /etc/systemd/system/
cp "$REPO/deploy/ai-backoffice.service" /etc/systemd/system/
systemctl daemon-reload
systemctl enable --now ai-backend
systemctl enable --now ai-backoffice

echo ""
echo "================================================="
echo " ✅ Setup complete!"
echo ""
echo " Backend:    http://localhost:3000"
echo " Backoffice: http://localhost:3001"
echo " Public:     http://vs224.dei.isep.ipp.pt:2224"
echo "================================================="
echo ""
echo "Check status:"
echo "  systemctl status ai-backend"
echo "  systemctl status ai-backoffice"
echo "  journalctl -u ai-backend -f"
echo "  journalctl -u ai-backoffice -f"
echo ""
echo "⚠  Remember to edit /opt/ai-email-assistant/.env"
echo "   with your actual API keys and secrets!"
echo ""
