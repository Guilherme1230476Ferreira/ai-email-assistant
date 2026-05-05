#!/usr/bin/env bash
# =============================================================================
# DATABASE BOOTSTRAP SCRIPT for vs224
# Run as root: bash /opt/ai-email-assistant/deploy/db-bootstrap.sh
#
# This is separate from setup.sh so it can be re-run safely at any time
# without re-doing the full setup (Rust install, Node install, etc.)
# It is IDEMPOTENT — safe to run multiple times.
# =============================================================================
set -euo pipefail

REPO=/opt/ai-email-assistant
ENV_FILE=$REPO/.env

echo ""
echo "================================================="
echo " Database Bootstrap — vs224"
echo "================================================="
echo ""

# ── Load env vars ─────────────────────────────────────────────────────────────
if [ ! -f "$ENV_FILE" ]; then
    echo "❌ ERROR: $ENV_FILE not found!"
    echo "   The .env file must exist before running this script."
    exit 1
fi

source <(grep -E '^(POSTGRES_USER|POSTGRES_PASSWORD|POSTGRES_DB)' "$ENV_FILE")
: "${POSTGRES_USER:=admin}"
: "${POSTGRES_PASSWORD:=admin}"
: "${POSTGRES_DB:=ai_email_assistant}"

echo "  User:     $POSTGRES_USER"
echo "  Database: $POSTGRES_DB"
echo ""

# ── 1. Ensure PostgreSQL is running ──────────────────────────────────────────
echo "[1/5] Starting PostgreSQL..."
systemctl start postgresql
systemctl enable postgresql
sleep 2
echo "    ✅ PostgreSQL running."

# ── 2. Fix pg_hba.conf for TCP/IP auth ───────────────────────────────────────
echo "[2/5] Configuring pg_hba.conf for TCP auth..."
PG_HBA=$(find /etc/postgresql -name pg_hba.conf 2>/dev/null | head -1)
if [ -z "$PG_HBA" ]; then
    echo "    ❌ pg_hba.conf not found. Is PostgreSQL installed?"
    exit 1
fi

if ! grep -q "127.0.0.1/32.*md5\|127.0.0.1/32.*scram" "$PG_HBA"; then
    cp "$PG_HBA" "${PG_HBA}.bak.$(date +%s)"
    # Insert before the first "host" line, or append at end
    if grep -q "^# IPv4 local connections:" "$PG_HBA"; then
        sed -i '/^# IPv4 local connections:/a host    all    all    127.0.0.1\/32    md5' "$PG_HBA"
    else
        echo "host    all    all    127.0.0.1/32    md5" >> "$PG_HBA"
    fi
    systemctl restart postgresql
    sleep 2
    echo "    ✅ pg_hba.conf updated, PostgreSQL restarted."
else
    echo "    ✅ pg_hba.conf already configured."
fi

# ── 3. Create user and database ───────────────────────────────────────────────
echo "[3/5] Creating database user and database..."
sudo -u postgres psql << PSQL
DO \$\$ BEGIN
  IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = '${POSTGRES_USER}') THEN
    CREATE USER "${POSTGRES_USER}" WITH PASSWORD '${POSTGRES_PASSWORD}' SUPERUSER CREATEDB;
    RAISE NOTICE 'Created user ${POSTGRES_USER}';
  ELSE
    ALTER USER "${POSTGRES_USER}" WITH PASSWORD '${POSTGRES_PASSWORD}' SUPERUSER CREATEDB;
    RAISE NOTICE 'Updated user ${POSTGRES_USER}';
  END IF;
END \$\$;
PSQL

sudo -u postgres psql -tc "SELECT 1 FROM pg_database WHERE datname='${POSTGRES_DB}'" \
    | grep -q 1 \
    && echo "    Database '${POSTGRES_DB}' already exists." \
    || sudo -u postgres psql -c "CREATE DATABASE \"${POSTGRES_DB}\" OWNER \"${POSTGRES_USER}\";"

sudo -u postgres psql -d "${POSTGRES_DB}" << PSQL
GRANT ALL PRIVILEGES ON DATABASE "${POSTGRES_DB}" TO "${POSTGRES_USER}";
GRANT ALL ON SCHEMA public TO "${POSTGRES_USER}";
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON TABLES TO "${POSTGRES_USER}";
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON SEQUENCES TO "${POSTGRES_USER}";
PSQL

echo "    ✅ User and database ready."

# ── 4. Install pgvector extension ────────────────────────────────────────────
echo "[4/5] Enabling pgvector extension..."
PGPASSWORD="${POSTGRES_PASSWORD}" psql -h 127.0.0.1 -U "${POSTGRES_USER}" -d "${POSTGRES_DB}" \
    -c "CREATE EXTENSION IF NOT EXISTS vector;" 2>/dev/null \
    && echo "    ✅ pgvector enabled." \
    || echo "    ⚠ pgvector not available (install: apt-get install postgresql-16-pgvector)"

# ── 5. Run migrations ─────────────────────────────────────────────────────────
echo "[5/5] Running database migrations..."
MIGRATION_DIR="$REPO/backend/migrations"

if [ ! -d "$MIGRATION_DIR" ]; then
    echo "    ❌ Migrations directory not found: $MIGRATION_DIR"
    exit 1
fi

# Track which migrations have already been applied (via _sqlx_migrations table)
# If the table doesn't exist, apply all; if it does, sqlx handles deduplication.
if PGPASSWORD="${POSTGRES_PASSWORD}" psql -h 127.0.0.1 -U "${POSTGRES_USER}" -d "${POSTGRES_DB}" \
    -tc "SELECT 1 FROM information_schema.tables WHERE table_name='_sqlx_migrations'" \
    | grep -q 1; then
    echo "    _sqlx_migrations table exists — only applying new migrations."
else
    echo "    Fresh database — applying all migrations."
fi

for f in $(ls -1 "$MIGRATION_DIR"/*.sql | sort); do
    FILENAME=$(basename "$f")
    # Check if this migration was already applied (by version prefix)
    VERSION=$(echo "$FILENAME" | grep -oP '^\d+')
    APPLIED=$(PGPASSWORD="${POSTGRES_PASSWORD}" psql -h 127.0.0.1 -U "${POSTGRES_USER}" \
        -d "${POSTGRES_DB}" -tc \
        "SELECT 1 FROM information_schema.tables WHERE table_name='_sqlx_migrations'" \
        | grep -q 1 && \
        PGPASSWORD="${POSTGRES_PASSWORD}" psql -h 127.0.0.1 -U "${POSTGRES_USER}" \
        -d "${POSTGRES_DB}" -tc \
        "SELECT 1 FROM _sqlx_migrations WHERE version=${VERSION}" 2>/dev/null \
        | grep -q 1 && echo "yes" || echo "no")

    if [ "$APPLIED" = "yes" ]; then
        echo "    ↩ Already applied: $FILENAME"
    else
        echo "    → Applying: $FILENAME"
        PGPASSWORD="${POSTGRES_PASSWORD}" psql -h 127.0.0.1 -U "${POSTGRES_USER}" \
            -d "${POSTGRES_DB}" -f "$f" 2>&1
    fi
done

echo ""
echo "================================================="
echo " ✅ Database bootstrap complete!"
echo ""
echo " Verifying tables:"
PGPASSWORD="${POSTGRES_PASSWORD}" psql -h 127.0.0.1 -U "${POSTGRES_USER}" -d "${POSTGRES_DB}" \
    -c "\dt" 2>/dev/null || true
echo ""
echo " Connection test:"
PGPASSWORD="${POSTGRES_PASSWORD}" psql -h 127.0.0.1 -U "${POSTGRES_USER}" -d "${POSTGRES_DB}" \
    -c "SELECT COUNT(*) as user_count FROM users;" 2>/dev/null || true
echo "================================================="
echo ""
echo "Next: systemctl restart ai-backend"
echo ""
