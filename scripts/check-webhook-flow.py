#!/usr/bin/env python3
"""Check the outbound webhook flow end to end against a RUNNING demo service, with an independent receiver (WPM-T301).

The receiver in this file knows only what spec/integrations.md says about the signature (`X-WPM-Signature: t=<unix>,v1=<hex HMAC-SHA256
of "<t>.<body>">`) and the payload (`{id, kind, subject, occurred_at}`, plus `data` for a hire or an ending since WPM-T372); it shares
no code with the service. It subscribes to `worker_activated`, which has no `data`, and checks that a delivery is signed and fresh, that
the body holds nothing but those four fields, that a 5xx is retried with a back-off and a 4xx is not repeated.

    WPM_WEBHOOK_SECRET_E2E=some-secret python3 scripts/check-webhook-flow.py [http://localhost:5150] [--retry]

Start the service first with the SAME variable and `WPM_REQUIRE_AUTH=0` (a demo posture, a database holding nothing real); this script
creates one subscription and a few workers there. It runs `cargo run -q -- task webhook_deliver` in the service folder to send what is
queued, so it needs the same DATABASE_URL in its environment. `--retry` also waits out the one-minute back-off and checks the second try.
"""
import hashlib, hmac, json, os, subprocess, sys, threading, time, urllib.request, uuid
from http.server import BaseHTTPRequestHandler, HTTPServer

API = next((a for a in sys.argv[1:] if a.startswith("http")), "http://localhost:5150").rstrip("/") + "/api"
RETRY = "--retry" in sys.argv
SECRET = os.environ.get("WPM_WEBHOOK_SECRET_E2E", "")
SERVICE_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "workforce-planning-management-service-with-rust")
PORT = 9099
MODE = {"answer": 200}
SEEN = []
FAILED = []


def check(ok, what):
    print(("  ok   " if ok else "  FAIL ") + what)
    if not ok:
        FAILED.append(what)


class Receiver(BaseHTTPRequestHandler):
    def log_message(self, *a):
        pass

    def do_POST(self):
        body = self.rfile.read(int(self.headers.get("content-length", 0)))
        parts = dict(p.split("=", 1) for p in self.headers.get("X-WPM-Signature", "").split(",") if "=" in p)
        t = parts.get("t", "0")
        mac = hmac.new(SECRET.encode(), f"{t}.".encode() + body, hashlib.sha256).hexdigest()
        SEEN.append({"valid": hmac.compare_digest(mac, parts.get("v1", "")), "fresh": abs(time.time() - int(t or 0)) < 300,
                     "body": json.loads(body or b"{}"), "type": self.headers.get("content-type")})
        self.send_response(MODE["answer"])
        self.end_headers()
        self.wfile.write(b"{}")


def call(method, path, body=None):
    req = urllib.request.Request(API + path, method=method, data=None if body is None else json.dumps(body).encode(),
                                 headers={"content-type": "application/json", "idempotency-key": uuid.uuid4().hex})
    with urllib.request.urlopen(req) as r:
        return json.loads(r.read() or b"{}")


def new_event(number):
    w = call("POST", "/workers", {"person_ref": "person:" + uuid.uuid4().hex, "organization_ref": "organization:99999999-9999-4999-8999-999999999999",
                                  "worker_number": number, "display_name": "Webhook Check Person", "employment_type": "permanent",
                                  "department": "care", "job_title": "Carer", "hired_on": "2020-01-05", "fte_percent": 100})
    call("POST", f"/workers/{w['pid']}/status", {"to": "active"})
    return w["pid"]


def deliver():
    out = subprocess.run(["cargo", "run", "-q", "--", "task", "webhook_deliver"], cwd=SERVICE_DIR, capture_output=True, text=True, timeout=600)
    # The one-line JSON summary: the task may print it on either stream.
    # The task sends everything that is due, so its summary counts leftovers from other runs; the log of this run's own subscription is the
    # evidence the checks use.
    line = next((l for l in (out.stdout + "\n" + out.stderr).splitlines() if l.startswith("{")), "{}")
    return json.loads(line)


def log(sub):
    return sorted(call("GET", f"/webhook-subscriptions/{sub}/deliveries"), key=lambda d: d["created_at"])


def main():
    if not SECRET:
        sys.exit("set WPM_WEBHOOK_SECRET_E2E (the service must have the same value)")
    server = HTTPServer(("127.0.0.1", PORT), Receiver)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    run = uuid.uuid4().hex[:8]
    sub = call("POST", "/webhook-subscriptions", {"name": f"flow check {run}", "url": f"http://127.0.0.1:{PORT}/hook",
                                                   "kinds": ["worker_activated"], "secret_ref": "WPM_WEBHOOK_SECRET_E2E", "active": True})["pid"]

    print("A delivery that is answered 200:")
    pid = new_event(f"WF-{run}-A")
    check(log(sub)[0]["status"] == "pending", "queued as pending, with no body column in the log")
    deliver()
    got = SEEN[-1]
    check(got["valid"] and got["fresh"], "the signature verifies against the shared secret and the timestamp is fresh")
    check(set(got["body"]) == {"id", "kind", "subject", "occurred_at"}, "the body holds exactly id, kind, subject, occurred_at")
    check(got["body"]["subject"] == f"worker:{pid}" and "Webhook Check Person" not in json.dumps(got["body"]), "the subject is a URN and no name or number is in the body")
    check(log(sub)[0]["status"] == "delivered" and log(sub)[0]["last_status"] == 200, "the log says delivered, status 200")

    print("A delivery that is answered 503:")
    MODE["answer"] = 503
    new_event(f"WF-{run}-B")
    deliver()
    calls = len(SEEN)
    deliver()
    check(len(SEEN) == calls, "a second run straight away sends nothing to the receiver: the back-off has not passed")
    b = log(sub)[1]
    check(b["status"] == "pending" and b["attempts"] == 1 and b["last_status"] == 503, "the log says pending, attempt 1, status 503")

    print("A delivery that is answered 400:")
    MODE["answer"] = 400
    new_event(f"WF-{run}-C")
    deliver()
    calls = len(SEEN)
    deliver()
    check(len(SEEN) == calls and log(sub)[2]["status"] == "failed", "it is not repeated, and the log says failed")

    if RETRY:
        print("The retry, after the one-minute back-off:")
        MODE["answer"] = 200
        wait = max(0.0, time.mktime(time.strptime(log(sub)[1]["next_attempt_at"][:19], "%Y-%m-%dT%H:%M:%S")) - time.mktime(time.gmtime()) + 2)
        time.sleep(wait)
        deliver()
        check(log(sub)[1]["status"] == "delivered" and log(sub)[1]["attempts"] == 2, "the log says delivered on attempt 2")

    call("DELETE", f"/webhook-subscriptions/{sub}")
    print("FAILED: " + "; ".join(FAILED) if FAILED else "all checks passed")
    sys.exit(1 if FAILED else 0)


main()
