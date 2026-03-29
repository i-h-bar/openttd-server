#!/bin/sh
set -e

echo "[tcp-guard] Applying configuration..."
envsubst '${TCP_GUARD_PROXY_TIMEOUT} ${TCP_GUARD_CONNECT_TIMEOUT} ${TCP_GUARD_PREREAD_TIMEOUT}' \
    < /etc/nginx/nginx.conf > /tmp/nginx-rendered.conf

echo "[tcp-guard] Starting nginx TCP/UDP proxy on port 3979..."
exec nginx -g 'daemon off;' -c /tmp/nginx-rendered.conf
