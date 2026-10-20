#!/usr/bin/env bash
# Load evidence (WPM-R120, WPM-T196): seed one organization at N workers in a scratch database, start the
# release binary against it, and measure the views that load an organization's records.
#
#   scripts/load-test.sh [sizes...]          default sizes: 1000 10000 50000
#   LOAD_SECONDS=10 LOAD_CONCURRENCY=8       per-endpoint duration and parallel connections
#
# Needs: podman (a Postgres container), oha, and a release build of the service
# (cargo build --release in workforce-planning-management-service-with-rust). Synthetic data only.
# Auth enforcement is off (WPM_REQUIRE_AUTH=0): this measures the service and the database, not the token check.
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
svc="$root/workforce-planning-management-service-with-rust"
bin="$svc/target/release/workforce-planning-management"
secs=${LOAD_SECONDS:-10}
conc=${LOAD_CONCURRENCY:-8}
sizes=("$@"); [ ${#sizes[@]} -gt 0 ] || sizes=(1000 10000 50000)
pg=wpm-load-pg
port=55460
port_api=5199

command -v oha >/dev/null || { echo "oha is not installed (cargo install oha)"; exit 1; }
[ -x "$bin" ] || { echo "no release binary at $bin (cargo build --release)"; exit 1; }

cleanup() {
  [ -n "${api_pid:-}" ] && kill "$api_pid" 2>/dev/null || true
  podman rm -f "$pg" >/dev/null 2>&1 || true
}
trap cleanup EXIT

! curl -fs "127.0.0.1:$port_api/_ping" >/dev/null 2>&1 || { echo "something already answers on port $port_api; stop it first"; exit 1; }
podman rm -f "$pg" >/dev/null 2>&1 || true
podman run -d --name "$pg" -e POSTGRES_USER=loco -e POSTGRES_PASSWORD=loco -e POSTGRES_DB=load \
  -p "$port:5432" docker.io/library/postgres:17 >/dev/null
for _ in $(seq 60); do podman exec "$pg" pg_isready -U loco -d load >/dev/null 2>&1 && break; sleep 1; done
sleep 2

psql() { podman exec -i "$pg" psql -v ON_ERROR_STOP=1 -q -U loco -d load "$@"; }
export DATABASE_URL="postgres://loco:loco@localhost:$port/load" WPM_REQUIRE_AUTH=0
# Rate limits are per network address and would answer a single-address load with 429; they are measured
# separately (tests/security.rs). Demo profile: it reads PORT, and the production profile demands a CORS origin and JWT secret.
export PORT=$port_api LOCO_ENV=demo WPM_RATE_LIMIT_PER_MINUTE=0 WPM_RATE_LIMIT_SENSITIVE_PER_MINUTE=0
(cd "$svc" && "$bin" db migrate >/dev/null 2>&1)

echo "# machine: $(uname -sm), $(sysctl -n hw.ncpu 2>/dev/null || nproc) cpus; ${secs}s per endpoint, ${conc} connections"
echo "| workers | endpoint | requests/s | p50 ms | p99 ms | non-2xx |"
echo "|---:|---|---:|---:|---:|---:|"

seeded=0
for n in "${sizes[@]}"; do
  psql <<SQL
INSERT INTO skills (pid, name, category) VALUES ('00000000-0000-4000-8000-000000000001', 'load-skill', 'load')
  ON CONFLICT DO NOTHING;
INSERT INTO skill_pools (pid, organization_ref, name) VALUES ('00000000-0000-4000-8000-000000000002', 'organization:load', 'Load pool')
  ON CONFLICT DO NOTHING;
INSERT INTO skill_pool_members (pid, pool_pid, skill_pid, min_proficiency)
  SELECT '00000000-0000-4000-8000-000000000003', '00000000-0000-4000-8000-000000000002', '00000000-0000-4000-8000-000000000001', 2
  WHERE NOT EXISTS (SELECT 1 FROM skill_pool_members);
INSERT INTO workers (pid, person_ref, organization_ref, worker_number, display_name, status, employment_type,
                     fte_percent, department, job_title, hired_on)
  SELECT gen_random_uuid(), 'person:' || g, 'organization:load', 'W' || g, 'Synthetic Worker ' || g, 'active', 'permanent',
         100, 'dept-' || (g % 20), 'Engineer', DATE '2020-01-01' + (g % 1000)
  FROM generate_series($seeded + 1, $n) g;
INSERT INTO worker_skills (pid, worker_pid, skill_pid, proficiency)
  SELECT gen_random_uuid(), w.pid, '00000000-0000-4000-8000-000000000001', 1 + (w.id % 5)
  FROM workers w WHERE w.id > $seeded;
INSERT INTO leave_requests (pid, worker_pid, kind, start_on, end_on, days, status)
  SELECT gen_random_uuid(), w.pid, 'annual', CURRENT_DATE + (w.id % 60), CURRENT_DATE + (w.id % 60) + 4, 5, 'approved'
  FROM workers w WHERE w.id > $seeded AND w.id % 3 = 0;
ANALYZE;
SQL
  seeded=$n

  # exec: the pid is the service itself, so the kill below stops it and not just a wrapper shell
  (cd "$svc" && exec "$bin" start >"${LOAD_LOG:-/dev/null}" 2>&1) & api_pid=$!
  ready=0
  for _ in $(seq 60); do curl -fs "127.0.0.1:$port_api/_ping" >/dev/null 2>&1 && { ready=1; break; }; sleep 1; done
  [ "$ready" = 1 ] || { echo "the service did not become ready"; exit 1; }

  for path in "/_ping" "/api/workers?limit=50" "/api/directory?organization=organization:load" "/api/capacity?organization=organization:load"; do
    out=$(oha -z "${secs}s" -c "$conc" --no-tui --output-format json "http://127.0.0.1:$port_api$path" 2>/dev/null || echo '{}')
    echo "$out" | python3 -I -c '
import json, sys
n, path = sys.argv[1], sys.argv[2]
d = json.load(sys.stdin)
s = d.get("summary", {})
p = d.get("latencyPercentiles", {})
codes = d.get("statusCodeDistribution", {})
bad = sum(v for k, v in codes.items() if not k.startswith("2"))
print("| %s | `%s` | %.0f | %.1f | %.1f | %d |" % (n, path, s.get("requestsPerSec", 0), (p.get("p50") or 0)*1000, (p.get("p99") or 0)*1000, bad + (0 if s.get("successRate", 0) > 0 else 1)))
if bad or d.get("errorDistribution"):
    print("<!-- %s %s: status %s, errors %s -->" % (n, path, codes, d.get("errorDistribution")))
' "$n" "$path"
  done
  kill "$api_pid" 2>/dev/null || true; wait "$api_pid" 2>/dev/null || true; api_pid=
done
