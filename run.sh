#!/bin/bash

REQUIRED_VARS="OPENTTD_DATA_DIR PLAYIT_SECRET_KEY"
ENV_FILE="./.env"

if [ -f "$ENV_FILE" ]; then
    echo "Sourcing environment variables from '$ENV_FILE'..."
    set -a
    . "$ENV_FILE"
    set +a
else
    echo "Warning: No '$ENV_FILE' file found. Checking only system environment variables."
fi

echo "Checking for required environment variables..."

for var in $REQUIRED_VARS; do
    eval value=\$${var}
    if [ -z "$value" ]; then
        echo "Error: The environment variable '$var' is not set."
        exit 1
    fi
done

echo "All required environment variables are set. Proceeding..."

# Default PUID/PGID to the current user if not set
export PUID="${PUID:-$(id -u)}"
export PGID="${PGID:-$(id -g)}"
echo "Using PUID=${PUID} PGID=${PGID}"

# --- OpenTTD config management ---

OPENTTD_CFG="${OPENTTD_DATA_DIR}/openttd.cfg"
SECRETS_CFG="${OPENTTD_DATA_DIR}/secrets.cfg"
PRIVATE_CFG="${OPENTTD_DATA_DIR}/private.cfg"

# Sets (or adds) a key = value within a named [section] of an ini-style file.
# Handles missing sections and missing keys. Preserves all comments and formatting.
set_ini_value() {
    local file="$1" section="$2" key="$3" value="$4"
    awk -v sec="[$section]" -v k="$key" -v v="$value" '
    BEGIN { in_sec=0; done=0; found_sec=0 }
    /^\[/ {
        if (in_sec && !done) { print k " = " v; done=1 }
        in_sec = ($0 == sec)
        if (in_sec) found_sec=1
        print; next
    }
    in_sec && !done && $0 ~ "^" k " *=" {
        print k " = " v; done=1; next
    }
    { print }
    END {
        if (!done) {
            if (!found_sec) { print ""; print sec }
            print k " = " v
        }
    }
    ' "$file" > "${file}.tmp" && mv "${file}.tmp" "$file"
}

echo ""
echo "Checking OpenTTD config files in '${OPENTTD_DATA_DIR}'..."
mkdir -p "$OPENTTD_DATA_DIR"

# --- openttd.cfg ---
if [ ! -f "$OPENTTD_CFG" ]; then
    echo "  openttd.cfg not found — creating with defaults..."
    cat > "$OPENTTD_CFG" <<EOF
[network]
server_name = ${SERVER_NAME:-}
server_admin_port = 3977
server_admin_chat = true
allow_insecure_admin_login = true
EOF
    echo "  Created $OPENTTD_CFG"
else
    echo "  openttd.cfg found — checking settings..."
    set_ini_value "$OPENTTD_CFG" "network" "allow_insecure_admin_login" "true"
    echo "    allow_insecure_admin_login = true  ✓"
    if [ -n "${SERVER_NAME:-}" ]; then
        set_ini_value "$OPENTTD_CFG" "network" "server_name" "$SERVER_NAME"
        echo "    server_name synced (openttd.cfg)  ✓"
    fi
fi

# --- private.cfg ---
if [ ! -f "$PRIVATE_CFG" ]; then
    echo "  private.cfg not found — creating..."
    cat > "$PRIVATE_CFG" <<EOF
[network]
server_name = ${SERVER_NAME:-}
EOF
    echo "  Created $PRIVATE_CFG"
else
    echo "  private.cfg found — checking settings..."
    if [ -n "${SERVER_NAME:-}" ]; then
        set_ini_value "$PRIVATE_CFG" "network" "server_name" "$SERVER_NAME"
        echo "    server_name synced  ✓"
    fi
fi

# --- secrets.cfg ---
if [ ! -f "$SECRETS_CFG" ]; then
    echo "  secrets.cfg not found — creating..."
    cat > "$SECRETS_CFG" <<EOF
[network]
server_password = ${SERVER_PASSWORD:-}
rcon_password =
admin_password = ${OPENTTD_ADMIN_PASSWORD:-}
EOF
    echo "  Created $SECRETS_CFG"
else
    echo "  secrets.cfg found — checking settings..."
    if [ -n "${OPENTTD_ADMIN_PASSWORD:-}" ]; then
        set_ini_value "$SECRETS_CFG" "network" "admin_password" "$OPENTTD_ADMIN_PASSWORD"
        echo "    admin_password synced  ✓"
    fi
    if [ -n "${SERVER_PASSWORD:-}" ]; then
        set_ini_value "$SECRETS_CFG" "network" "server_password" "$SERVER_PASSWORD"
        echo "    server_password synced  ✓"
    fi
fi

echo "Config OK."
echo ""

echo "Building and starting Docker containers..."

docker compose build
docker compose up -d

echo "Script finished. Check your tunnel address at https://playit.gg"