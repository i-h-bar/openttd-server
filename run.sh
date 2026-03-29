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

echo "Building and starting Docker containers..."

docker compose build
docker compose up -d

echo "Script finished. Check your tunnel address at https://playit.gg"
