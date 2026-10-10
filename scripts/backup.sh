#!/usr/bin/env bash
# Back the database up (WPM-R89, WPM-R117): a custom-format pg_dump, ENCRYPTED, owner-only, with a
# checksum and a keyed tag, and old backups pruned.
#
#   WPM_BACKUP_KEY_FILE=/path/to/key DATABASE_URL=postgres://… scripts/backup.sh [OUTPUT_DIR]
#
# The key file's first line is the passphrase. Make one with
#   umask 077; openssl rand -base64 48 > backup.key
# and keep it APART from the backups (a secrets manager, a hardware module, a second person's
# safe). A backup without the key is unreadable; a key kept beside its backups protects nothing.
#
# The dump is piped straight into the encryption, so the plaintext never touches the disk.
# Output: wpm-<db>-<time>.dump.enc, its .sha256, and a .hmac (HMAC-SHA256 over the ciphertext,
# with a key derived from the same passphrase) that restore.sh checks BEFORE decrypting, so a
# wrong key or a damaged or altered file is named as such.
# The cipher is AES-256-CBC with PBKDF2 (600000 iterations) from the openssl in your PATH. It is
# a pragmatic choice that every host has; where you can, use your platform's own encrypted
# storage or a KMS as well.
#
# Without WPM_BACKUP_KEY_FILE the script REFUSES, unless WPM_BACKUP_ALLOW_PLAINTEXT=1 says you accept
# an unencrypted backup (then it warns). Database dumps hold everyone's personal data.
#
# OUTPUT_DIR defaults to ./backups. Keep the output off the host that holds the database.
# BACKUP_RETENTION_DAYS (default 35 calendar days) is how long a backup is kept:
# older ones matching wpm-*.dump* in OUTPUT_DIR are deleted, so erased data does
# not live on in backups indefinitely (WPM-D63).
set -euo pipefail

: "${DATABASE_URL:?set DATABASE_URL to the database to back up}"
out="${1:-./backups}"
keep="${BACKUP_RETENTION_DAYS:-35}"
case "$keep" in '' | *[!0-9]*) echo "BACKUP_RETENTION_DAYS must be a whole number of calendar days" >&2; exit 2 ;; esac

key="${WPM_BACKUP_KEY_FILE:-}"
if [ -n "$key" ]; then
  [ -r "$key" ] || { echo "WPM_BACKUP_KEY_FILE $key is not a readable file" >&2; exit 2; }
  pass="$(head -n1 "$key")"
  [ "${#pass}" -ge 32 ] || { echo "the first line of the key file must be at least 32 characters (use: openssl rand -base64 48)" >&2; exit 2; }
  if [ -n "$(find "$key" -maxdepth 0 -perm +077 2>/dev/null)" ]; then
    echo "warning: $key is readable by group or others; chmod 600 it" >&2
  fi
elif [ "${WPM_BACKUP_ALLOW_PLAINTEXT:-}" = "1" ]; then
  echo "WARNING: writing an UNENCRYPTED backup (WPM_BACKUP_ALLOW_PLAINTEXT=1). It holds everyone's personal data." >&2
else
  echo "refusing: no WPM_BACKUP_KEY_FILE. Backups are encrypted by default; see this script's header." >&2
  echo "(To accept an unencrypted backup, set WPM_BACKUP_ALLOW_PLAINTEXT=1.)" >&2
  exit 2
fi

mkdir -p "$out"
umask 077
stamp="$(date -u +%Y%m%dT%H%M%SZ)"
name="$(psql "$DATABASE_URL" -Atc 'select current_database()')"
file="$out/wpm-${name}-${stamp}.dump"

if [ -n "$key" ]; then
  file="$file.enc"
  pg_dump --format=custom --no-owner --no-privileges "$DATABASE_URL" \
    | openssl enc -aes-256-cbc -pbkdf2 -iter 600000 -salt -pass "file:$key" -out "$file"
  mac_key="$(printf 'wpm-backup-mac:%s' "$pass" | openssl dgst -sha256 -r | cut -d' ' -f1)"
  openssl dgst -sha256 -mac HMAC -macopt "hexkey:$mac_key" -r "$file" | cut -d' ' -f1 > "$file.hmac"
  chmod 600 "$file.hmac"
else
  pg_dump --format=custom --no-owner --no-privileges --file "$file" "$DATABASE_URL"
fi
chmod 600 "$file"
if command -v sha256sum >/dev/null 2>&1; then
  (cd "$out" && sha256sum "$(basename "$file")" > "$(basename "$file").sha256")
else
  (cd "$out" && shasum -a 256 "$(basename "$file")" > "$(basename "$file").sha256")
fi
chmod 600 "$file.sha256"

# Prune: only files this script names, and only older than the retention.
find "$out" -maxdepth 1 -type f \( -name 'wpm-*.dump' -o -name 'wpm-*.dump.sha256' -o -name 'wpm-*.dump.enc' -o -name 'wpm-*.dump.enc.sha256' -o -name 'wpm-*.dump.enc.hmac' \) \
  -mtime "+$keep" -print -delete | sed 's/^/pruned: /' >&2 || true

echo "$file"
