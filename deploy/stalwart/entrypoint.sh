#!/usr/bin/env bash
# Start Stalwart and, the first time, set it up (WPM-D81).
#
# A fresh Stalwart has no configuration: it starts in "bootstrap mode" with only a management listener on
# 8080. One setup call (the `Bootstrap` object) writes /etc/stalwart/config.json and the server must then be
# restarted to run normally. After that the plan in plan.ndjson (a domain with a catch-all and two accounts) is
# applied. Every step is safe to repeat, so a restart with the same volumes converges to the same state.
#
# STALWART_RECOVERY_ADMIN=user:password is required: it is the administrator the setup uses, and the one
# you use to look around. It is a demo credential, set in compose.yaml; never reuse it.
set -euo pipefail

: "${STALWART_RECOVERY_ADMIN:?set STALWART_RECOVERY_ADMIN=user:password}"
# The CLI caches the server schema under its home; the stalwart user has none that is writable.
export XDG_CACHE_HOME=/tmp/wpm-cli-cache
export STALWART_URL="http://127.0.0.1:8080"
export STALWART_USER="${STALWART_RECOVERY_ADMIN%%:*}"
export STALWART_PASSWORD="${STALWART_RECOVERY_ADMIN#*:}"
HOSTNAME_FQDN="${WPM_MAIL_HOSTNAME:-mail.wpm.test}"
DOMAIN="${WPM_MAIL_DOMAIN:-wpm.test}"
READY=/tmp/wpm-mail-ready
rm -f "$READY"

SERVER=
start() { stalwart --config /etc/stalwart/config.json & SERVER=$!; }
stop() { [ -n "$SERVER" ] && kill -TERM "$SERVER" 2>/dev/null && wait "$SERVER" 2>/dev/null || true; SERVER=; }
trap 'stop; exit 0' TERM INT

# Wait (up to about two minutes) for a command to succeed.
wait_for() {
  local what=$1; shift
  for _ in $(seq 1 120); do
    if "$@" >/dev/null 2>&1; then return 0; fi
    kill -0 "$SERVER" 2>/dev/null || { echo "stalwart exited while waiting for $what" >&2; exit 1; }
    sleep 1
  done
  echo "timed out waiting for $what" >&2; exit 1
}
http_up() { curl -sf -u "$STALWART_USER:$STALWART_PASSWORD" "$STALWART_URL/jmap/session"; }
smtp_up() { exec 3<>/dev/tcp/127.0.0.1/25 && head -n1 <&3 | grep -q '^220'; }

start
wait_for "the management listener" http_up

if [ ! -f /etc/stalwart/config.json ]; then
  echo "wpm: first start: setting the server up for $DOMAIN"
  stalwart-cli --no-color update Bootstrap --json "$(printf \
    '{"serverHostname":"%s","defaultDomain":"%s","requestTlsCertificate":false,"generateDkimKeys":false,"tracer":{"@type":"Stdout","enable":true,"ansi":false,"buffered":false,"level":"info"}}' \
    "$HOSTNAME_FQDN" "$DOMAIN")"
  for _ in $(seq 1 30); do [ -f /etc/stalwart/config.json ] && break; sleep 1; done
  [ -f /etc/stalwart/config.json ] || { echo "setup did not write config.json" >&2; exit 1; }
  stop
  start
  wait_for "the server to restart" http_up
fi

wait_for "SMTP" smtp_up
stalwart-cli --no-color apply --file /opt/wpm/plan.ndjson
touch "$READY"
echo "wpm: mail is ready: SMTP on 25 (no login, local domain only), accounts inbox@$DOMAIN and noreply@$DOMAIN"

# Hold the container to the server's life.
wait "$SERVER"
