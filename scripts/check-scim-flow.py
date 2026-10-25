#!/usr/bin/env python3
"""Check the SCIM 2.0 push end to end against a RUNNING demo service, with an independent SCIM server (WPM-T302).

The server in this file is written from RFC 7643 (schema) and RFC 7644 (protocol), not from the service's code: bearer authentication,
`POST /Users` (201 with an `id` and `meta`, 409 `uniqueness` on a duplicate `userName`), `GET /Users?filter=externalId eq "..."` (a
ListResponse), `PUT /Users/{id}` (a replacement; 404 for an unknown id). It also keeps a list of VIOLATIONS: an attribute outside the
core User and enterprise extension that WPM is meant to send, any contact detail (`emails`, `phoneNumbers`, `addresses`, `name`,
`costCenter`), an enterprise attribute without its schema URN, or a `manager.value` that is not an id this server issued.

    WPM_SCIM_TOKEN=some-token python3 scripts/check-scim-flow.py [http://localhost:5150] [--retry]

Start the service first with `WPM_REQUIRE_AUTH=0` against a database holding nothing real. This script runs `cargo run -q -- task
scim_push` in the service folder with `WPM_SCIM_BASE_URL` pointed at its own server, `WPM_SCIM_TOKEN` and `WPM_SCIM_ORGANIZATION` set to
a fresh organization reference, so it needs the same DATABASE_URL in its environment and only sees its own workers. `--retry` also waits
out the one-minute back-off after a 503 and checks the second try.
"""
import json, os, subprocess, sys, threading, time, urllib.parse, urllib.request, uuid
from http.server import BaseHTTPRequestHandler, HTTPServer

API = next((a for a in sys.argv[1:] if a.startswith("http")), "http://localhost:5150").rstrip("/") + "/api"
RETRY = "--retry" in sys.argv
TOKEN = os.environ.get("WPM_SCIM_TOKEN", "")
SERVICE_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "workforce-planning-management-service-with-rust")
PORT = 9100
CORE = "urn:ietf:params:scim:schemas:core:2.0:User"
ENT = "urn:ietf:params:scim:schemas:extension:enterprise:2.0:User"
ERR = "urn:ietf:params:scim:api:messages:2.0:Error"
ALLOWED = {"schemas", "externalId", "userName", "displayName", "title", "active", ENT}
FORBIDDEN = {"emails", "phoneNumbers", "addresses", "name", "costCenter", "ims", "photos", "x509Certificates", "password"}

USERS = {}      # id -> resource
REQUESTS = []   # (method, path) in order
VIOLATIONS = []
MODE = {"answer": None, "token": TOKEN}
FAILED = []
ORG = f"organization:{uuid.uuid4()}"


def check(ok, what):
    print(("  ok   " if ok else "  FAIL ") + what)
    if not ok:
        FAILED.append(what)


def scim_error(h, status, detail, scim_type=None):
    body = {"schemas": [ERR], "status": str(status), "detail": detail}
    if scim_type:
        body["scimType"] = scim_type
    h.send_response(status)
    h.send_header("Content-Type", "application/scim+json")
    h.end_headers()
    h.wfile.write(json.dumps(body).encode())


def audit(body):
    unknown = set(body) - ALLOWED
    if unknown:
        VIOLATIONS.append(f"attributes outside what WPM is meant to send: {sorted(unknown)}")
    bad = (set(body) | set(body.get(ENT, {}))) & FORBIDDEN
    if bad:
        VIOLATIONS.append(f"contact or cost detail sent: {sorted(bad)}")
    if CORE not in body.get("schemas", []):
        VIOLATIONS.append("the core User schema URN is missing")
    if ENT in body and ENT not in body.get("schemas", []):
        VIOLATIONS.append("an enterprise attribute without its schema URN")
    manager = body.get(ENT, {}).get("manager", {}).get("value")
    if manager and manager not in USERS:
        VIOLATIONS.append(f"manager.value {manager} is not an id this server issued")


class Scim(BaseHTTPRequestHandler):
    def log_message(self, *a):
        pass

    def _begin(self):
        REQUESTS.append((self.command, self.path.split("?")[0]))
        if self.headers.get("Authorization") != f"Bearer {MODE['token']}":
            scim_error(self, 401, "bad or missing bearer token")
            return False
        if MODE["answer"]:
            scim_error(self, MODE["answer"], "unavailable")
            return False
        return True

    def _body(self):
        raw = self.rfile.read(int(self.headers.get("content-length", 0)))
        ctype = (self.headers.get("content-type") or "").split(";")[0]
        if ctype not in ("application/scim+json", "application/json"):
            VIOLATIONS.append(f"content-type {ctype!r}")
        return json.loads(raw or b"{}")

    def do_POST(self):
        if not self._begin() or self.path.split("?")[0] != "/scim/v2/Users":
            return
        body = self._body()
        audit(body)
        if not body.get("userName"):
            return scim_error(self, 400, "userName is required", "invalidValue")
        if any(u["userName"].lower() == body["userName"].lower() for u in USERS.values()):
            return scim_error(self, 409, "userName is already in use", "uniqueness")
        uid = str(uuid.uuid4())
        now = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
        USERS[uid] = {**body, "id": uid, "meta": {"resourceType": "User", "created": now, "lastModified": now, "location": f"http://127.0.0.1:{PORT}/scim/v2/Users/{uid}"}}
        self.send_response(201)
        self.send_header("Content-Type", "application/scim+json")
        self.send_header("Location", USERS[uid]["meta"]["location"])
        self.end_headers()
        self.wfile.write(json.dumps(USERS[uid]).encode())

    def do_GET(self):
        if not self._begin():
            return
        url = urllib.parse.urlparse(self.path)
        if url.path != "/scim/v2/Users":
            return scim_error(self, 404, "not found")
        flt = urllib.parse.parse_qs(url.query).get("filter", [""])[0]
        found = []
        if flt.startswith('externalId eq "') and flt.endswith('"'):
            want = flt[len('externalId eq "'):-1]
            found = [u for u in USERS.values() if u.get("externalId") == want]
        elif flt:
            return scim_error(self, 400, "unsupported filter", "invalidFilter")
        self.send_response(200)
        self.send_header("Content-Type", "application/scim+json")
        self.end_headers()
        self.wfile.write(json.dumps({"schemas": ["urn:ietf:params:scim:api:messages:2.0:ListResponse"], "totalResults": len(found),
                                     "startIndex": 1, "itemsPerPage": len(found), "Resources": found}).encode())

    def do_PUT(self):
        if not self._begin():
            return
        uid = self.path.split("?")[0].rsplit("/", 1)[-1]
        if not self.path.startswith("/scim/v2/Users/") or uid not in USERS:
            return scim_error(self, 404, "no such user")
        body = self._body()
        audit(body)
        meta = {**USERS[uid]["meta"], "lastModified": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())}
        USERS[uid] = {**body, "id": uid, "meta": meta}
        self.send_response(200)
        self.send_header("Content-Type", "application/scim+json")
        self.end_headers()
        self.wfile.write(json.dumps(USERS[uid]).encode())


def call(method, path, body=None):
    req = urllib.request.Request(API + path, method=method, data=None if body is None else json.dumps(body).encode(),
                                 headers={"content-type": "application/json", "idempotency-key": uuid.uuid4().hex})
    with urllib.request.urlopen(req) as r:
        return json.loads(r.read() or b"{}")


def hire(number, title="Carer", manager=None):
    body = {"person_ref": "person:" + uuid.uuid4().hex, "organization_ref": ORG, "worker_number": number, "display_name": f"Scim Check {number}",
            "employment_type": "permanent", "department": "care", "job_title": title, "hired_on": "2020-01-05", "fte_percent": 100}
    if manager:
        body["manager_pid"] = manager
    return call("POST", "/workers", body)["pid"], body


def push():
    """Run the task; returns (summary dict, exit code, all text it printed)."""
    env = {**os.environ, "WPM_SCIM_BASE_URL": f"http://127.0.0.1:{PORT}/scim/v2", "WPM_SCIM_TOKEN": TOKEN, "WPM_SCIM_ORGANIZATION": ORG}
    out = subprocess.run(["cargo", "run", "-q", "--", "task", "scim_push"], cwd=SERVICE_DIR, env=env, capture_output=True, text=True, timeout=600)
    text = out.stdout + "\n" + out.stderr
    line = next((l for l in text.splitlines() if l.startswith("{")), "{}")
    return json.loads(line), out.returncode, text


def by_external(pid):
    return next((u for u in USERS.values() if u.get("externalId") == f"worker:{pid}"), None)


def main():
    if not TOKEN:
        sys.exit("set WPM_SCIM_TOKEN")
    threading.Thread(target=HTTPServer(("127.0.0.1", PORT), Scim).serve_forever, daemon=True).start()

    print("The first push (a manager and two reports):")
    boss, _ = hire(f"SC-{ORG[-6:]}-M")
    r1, r1_body = hire(f"SC-{ORG[-6:]}-1", manager=boss)
    r2, _ = hire(f"SC-{ORG[-6:]}-2", manager=boss)
    summary, code, _ = push()
    check(summary.get("created") == 3 and code == 0, f"three users created, exit 0 ({summary})")
    check([m for m, _ in REQUESTS[:3]] == ["POST", "POST", "POST"], "three POSTs")
    boss_user = by_external(boss)
    check(boss_user is not None and "manager" not in boss_user.get(ENT, {}), "the manager was sent first and has no manager of their own")
    check(by_external(r1)[ENT]["manager"]["value"] == boss_user["id"], "a report's manager is the id the provider gave the manager, not a WPM id")
    check(by_external(r1)["userName"] == r1_body["worker_number"] == by_external(r1)[ENT]["employeeNumber"], "userName and employeeNumber are the employee number")
    check(by_external(r1)["externalId"] == f"worker:{r1}" and by_external(r1)["active"] is True, "externalId is a worker URN and the user is active")
    check(not VIOLATIONS, f"no violation of the schema or the data rules ({VIOLATIONS})")

    print("A second push with nothing changed:")
    calls = len(REQUESTS)
    summary, _, _ = push()
    check(summary.get("unchanged") == 3 and len(REQUESTS) == calls, f"three unchanged and the provider was not called ({summary})")

    print("A change in one worker:")
    call("PUT", f"/workers/{r1}", {**r1_body, "job_title": "Senior carer"})
    summary, _, _ = push()
    check(summary.get("replaced") == 1 and REQUESTS[-1] == ("PUT", f"/scim/v2/Users/{by_external(r1)['id']}"), f"one PUT to that user's own id ({summary})")
    check(by_external(r1)["title"] == "Senior carer" and len(USERS) == 3, "the title changed and there is still one user each")

    print("A user the provider already holds (409):")
    r3, r3_body = hire(f"SC-{ORG[-6:]}-3", manager=boss)
    uid = str(uuid.uuid4())
    USERS[uid] = {"schemas": [CORE], "id": uid, "userName": r3_body["worker_number"], "externalId": f"worker:{r3}", "active": False,
                  "meta": {"resourceType": "User", "created": "2020-01-01T00:00:00Z", "lastModified": "2020-01-01T00:00:00Z", "location": ""}}
    calls = len(REQUESTS)
    summary, code, _ = push()
    sent = REQUESTS[calls:]
    check(sent == [("POST", "/scim/v2/Users"), ("GET", "/scim/v2/Users"), ("PUT", f"/scim/v2/Users/{uid}")], f"POST refused 409, found by externalId, then replaced ({sent})")
    check(len(USERS) == 4 and USERS[uid]["active"] is True and code == 0, "no duplicate was made and the held user was brought up to date")

    print("A provider that is down (503) and a wrong token (401):")
    call("PUT", f"/workers/{r3}", {**r3_body, "job_title": "Team lead"})
    MODE["answer"] = 503
    summary, code, text = push()
    check(summary.get("failed") == 1 and code != 0, f"one failure reported, non-zero exit ({summary})")
    calls = len(REQUESTS)
    summary, _, _ = push()
    check(summary.get("waiting") == 1 and len(REQUESTS) == calls, f"the next run waits out the back-off and sends nothing ({summary})")
    MODE["answer"] = None
    check(TOKEN not in text, "the token is not in anything the task printed")
    # A provider that rejects the bearer token: the push is refused, recorded as a failure (not retried at once), and nothing leaks.
    r4, _ = hire(f"SC-{ORG[-6:]}-4", manager=boss)
    MODE["token"] = "not-the-token"
    summary, code, text = push()
    check(summary.get("failed") == 1 and code != 0 and by_external(r4) is None, f"a rejected token fails the push and creates nothing ({summary})")
    check(TOKEN not in text and "not-the-token" not in text, "neither token appears in what the task printed")
    MODE["token"] = TOKEN
    if RETRY:
        print("The retry, after the one-minute back-off:")
        time.sleep(65)
        summary, code, _ = push()
        check(summary.get("replaced") == 1 and code == 0 and USERS[uid]["title"] == "Team lead", f"sent again and the provider has the new title ({summary})")
    check(not VIOLATIONS, f"still no violation ({VIOLATIONS})")

    print("FAILED: " + "; ".join(FAILED) if FAILED else "all checks passed")
    sys.exit(1 if FAILED else 0)


main()
