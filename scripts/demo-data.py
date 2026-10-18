#!/usr/bin/env python3
"""Fill a DEMO service with fictitious data so every page of the UI has something to show.

Run it once, after `cargo run -- task seed`, against a service started with WPM_REQUIRE_AUTH=0 that holds
nothing real (it is for the guided tour: sign in as example@example.com, see `ui AGENTS.md`):

    python3 scripts/demo-data.py [http://localhost:5150]

It only talks to the public API (so every rule that guards a write still applies) and uses the standard
library only. Every name is obviously fictitious. Writes are rate-limited by the service (429); the script
waits the time it is told and carries on. A step that fails prints one line and the rest still run.
"""
import datetime as dt
import json
import sys
import time
import urllib.error
import urllib.request
import uuid

BASE = (sys.argv[1] if len(sys.argv) > 1 else "http://localhost:5150").rstrip("/") + "/api"
TODAY = dt.date.today()
failures = []


def call(method, path, body=None, quiet=False):
    for _ in range(20):
        req = urllib.request.Request(
            BASE + path,
            method=method,
            data=None if body is None else json.dumps(body).encode(),
            headers={"content-type": "application/json", "idempotency-key": uuid.uuid4().hex},
        )
        try:
            with urllib.request.urlopen(req) as r:
                raw = r.read()
                return json.loads(raw) if raw else {}
        except urllib.error.HTTPError as e:
            if e.code == 429:
                time.sleep(int(e.headers.get("retry-after", "10")) + 1)
                continue
            if not quiet:
                msg = e.read()[:160].decode(errors="replace")
                failures.append(f"{method} {path} -> {e.code} {msg}")
                print(f"  ! {method} {path} -> {e.code} {msg}")
            return None
    return None


def get(path):
    return call("GET", path, quiet=True)


def day(n):
    return str(TODAY + dt.timedelta(days=n))


def month(n):
    m = TODAY.replace(day=1)
    y, mo = divmod(m.year * 12 + m.month - 1 + n, 12)
    return dt.date(y, mo + 1, 1)


def step(name):
    print(f"- {name}")


workers = []
for _ in range(15):  # the service may still be starting
    workers = get("/workers") or []
    if workers:
        break
    time.sleep(2)
if not workers:
    sys.exit("No workers: run `cargo run -- task seed` first.")
ORG = workers[0]["organization_ref"]
by_dept = {}
for w in workers:
    by_dept.setdefault(w["department"], []).append(w)
active = [w for w in workers if w["status"] == "active"]
perm = [w for w in active if w["employment_type"] == "permanent"]
fixed = [w for w in active if w["employment_type"] == "fixed_term"]
eng = by_dept.get("engineering", [])
PROG = "thing:" + str(uuid.UUID(int=1))
PROG2 = "thing:" + str(uuid.UUID(int=2))


def course(n):
    """A course reference: the learning records name courses by URN."""
    return "course:" + str(uuid.UUID(int=n))


FIN_GO_LIVE, PLAT_101, PLAT_201 = course(0xF1), course(0x101), course(0x201)

step("more organizations: three regions and a parent that contains them all")
REGIONS = [("north", "North region"), ("south", "South region"), ("central", "Central services")]
region_orgs = []
for slug, title in REGIONS:
    org = "organization:" + str(uuid.uuid4())
    region_orgs.append(org)
    depts = ["operations", "clinical", "finance"]
    for i in range(12):
        w = call("POST", "/workers", {
            "person_ref": f"person:{uuid.uuid4()}", "organization_ref": org,
            "worker_number": f"{slug[:2].upper()}-{i:04d}", "display_name": f"Fictitious {title} Worker {i:02d}",
            "employment_type": "permanent" if i % 5 else "fixed_term", "department": depts[i % 3],
            "job_title": ["Coordinator", "Analyst", "Nurse", "Engineer"][i % 4],
            "hired_on": day(-900 + i * 40), **({"engagement_ends_on": day(120 + i * 20)} if i % 5 == 0 else {}),
        })
        if w:
            call("POST", f"/workers/{w['pid']}/status", {"to": "active"}, quiet=True)
    call("POST", "/governance-commitments", {"organization_ref": org, "name": f"{title} monthly review", "role": "Regional lead", "senior": True, "kind": "review", "frequency": "monthly", "minutes": 60, "holders": 2})
    lv = [call("POST", "/oversight/levels", {"organization_ref": org, "ordinal": n, "name": f"Level {n}"}) for n in (1, 2, 3)]
    if lv[0]:
        call("POST", "/oversight/positions", {"organization_ref": org, "level_pid": lv[0]["pid"], "effective_on": day(-60), "deciding_body": "Fictitious regional board", "reason": "Fictitious starting position"})
PARENT = "organization:" + str(uuid.uuid4())
for child in [ORG] + region_orgs:
    call("POST", "/organization-confederations", {"parent_organization_ref": PARENT, "child_organization_ref": child, "starts_on": day(-400)})

step("skills, role profile, declared levels")
sk_db = call("POST", "/skills", {"name": "Database administration", "category": "technical"})
sk_ir = call("POST", "/skills", {"name": "Incident command", "category": "leadership"})
sk_sm = call("POST", "/skills", {"name": "Stakeholder management", "category": "leadership"})
rp = call("POST", "/role-profiles", {"job_title": "Platform engineer", "description": "Keeps the shared platform running."})
if rp and sk_db and sk_ir and sk_sm:
    call("PUT", f"/role-profiles/{rp['pid']}/requirements", {"skill_pid": sk_db["pid"], "min_proficiency": 4, "importance": "critical"})
    call("PUT", f"/role-profiles/{rp['pid']}/requirements", {"skill_pid": sk_ir["pid"], "min_proficiency": 3, "importance": "critical"})
    call("PUT", f"/role-profiles/{rp['pid']}/requirements", {"skill_pid": sk_sm["pid"], "min_proficiency": 3, "importance": "important"})
    for i, w in enumerate(eng[:6]):
        call("PUT", f"/workers/{w['pid']}/skills", {"skill_pid": sk_db["pid"], "proficiency": [5, 4, 3, 3, 2, 2][i], "target": 4})
        call("PUT", f"/workers/{w['pid']}/skills", {"skill_pid": sk_ir["pid"], "proficiency": [4, 3, 3, 2, 2, 1][i], "target": 4})
        call("PUT", f"/workers/{w['pid']}/skills", {"skill_pid": sk_sm["pid"], "proficiency": [3, 3, 2, 2, 1, 1][i], "target": 3})
    if len(eng) > 1:
        call("POST", f"/workers/{eng[2]['pid']}/backups", {"backup_pid": eng[1]["pid"]})

step("capacity: a skill pool, claims, a start decision")
pool = call("POST", "/skill-pools", {"organization_ref": ORG, "name": "Platform engineers", "operations_reservation_bp": 2000})
if pool:
    call("POST", f"/skill-pools/{pool['pid']}/members", {"skill_pid": sk_db["pid"], "min_proficiency": 1})
    for org in region_orgs[:2]:
        rp_pool = call("POST", "/skill-pools", {"organization_ref": org, "name": "Regional delivery", "operations_reservation_bp": 1500})
        if rp_pool:
            call("POST", f"/skill-pools/{rp_pool['pid']}/members", {"role_profile_pid": rp["pid"]})
            call("PUT", "/programme-demands", {"programme_ref": PROG2, "pool_pid": rp_pool["pid"], "month": str(month(1)), "fte_centi": 150})
    for n, fte in enumerate([300, 500, 800, 1100]):
        call("PUT", "/programme-demands", {"programme_ref": PROG, "pool_pid": pool["pid"], "month": str(month(n)), "fte_centi": fte})
    call("PUT", "/programme-demands", {"programme_ref": PROG2, "pool_pid": pool["pid"], "month": str(month(5)), "fte_centi": 250})
    call("POST", "/capacity/start-decisions", {"organization_ref": ORG, "programme_ref": PROG, "decision": "proceed", "reason": "Fictitious: start despite the shortfall."})
    call("PUT", "/capacity/settings", {"organization_ref": ORG, "wip_limit": 2})

step("governance load")
for name, role, senior, kind, freq, mins, holders in [
    ("Quarterly board", "Director", True, "meeting", "quarterly", 180, 5),
    ("Monthly performance review", "Operations lead", True, "review", "monthly", 90, 3),
    ("Weekly stand-up", "Analyst", False, "meeting", "weekly", 30, 6),
]:
    call("POST", "/governance-commitments", {"organization_ref": ORG, "name": name, "role": role, "senior": senior, "kind": kind, "frequency": freq, "minutes": mins, "holders": holders})

step("oversight: reported and derived measures with readings")
status = get(f"/oversight/status?organization={ORG}") or {}
levels = status.get("scale", [])
if len(levels) >= 2:
    crit = call("POST", "/oversight/criteria", {"level_pid": levels[1]["pid"], "name": "Cover is adequate"})
    if crit:
        meas = call("POST", "/oversight/measures", {"criterion_pid": crit["pid"], "name": "On-call cover", "origin": "reported", "unit": "percent", "threshold": 90, "direction": "at_least", "sustain": 3})
        for back, val in [(1, 94), (2, 92), (3, 91)]:
            if meas:
                call("PUT", f"/oversight/measures/{meas['pid']}/readings", {"period_on": str(month(-back)), "value": val, "evidence": "Rota audit", "source_document": f"rota-audit-{back}.pdf"})
    crit2 = call("POST", "/oversight/criteria", {"level_pid": levels[1]["pid"], "name": "Turnover is under control"})
    if crit2:
        call("POST", "/oversight/measures", {"criterion_pid": crit2["pid"], "name": "Turnover rate", "origin": "derived", "derivation": "turnover_rate", "unit": "basis points", "threshold": 2500, "direction": "at_most", "sustain": 3})
    call("POST", "/oversight/packs", {"organization_ref": ORG})

step("on-call rota, with a swap")
rota = call("POST", "/rotas", {"organization_ref": ORG, "name": "Platform on-call", "period_days": 7, "starts_on": day(-14), "members": [w["pid"] for w in eng[:4]]})
if rota and len(eng) > 4:
    call("POST", f"/rotas/{rota['pid']}/overrides", {"worker_pid": eng[4]["pid"], "starts_on": day(3), "ends_on": day(5), "note": "Fictitious cover for leave"})

step("cutover with a role and a pool")
cut = call("POST", "/cutovers", {"organization_ref": ORG, "programme_ref": PROG, "name": "Finance system go-live", "site": "Main site", "window_start": day(21), "window_end": day(34), "site_aliases": ["HQ"], "rota_pid": rota["pid"] if rota else None})
if cut and pool and rp:
    call("POST", f"/cutovers/{cut['pid']}/pools", {"pool_pid": pool["pid"]})
    for w in eng[:2]:
        t = call("POST", f"/workers/{w['pid']}/training-enrollments", {"course_ref": FIN_GO_LIVE})
        if t:
            call("PUT", f"/training-enrollments/{t['pid']}", {"status": "completed", "completed_on": day(-5), "certificate_expires_on": day(400)})
    call("POST", f"/cutovers/{cut['pid']}/roles", {"role_profile_pid": rp["pid"], "course_ref": FIN_GO_LIVE, "required_headcount": 4})

step("contingent workforce: contractors, funding, an extension, a conversion plan")
contractors = []
for i in range(11):
    w = call("POST", "/workers", {"person_ref": f"person:{uuid.uuid4()}", "organization_ref": ORG, "worker_number": f"DEMO-C-{i:03d}", "display_name": f"Fictitious Contractor {i:02d}", "employment_type": "contractor", "department": ["engineering", "operations", "finance"][i % 3], "job_title": "Contract specialist", "hired_on": day(-200 + i * 7), "engagement_ends_on": day(20 + i * 12)})
    if w:
        contractors.append(w)
        call("POST", f"/workers/{w['pid']}/status", {"to": "active"})
        call("PUT", f"/workers/{w['pid']}/funding", {"programme_ref": PROG if i % 2 == 0 else PROG2, "funding_ends_on": day(60 + i * 10), "fte_percent": 100}, quiet=True)
if contractors:
    call("POST", f"/workers/{contractors[0]['pid']}/engagement/extensions", {"new_end": day(150), "reason": "Fictitious: programme extended"})
ft = [w for w in fixed if w.get("engagement_ends_on")]
if ft:
    call("POST", f"/workers/{ft[0]['pid']}/conversion-plans", {"intent": "convert", "target_on": day(30), "post_funding_kind": "core", "reason": "Fictitious: the post is now core funded"})

step("workforce plan with demand and objectives")
plan = call("POST", "/workforce-plans", {"name": "Next two years", "organization_ref": ORG, "horizon_start": day(0), "horizon_end": str(TODAY.replace(year=TODAY.year + 2)), "attrition_bp": 800})
if plan:
    nxt = str(TODAY.replace(year=TODAY.year + 1))
    call("PUT", f"/workforce-plans/{plan['pid']}/demand-lines", {"department": "engineering", "target_on": nxt, "target_headcount": 14, "programme_ref": PROG})
    call("PUT", f"/workforce-plans/{plan['pid']}/demand-lines", {"department": "clinical", "target_on": nxt, "target_headcount": 9})
    call("PUT", f"/workforce-plans/{plan['pid']}/demand-lines", {"department": "finance", "target_on": nxt, "target_headcount": 7, "basis": "fixed_term", "expected_ends_on": str(TODAY.replace(year=TODAY.year + 2))})
    call("POST", f"/workforce-plans/{plan['pid']}/objectives", {"title": "Grow platform capacity"})
    call("POST", f"/workforce-plans/{plan['pid']}/status", {"to": "active"})

step("groups")
for name in ["Wellbeing champions", "Women in engineering network", "Graduate cohort"]:
    g = call("POST", "/groups", {"scope": "organization", "organization_ref": ORG, "name": name, "kind": "practice" if "network" not in name else "interest", "description": "Fictitious community"})
    if g:
        for w in perm[:5]:
            call("POST", f"/groups/{g['pid']}/members", {"worker_pid": w["pid"], "role": "member"}, quiet=True)

step("joiners and leavers")
if len(perm) > 8:
    call("POST", f"/workers/{perm[7]['pid']}/movements", {"kind": "leaver", "effective_on": day(21), "reason": "resignation", "notes": "Fictitious leaver"})
    call("POST", f"/workers/{perm[8]['pid']}/movements", {"kind": "joiner", "effective_on": day(-3), "notes": "Fictitious joiner"}, quiet=True)

step("learning: a path, enrolments, training, mentoring")
path = call("POST", "/learning-paths", {"name": "Platform foundations", "summary": "Fictitious path", "steps": [{"course_ref": PLAT_101, "title": "Platform basics"}, {"course_ref": PLAT_201, "title": "Incident response"}]})
for w in eng[:4]:
    if path:
        call("POST", f"/learning-paths/{path['pid']}/enrollments", {"worker_pid": w["pid"]}, quiet=True)
    tr = call("POST", f"/workers/{w['pid']}/training-enrollments", {"course_ref": PLAT_101}, quiet=True)
    if tr:
        call("PUT", f"/training-enrollments/{tr['pid']}", {"status": "completed", "completed_on": day(-30), "certificate_expires_on": day(300)}, quiet=True)
if len(eng) > 3:
    ms = call("POST", "/mentorships", {"mentor_pid": eng[0]["pid"], "mentee_pid": eng[3]["pid"], "focus": "Incident command"})
    if ms:
        call("POST", f"/mentorships/{ms['pid']}/status", {"to": "active"})
        call("POST", f"/mentorships/{ms['pid']}/sessions", {"held_on": day(-9), "notes": "Fictitious: first session."})

step("reviews, goals, succession")
cyc = call("POST", "/review-cycles", {"name": "Annual review", "period_start": day(-300), "period_end": day(-10)})
if cyc and perm:
    rv = call("POST", f"/review-cycles/{cyc['pid']}/reviews", {"worker_pid": perm[0]["pid"], "reviewer_ref": "worker:" + (eng[0]["pid"] if eng else perm[1]["pid"])})
    if rv:
        call("POST", f"/reviews/{rv['pid']}/goals", {"title": "Run the incident drill", "weight_percent": 50})
sp = call("POST", "/succession-plans", {"role_title": "Head of Platform", "department": "engineering", "criticality": 5, "incumbent_pid": eng[0]["pid"] if eng else None})
if sp and len(eng) > 1:
    call("POST", f"/succession-plans/{sp['pid']}/candidates", {"worker_pid": eng[1]["pid"], "readiness": "ready_now", "rank": 1})

step("CPD and development plans")
call("POST", "/cpd-requirements", {"name": "Annual CPD hours", "unit": "hours", "required": 30, "period_start": day(-100), "period_end": day(260)})
if eng:
    call("POST", f"/workers/{eng[0]['pid']}/cpd-entries", {"entry_date": day(-20), "activity": "Conference talk", "category": "conference", "unit": "hours", "amount": 6})
    if sk_db and perm:
        # The demo user's own board (the first permanent worker, as the demo's memberships say): one step in each status.
        dp = call("POST", f"/workers/{perm[0]['pid']}/development-plans", {"kind": "upskill", "rationale": "Fictitious", "target_on": day(180), "items": [
            {"skill_pid": sk_db["pid"], "current_level": 2, "target_level": 4, "method": "course", "course_ref": PLAT_201, "due_on": day(120)},
            {"skill_pid": sk_ir["pid"], "current_level": 1, "target_level": 3, "method": "mentorship"},
            {"skill_pid": sk_sm["pid"], "current_level": 2, "target_level": 3, "method": "on_the_job"},
        ]})
        if dp:
            call("POST", f"/development-plans/{dp['pid']}/status", {"to": "active"})
            plan_items = (get(f"/workers/{perm[0]['pid']}/development-plans") or {}).get("plans", [{}])[0].get("items", [])
            for it, to in zip(plan_items, ["planned", "in_progress", "achieved"]):
                if to != "planned":
                    call("PUT", f"/development-plan-items/{it['pid']}", {"status": to})

step("wellbeing, pulse survey, announcements")
ent = call("POST", "/wellbeing-entitlements", {"name": "Eye test", "kind": "health", "description": "Fictitious entitlement offered to desk-based roles."})
sv = call("POST", "/pulse-surveys", {"name": "Q3 pulse", "question": "How supported do you feel? (1 to 5)", "active_from": day(-10), "active_until": day(20)})
if sv:
    for i, w in enumerate(perm[:8]):
        call("POST", f"/pulse-surveys/{sv['pid']}/responses", {"worker_pid": w["pid"], "score": 3 + (i % 3)}, quiet=True)
for title, body, pinned in [("Welcome to the fictitious organization", "Everything in this demo is made up.", True), ("Platform on-call changes", "A new on-call rota starts next week.", False)]:
    call("POST", "/announcements", {"organization_ref": ORG, "title": title, "body": body, "pinned": pinned, "publish_on": day(-2)})

step("payroll and expenses")
run = call("POST", "/payroll-runs", {"organization_ref": ORG, "period_start": str(TODAY.replace(day=1) - dt.timedelta(days=30)), "period_end": str(TODAY.replace(day=1) - dt.timedelta(days=1)), "notes": "Fictitious run"})
if run:
    call("POST", f"/payroll-runs/{run['pid']}/calculate")
if perm:
    claim = call("POST", f"/workers/{perm[0]['pid']}/expense-claims", {"title": "Conference travel", "currency": "GBP", "description": "Fictitious"})
    if claim:
        call("POST", f"/expense-claims/{claim['pid']}/items", {"incurred_on": day(-12), "category": "travel", "amount_minor": 12450, "description": "Train"})
        call("POST", f"/expense-claims/{claim['pid']}/submit")

step("change initiative")
ci = call("POST", "/change-initiatives", {"name": "Platform modernisation", "kind": "process_redesign", "description": "Fictitious"})
if ci and rp and sk_db:
    call("PUT", f"/change-initiatives/{ci['pid']}/role-impacts", {"role_profile_pid": rp["pid"], "impact": "reshaped", "timeframe": "within_1y"})
    call("PUT", f"/change-initiatives/{ci['pid']}/skill-shifts", {"skill_pid": sk_db["pid"], "direction": "rising"})

step("early warnings")
call("POST", f"/early-warnings/sweep?organization={ORG}")

print(f"\nDone. {len(failures)} step(s) failed (a failure is one line above).")
print("Organizations:", ORG, *region_orgs, "parent:", PARENT)
