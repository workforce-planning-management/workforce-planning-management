# Tutorial: the family bridges

How WPM talks to its two siblings, **project-portfolio-management (PPM)** and
**contact-relationship-management (CRM)**, one bridge at a time. Each step is a
`curl` against a local service; the rules behind each one are in
[spec/cross-app-bridges.md](../../spec/cross-app-bridges.md) (WPM-R206 to
WPM-R216, WPM-D134 to WPM-D147).

> ⚠️ **Demo software.** Synthetic data only. See
> [spec/regulatory.md](../../spec/regulatory.md).

Every bridge carries **references and counts, never records** (WPM-D134), and
every inbound call acts **under the caller's own token** (WPM-D135). Here
sign-in is turned off, so no token is sent; in a real deployment PPM and CRM
forward their user's `Authorization: Bearer` token, and WPM's policy decides
exactly as it does for that user on the same route.

**What this tutorial does not show.** PPM's sender and puller and CRM's webhook
receiver and import are built by their owners and have not been run against
WPM; here `curl` plays both siblings.

## 0. Start a throwaway service

You need PostgreSQL, Rust, `jq` and `uuidgen` (see [INSTALL.md](../../INSTALL.md)).
Use an empty database that holds nothing real, because sign-in is off:

```sh
cd workforce-planning-management-service-with-rust
export DATABASE_URL=postgres://localhost:5432/wpm_tutorial   # an empty database
cargo run -- db migrate
export WPM_REQUIRE_AUTH=0                   # never on a service that holds real data
export WPM_WEBHOOK_SECRET_CRM=tutorial-only-secret
cargo run -- start &                        # the API on :5150
```

In another shell, make an organization reference and a few workers in one
department (the person references are made up here; in the family they are
person-service records). A worker created by HR starts `onboarding`; the
function makes each one `active`:

```sh
API=localhost:5150/api
ORG="organization:$(uuidgen | tr A-Z a-z)"
hire() {  # hire <worker number> -> prints the worker's pid
  local pid
  pid=$(curl -s $API/workers -H 'content-type: application/json' -d '{
    "person_ref": "person:'$(uuidgen | tr A-Z a-z)'", "organization_ref": "'$ORG'",
    "worker_number": "'$1'", "display_name": "Tutorial '$1'",
    "employment_type": "permanent", "department": "tutorial",
    "job_title": "Engineer", "salary_minor": 3600000, "salary_currency": "GBP",
    "hired_on": "2026-01-05" }' | jq -r .pid)
  curl -s $API/workers/$pid/status -H 'content-type: application/json' -d '{"to": "active"}' >/dev/null
  echo $pid
}
ADA=$(hire T-1); hire T-2 >/dev/null; hire T-3 >/dev/null
ADA_PERSON=$(curl -s $API/workers/$ADA | jq -r .person_ref)
```

## 1. Link a pool to PPM's pool (WPM-R213, WPM-D139)

A skill pool is WPM's; the link to PPM's pool for the same skills is set by a
person, by id, never matched by name. PPM holds the WPM pool's pid in its own
`peer_pool_ref`.

```sh
POOL=$(curl -s $API/skill-pools -H 'content-type: application/json' \
  -d '{"organization_ref": "'$ORG'", "name": "Tutorial engineers"}' | jq -r .pid)
PPM_POOL=$(uuidgen | tr A-Z a-z)            # PPM's pool pid, as PPM shows it
curl -s -X PUT $API/skill-pools/$POOL/peer-link -H 'content-type: application/json' \
  -d '{"peer_pool_ref": "'$PPM_POOL'"}' | jq '{peer_pool_ref, demand_source}'
```

A linked pool counts PPM's demand and ignores what was entered here (one owner
per fact, WPM-D140). Until PPM sends a figure, its demand is **unknown**, not
zero (WPM-D141). `{"peer_pool_ref": null}` removes the link and the local
figures count again at once.

## 2. PPM sends demand; WPM gives supply (WPM-R210, WPM-R215)

PPM names its plan as a `thing:` reference to the plan's pid. A figure is in
hundredths of an FTE for the first day of a month, with PPM's own event id.
An event id is a repeat only for the same pool, programme and month: the same
id for another pool, programme or month is a new event there (WPM-D148). Here
each run makes its own:

```sh
PLAN="thing:$(uuidgen | tr A-Z a-z)"
MONTH=$(date +%Y-%m-01)
EV=$(uuidgen | tr A-Z a-z)                  # a prefix for this run's event ids
demand() {  # demand <fte_centi> <event id> <sent_at>
  curl -s $API/capacity/peer-demand -H 'content-type: application/json' -d '{
    "pool_pid": "'$POOL'", "programme_ref": "'$PLAN'", "month": "'$MONTH'",
    "fte_centi": '$1', "peer_event_id": "'$2'", "sent_at": "'$3'" }' | jq -c .
}
demand 150 $EV-1 2026-10-30T10:00:00Z   # {"duplicate":false,"outcome":"recorded",...,"status":"proposed"}
demand 150 $EV-1 2026-10-30T10:00:00Z   # the same event again: duplicate, nothing changes
demand 120 $EV-0 2026-10-30T09:00:00Z   # sent before $EV-1: "stale", ignored
demand 0   $EV-2 2026-10-30T11:00:00Z   # zero withdraws it: "withdrawn", status "closed"
curl -s "$API/programme-demands?pool=$POOL" | jq '.[] | {source, status, fte_centi, peer_sent_at, peer_withdrawn_at}'
```

A received claim arrives `proposed` and counts only after a person's start
decision (WPM-D50); a withdrawal never undoes that decision. The full table of
outcomes is in [cross-app-bridges.md](../../spec/cross-app-bridges.md), WPM-R215.

PPM pulls the pool's supply, by month, in hundredths of an FTE. A month made up
of one to four workers is hidden (`supply_centi: null`, `hidden: true`), and no
total is given:

```sh
curl -s "$API/capacity/peer-supply?pool=$POOL&months=3" | jq -c '.months[]'
# {"hidden":false,"month":"2026-10-01","supply_centi":0} ... one line a month
```

This pool names no role profile or skill, so nobody counts towards it and each
month is a known zero (zero is shown; it names no one). A pool's members come
from its role profiles and skills, as in the capacity view
([spec/delivery-capacity.md](../../spec/delivery-capacity.md)).

## 3. PPM asks for people impact (WPM-R211)

Counts only, for a programme: people committed to it by pool, contingent
engagements ending before the programme ends, and over-committed pool-months.
WPM holds no end date for a PPM plan, so PPM passes `ends_on`; without it that
part is unknown with a reason. A count under ten is hidden.

```sh
curl -s "$API/capacity/people-impact?programme=$PLAN&ends_on=2027-03-31" \
  | jq '{workers_committed: .workers_committed.known, reason: .workers_committed.reason,
         over: .over_committed_pool_months.over}'
```

Here no post is recorded as funded by the plan, so `workers_committed` is
unknown: WPM cannot tell "nobody" from "not recorded", and says so.

## 4. CRM hears of a hire or a leaver by webhook (WPM-R207, WPM-D145)

CRM subscribes an endpoint to the three kinds. The signing secret stays in the
service's environment; the subscription holds only its variable's name:

```sh
SUB=$(curl -s $API/webhook-subscriptions -H 'content-type: application/json' -d '{
  "name": "crm-handoff", "url": "http://localhost:9999/wpm-events",
  "kinds": ["worker_hired", "worker_terminated", "worker_retired"],
  "secret_ref": "WPM_WEBHOOK_SECRET_CRM" }' | jq -r .pid)
NEW=$(hire T-4)
cargo run -- task webhook_deliver           # the scheduler runs this every minute
curl -s $API/webhook-subscriptions/$SUB/deliveries | jq -c '.[0] | {kind, status, attempts, last_error}'
# {"kind":"worker_hired","status":"pending","attempts":1,"last_error":"request failed"}
```

The delivery `POST`ed to the endpoint carries the handoff references and
nothing else: no name, department or pay.

```json
{ "id": "<event uuid>", "kind": "worker_hired", "subject": "worker:<pid>", "occurred_at": "…",
  "data": { "person_ref": "person:<uuid>", "organization_ref": "organization:<uuid>", "effective_on": "2026-01-05" } }
```

Nothing listens on port 9999 here, so the delivery stays `pending`, with its
error and the time of its next attempt (`next_attempt_at`), and is retried with
back-off. A receiver checks `x-wpm-signature` (`t=<unix>,v1=<hex
HMAC-SHA256 of "<t>.<body>">`) and is idempotent on the event `id`, because
delivery is at least once.

## 5. CRM checks which of its contacts are workers (WPM-R208)

CRM already holds `person:` references. It may ask, for up to 500 of them,
which are workers of the organization today; the answer is a yes or no for each
and nothing more, and the audit entry holds counts, not references:

```sh
curl -s $API/bridges/overlap -H 'content-type: application/json' -d '{
  "organization_ref": "'$ORG'",
  "person_refs": ["'$ADA_PERSON'", "person:'$(uuidgen | tr A-Z a-z)'"] }' | jq -c '.results[]'
# {"person_ref":"person:…","worker":true}
# {"person_ref":"person:…","worker":false}
```

## 6. CRM's import asks for a lean worker list (WPM-R216, WPM-D147)

CRM's teams import needs three fields of each active worker. It names them, and
gets those and no others:

```sh
curl -s "$API/workers?status=active&fields=pid,department,person_ref&limit=200&offset=0" | jq -c '.[0]'
# {"department":"tutorial","person_ref":"person:…","pid":"…"}
curl -s -o /dev/null -w '%{http_code}\n' "$API/workers?fields=pid,nmae"   # an unknown name: 422
```

A list that carries a salary (no `fields`, or `fields` naming `salary_minor` or
`salary_currency`) records a `salary_read` audit entry for each worker whose
salary it shows unmasked, as the single-worker read does; the lean list above
records none.

## Clean up

Stop the service and drop the database (`dropdb wpm_tutorial`).
