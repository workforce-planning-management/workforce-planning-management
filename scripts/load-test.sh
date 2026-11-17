#!/usr/bin/env bash
# Load evidence (WPM-R120, WPM-T196): seed one organization at N workers in a scratch database, start the
# release binary against it, and measure the views that load an organization's records.
#
#   scripts/load-test.sh [sizes...]          default sizes: 1000 10000 50000
#   LOAD_SECONDS=10 LOAD_CONCURRENCY=8       per-endpoint duration and parallel connections
#
# Scenarios (WPM-T496), each on the same seed; without one, every path is measured alone at LOAD_CONCURRENCY:
#   LOAD_RAMP="8 16 32 64"   measure every path at each concurrency in turn, to find where the bar is first missed
#   LOAD_MIX=1               reads and writes at once: four reads, each at LOAD_CONCURRENCY, while LOAD_WRITERS (default 2)
#                            connections create workforce plans (a write with validation, an audit row in the hash chain
#                            and an outbox event); every stream is reported, so a write that slows the reads shows
#   LOAD_SOAK=<seconds>      the mixed traffic for that long, sampling the service's resident memory and the database's
#                            connections every LOAD_SAMPLE seconds (default 5); the first and last samples and the peaks
#                            are printed, so a leak or a connection that is never returned shows as a climb
#   LOAD_CHECK=1             exit 1 when any measured stream has a failed request (anything not 2xx), or a p95 over
#                            LOAD_BAR_MS (default 2000, the bar of WPM-R142); LOAD_BAR_MS=0 checks failures only
#   LOAD_LISTS=1             also seed the paged lists (WPM-T254): a requisition and an open leaver movement for one worker
#                            in ten, so /api/requisitions and /api/movements answer their first 500 out of n/10
#   LOAD_BIN=<path>          the service binary (default: the release build); LOAD_OUT=<file> also writes the rows as JSON lines
#
# Needs: curl, podman (a Postgres container), oha, and a release build of the service
# (cargo build --release in workforce-planning-management-service-with-rust). Synthetic data only.
# Auth enforcement is off (WPM_REQUIRE_AUTH=0): this measures the service and the database, not the token check.
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
svc="$root/workforce-planning-management-service-with-rust"
bin=${LOAD_BIN:-"$svc/target/release/workforce-planning-management"}
secs=${LOAD_SECONDS:-10}
conc=${LOAD_CONCURRENCY:-8}
sizes=("$@"); [ ${#sizes[@]} -gt 0 ] || sizes=(1000 10000 50000)
pg=wpm-load-pg
port=55460
port_api=5199

command -v oha >/dev/null || { echo "oha is not installed (cargo install oha)"; exit 1; }
[ -x "$bin" ] || { echo "no service binary at $bin (cargo build --release, or set LOAD_BIN)"; exit 1; }
results=$(mktemp); : > "$results"

cleanup() {
  [ -n "${api_pid:-}" ] && kill "$api_pid" 2>/dev/null || true
  podman rm -f "$pg" >/dev/null 2>&1 || true
  rm -f "$results"
}
trap cleanup EXIT

! curl -fs "127.0.0.1:$port_api/_health/live" >/dev/null 2>&1 || { echo "something already answers on port $port_api; stop it first"; exit 1; }
podman rm -f "$pg" >/dev/null 2>&1 || true
podman run -d --shm-size=512m --name "$pg" -e POSTGRES_USER=loco -e POSTGRES_PASSWORD=loco -e POSTGRES_DB=load \
  -p "$port:5432" docker.io/library/postgres:18@sha256:979bccfde35445db618c65107d27d08c6c21339d8dc3ad4c830c6633947be0e4 >/dev/null
for _ in $(seq 60); do podman exec "$pg" pg_isready -U loco -d load >/dev/null 2>&1 && break; sleep 1; done
sleep 2

psql() { podman exec -i "$pg" psql -v ON_ERROR_STOP=1 -q -U loco -d load "$@"; }
export DATABASE_URL="postgres://loco:loco@localhost:$port/load" WPM_REQUIRE_AUTH=0
# Rate limits are per network address and would answer a single-address load with 429; they are measured
# separately (tests/security.rs). Demo profile: it reads PORT, and the production profile demands a CORS origin and JWT secret.
export PORT=$port_api APP_ENV=demo WPM_RATE_LIMIT_PER_MINUTE=0 WPM_RATE_LIMIT_SENSITIVE_PER_MINUTE=0
# The demo profile holds ONE database connection with a 500 ms wait; a production deployment holds 20 (config/production.yaml).
# Measured with one, every query over half a second made the others fail with a pool timeout.
export DB_MAX_CONNECTIONS=${LOAD_DB_CONNECTIONS:-20} DB_CONNECT_TIMEOUT=${LOAD_DB_TIMEOUT_MS:-5000}
(cd "$svc" && "$bin" db migrate >/dev/null 2>&1)

echo "# machine: $(uname -sm), $(sysctl -n hw.ncpu 2>/dev/null || nproc) cpus; ${secs}s per endpoint, ${conc} connections"
echo "| workers | endpoint | connections | requests/s | p50 ms | p95 ms | p99 ms | non-2xx |"
echo "|---:|---|---:|---:|---:|---:|---:|---:|"

# measure <workers> <connections> <seconds> <path> [POST body]: one oha stream; prints a table row and appends it to $results.
measure() {
  local n=$1 c=$2 t=$3 path=$4 body=${5:-} out
  if [ -n "$body" ]; then
    out=$(oha -z "${t}s" -c "$c" --no-tui --output-format json -m POST -H 'content-type: application/json' -d "$body" \
      "http://127.0.0.1:$port_api$path" 2>/dev/null || echo '{}')
  else
    out=$(oha -z "${t}s" -c "$c" --no-tui --output-format json "http://127.0.0.1:$port_api$path" 2>/dev/null || echo '{}')
  fi
  echo "$out" | python3 -I -c '
import json, sys
n, c, path, label, results = sys.argv[1:6]
d = json.load(sys.stdin)
s = d.get("summary", {})
p = d.get("latencyPercentiles", {})
codes = d.get("statusCodeDistribution", {})
bad = sum(v for k, v in codes.items() if not k.startswith("2"))
# oha cancels the request each connection has in flight when the time is up; that is the end of the run, not a failure.
errors = sum(v for k, v in (d.get("errorDistribution") or {}).items() if k != "aborted due to deadline")
failed = bad + errors + (0 if (s.get("successRate") or 0) > 0 else 1)
row = {"workers": int(n), "endpoint": label, "connections": int(c), "rps": s.get("requestsPerSec", 0),
       "p50_ms": (p.get("p50") or 0) * 1000, "p95_ms": (p.get("p95") or 0) * 1000, "p99_ms": (p.get("p99") or 0) * 1000,
       "failed": failed}
print("| %(workers)d | `%(endpoint)s` | %(connections)d | %(rps).0f | %(p50_ms).1f | %(p95_ms).1f | %(p99_ms).1f | %(failed)d |" % row)
if bad or errors:
    print("<!-- %s %s: status %s, errors %s -->" % (n, label, codes, d.get("errorDistribution")))
with open(results, "a") as handle:
    handle.write(json.dumps(row) + "\n")
' "$n" "$c" "$path" "${body:+POST }$path" "$results"
}

# sample <seconds> <every> <file>: the service's resident memory (KiB) and the database's client connections, every few seconds.
sample() {
  local end=$((SECONDS + $1)) every=$2 file=$3
  : > "$file"
  while [ "$SECONDS" -lt "$end" ]; do
    rss=$(ps -o rss= -p "$api_pid" 2>/dev/null | tr -d ' ')
    db=$(psql -t -A -c "SELECT count(*) FROM pg_stat_activity WHERE backend_type = 'client backend'" 2>/dev/null | tr -d '[:space:]')
    echo "$SECONDS ${rss:-0} ${db:-0}" >> "$file"
    sleep "$every"
  done
}

seeded=0
for n in "${sizes[@]}"; do
  psql <<SQL
INSERT INTO skills (pid, name, category) VALUES ('00000000-0000-4000-8000-000000000001', 'load-skill', 'load')
  ON CONFLICT DO NOTHING;
INSERT INTO skill_pools (pid, organization_ref, name) VALUES ('00000000-0000-4000-8000-000000000002', 'organization:00000000-0000-4000-8000-0000000000aa', 'Load pool')
  ON CONFLICT DO NOTHING;
INSERT INTO skill_pool_members (pid, pool_pid, skill_pid, min_proficiency)
  SELECT '00000000-0000-4000-8000-000000000003', '00000000-0000-4000-8000-000000000002', '00000000-0000-4000-8000-000000000001', 2
  WHERE NOT EXISTS (SELECT 1 FROM skill_pool_members);
INSERT INTO workers (pid, person_ref, organization_ref, worker_number, display_name, status, employment_type,
                     fte_percent, department, job_title, hired_on)
  SELECT gen_random_uuid(), 'person:' || g, 'organization:00000000-0000-4000-8000-0000000000aa', 'W' || g, 'Synthetic Worker ' || g, 'active', 'permanent',
         100, 'dept-' || (g % 20), 'Engineer', DATE '2020-01-01' + (g % 1000)
  FROM generate_series($seeded + 1, $n) g;
INSERT INTO worker_skills (pid, worker_pid, skill_pid, proficiency)
  SELECT gen_random_uuid(), w.pid, '00000000-0000-4000-8000-000000000001', 1 + (w.id % 5)
  FROM workers w WHERE w.id > $seeded;
INSERT INTO leave_requests (pid, worker_pid, kind, start_on, end_on, days, status)
  SELECT gen_random_uuid(), w.pid, 'annual', CURRENT_DATE + (w.id % 60), CURRENT_DATE + (w.id % 60) + 4, 5, 'approved'
  FROM workers w WHERE w.id > $seeded AND w.id % 3 = 0;
INSERT INTO role_profiles (pid, job_title) VALUES ('00000000-0000-4000-8000-000000000004', 'Engineer')
  ON CONFLICT DO NOTHING;
INSERT INTO role_skill_requirements (pid, role_profile_pid, skill_pid, min_proficiency, importance)
  VALUES ('00000000-0000-4000-8000-000000000005', '00000000-0000-4000-8000-000000000004',
          '00000000-0000-4000-8000-000000000001', 5, 'critical')
  ON CONFLICT DO NOTHING;
INSERT INTO time_entries (pid, worker_pid, worked_on, minutes, kind, status)
  SELECT gen_random_uuid(), w.pid, CURRENT_DATE - d, CASE WHEN w.id % 100 = 0 THEN 3000 ELSE 480 END, 'regular', 'approved'
  FROM workers w, generate_series(1, 28) d
  WHERE w.id > $seeded AND EXTRACT(ISODOW FROM CURRENT_DATE - d) < 6;  -- weekdays: 20 entries, a 40-hour week is 2,400 minutes
ANALYZE;
SQL
  if [ -n "${LOAD_LISTS:-}" ]; then
    psql <<SQL
INSERT INTO requisitions (pid, organization_ref, department, job_title, headcount, status, opened_on)
  SELECT gen_random_uuid(), 'organization:00000000-0000-4000-8000-0000000000aa', 'dept-' || (g % 20), 'Engineer', 1, 'open', DATE '2026-01-01' + (g % 300)
  FROM generate_series($seeded + 1, $n) g WHERE g % 10 = 0;
INSERT INTO movements (pid, worker_pid, kind, effective_on, status)
  SELECT gen_random_uuid(), w.pid, 'leaver', CURRENT_DATE + (w.id % 90), 'open'
  FROM workers w WHERE w.id > $seeded AND w.id % 10 = 0;
ANALYZE;
SQL
  fi
  seeded=$n

  # exec: the pid is the service itself, so the kill below stops it and not just a wrapper shell
  (cd "$svc" && exec "$bin" start >"${LOAD_LOG:-/dev/null}" 2>&1) & api_pid=$!
  ready=0
  for _ in $(seq 60); do curl -fs "127.0.0.1:$port_api/_health/live" >/dev/null 2>&1 && { ready=1; break; }; sleep 1; done
  [ "$ready" = 1 ] || { echo "the service did not become ready"; exit 1; }

  # A plan with a demand line in every department (the workers are in dept-0..dept-19): the worst case for
  # the plan views, which read the workers of the departments the plan names.
  api="http://127.0.0.1:$port_api"
  plan=$(curl -fsS -X POST "$api/api/workforce-plans" -H 'content-type: application/json' \
    -d '{"name":"load","organization_ref":"organization:00000000-0000-4000-8000-0000000000aa","horizon_start":"2026-01-01","horizon_end":"2028-12-31","attrition_bp":1000}' \
    | python3 -I -c 'import json,sys; print(json.load(sys.stdin)["pid"])')
  for d in $(seq 0 19); do
    curl -fs -X PUT "$api/api/workforce-plans/$plan/demand-lines" -H 'content-type: application/json' \
      -d "{\"department\":\"dept-$d\",\"target_on\":\"2027-06-30\",\"target_headcount\":$((n / 20 + 10))}" >/dev/null
  done
  # a CPD requirement for every worker: the overview counts who has met it
  curl -fsS -X POST "$api/api/cpd-requirements" -H 'content-type: application/json' \
    -d '{"name":"load","unit":"hours","required":10,"period_start":"2026-01-01","period_end":"2026-12-31"}' >/dev/null
  worker=$(psql -t -A -c "SELECT pid FROM workers ORDER BY id LIMIT 1 OFFSET $((n / 2))" | tr -d '[:space:]')
  # LOAD_SURVEY=<file of paths>: one timed request per path instead of the load, to find the views that
  # read a whole table (WPM-R142). Slowest first; a path that needs a parameter answers 422 and is listed as such.
  # {PLAN} and {WORKER} in a path stand for the seed's plan and a worker from the middle of it.
  if [ -n "${LOAD_SURVEY:-}" ]; then
    org="organization:00000000-0000-4000-8000-0000000000aa"
    while read -r path; do
      path=${path//\{PLAN\}/$plan}; path=${path//\{WORKER\}/$worker}  # {PLAN} and {WORKER}: a plan and a worker of the seed
      join='?'; case "$path" in *\?*) join='&';; esac
      curl -s -o /tmp/wpm-survey-body -m 60 -w "%{time_total} %{http_code} %{size_download} $path" "$api$path${join}organization=$org"
      code=$(head -c 160 /tmp/wpm-survey-body | tr -d '\n'); [ ${#code} -lt 160 ] && [ "$(wc -c < /tmp/wpm-survey-body)" -lt 160 ] && printf ' %s' "$code"; echo
    done < "$LOAD_SURVEY" | sort -rn | head -${LOAD_SURVEY_TOP:-40}
    kill "$api_pid" 2>/dev/null || true; wait "$api_pid" 2>/dev/null || true; api_pid=
    continue
  fi
  # LOAD_PATHS=<file>: measure these paths (one per line) instead of the default set; the organization is added.
  if [ -n "${LOAD_PATHS:-}" ]; then
    paths=(); while read -r line; do line=${line//\{PLAN\}/$plan}; line=${line//\{WORKER\}/$worker}; paths+=("$line?organization=organization:00000000-0000-4000-8000-0000000000aa"); done < "$LOAD_PATHS"
  else
    paths=("/_health/live" "/api/workers?limit=50" "/api/directory?organization=organization:00000000-0000-4000-8000-0000000000aa" "/api/capacity?organization=organization:00000000-0000-4000-8000-0000000000aa" "/api/workforce-plans/$plan/forecast" "/api/workforce-plans/$plan/cost" "/api/cpd/overview" "/api/workforce-intelligence/overview" "/api/workforce-intelligence/metrics" "/api/workforce-intelligence/insights" "/api/workforce-intelligence/key-person?organization=organization:00000000-0000-4000-8000-0000000000aa" "/api/workforce/working-time")
  fi
  org="organization:00000000-0000-4000-8000-0000000000aa"
  plan_body="{\"name\":\"load write\",\"organization_ref\":\"$org\",\"horizon_start\":\"2026-01-01\",\"horizon_end\":\"2027-12-31\",\"attrition_bp\":1000}"
  mix_paths=("/api/workers?limit=50" "/api/directory?organization=$org" "/api/workforce-intelligence/metrics" "/api/workforce-plans/$plan/forecast")
  # mixed <seconds>: the four reads and the writers at once, each its own stream; waits for all of them.
  mixed() {
    local t=$1 pids=() mixed_out
    mixed_out=$(mktemp)
    for path in "${mix_paths[@]}"; do measure "$n" "$conc" "$t" "$path" >> "$mixed_out" & pids+=($!); done
    measure "$n" "${LOAD_WRITERS:-2}" "$t" /api/workforce-plans "$plan_body" >> "$mixed_out" & pids+=($!)
    for pid in "${pids[@]}"; do wait "$pid"; done
    cat "$mixed_out"; rm -f "$mixed_out"
  }
  if [ -n "${LOAD_SOAK:-}" ]; then
    samples=$(mktemp)
    sample "$LOAD_SOAK" "${LOAD_SAMPLE:-5}" "$samples" & sampler=$!
    mixed "$LOAD_SOAK"
    wait "$sampler" 2>/dev/null || true
    python3 -I - "$samples" "$n" <<'PY'
import sys
rows = [tuple(int(x) for x in line.split()) for line in open(sys.argv[1]) if line.strip()]
if rows:
    first, last = rows[0], rows[-1]
    print(f"<!-- soak, {sys.argv[2]} workers, {len(rows)} samples over {last[0] - first[0]} s: resident memory {first[1] // 1024} MiB "
          f"first, {last[1] // 1024} last, {max(r[1] for r in rows) // 1024} peak; database connections {first[2]} first, "
          f"{last[2]} last, {max(r[2] for r in rows)} peak -->")
PY
    rm -f "$samples"
  elif [ -n "${LOAD_MIX:-}" ]; then
    mixed "$secs"
  else
    for c in ${LOAD_RAMP:-$conc}; do
      for path in "${paths[@]}"; do measure "$n" "$c" "$secs" "$path"; done
    done
  fi
  kill "$api_pid" 2>/dev/null || true; wait "$api_pid" 2>/dev/null || true; api_pid=
done

# LOAD_OUT: the rows as JSON lines, for a later comparison. LOAD_CHECK: the bar, as an exit status.
if [ -n "${LOAD_OUT:-}" ]; then cp "$results" "$LOAD_OUT"; fi
if [ -n "${LOAD_CHECK:-}" ]; then
  python3 -I - "$results" "${LOAD_BAR_MS:-2000}" <<'PY'
import json, sys
bar = float(sys.argv[2])
rows = [json.loads(line) for line in open(sys.argv[1]) if line.strip()]
missed = [r for r in rows if r["failed"] or (bar > 0 and r["p95_ms"] > bar)]
for r in missed:
    print(f"missed the bar: {r['workers']} workers, {r['endpoint']}, {r['connections']} connections: "
          f"p95 {r['p95_ms']:.0f} ms, {r['failed']} failed")
print(f"{len(rows) - len(missed)} of {len(rows)} streams met the bar ({'no failed request' if bar <= 0 else f'p95 at most {bar:.0f} ms and no failed request'})")
sys.exit(1 if missed or not rows else 0)
PY
fi
