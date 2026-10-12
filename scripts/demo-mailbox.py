#!/usr/bin/env python3
"""Read (or send to) the demo mail server's catch-all inbox (WPM-D81). Standard library only.

  scripts/demo-mailbox.py                   list the messages in the inbox, newest first
  scripts/demo-mailbox.py --show ID         print one message
  scripts/demo-mailbox.py --send            send a test message to anyone@wpm.test, then list

The demo server (`deploy/stalwart`, started by `podman compose up`) accepts mail for wpm.test on
127.0.0.1:1025 with no login and delivers every message to inbox@wpm.test. It never sends anything outside.
Override the defaults with WPM_MAIL_SMTP (host:port), WPM_MAIL_JMAP (base URL), WPM_MAIL_USER and WPM_MAIL_PASSWORD.
"""
import base64
import json
import os
import smtplib
import sys
import urllib.request
from email.message import EmailMessage

SMTP = os.environ.get("WPM_MAIL_SMTP", "127.0.0.1:1025")
JMAP = os.environ.get("WPM_MAIL_JMAP", "http://127.0.0.1:8025").rstrip("/")
USER = os.environ.get("WPM_MAIL_USER", "inbox@wpm.test")
PASSWORD = os.environ.get("WPM_MAIL_PASSWORD", "wpm-demo-mail")
AUTH = "Basic " + base64.b64encode(f"{USER}:{PASSWORD}".encode()).decode()
MAIL = "urn:ietf:params:jmap:mail"


def call(path, body=None):
    request = urllib.request.Request(
        JMAP + path,
        data=None if body is None else json.dumps(body).encode(),
        headers={"Authorization": AUTH, "Content-Type": "application/json"},
    )
    with urllib.request.urlopen(request, timeout=15) as response:
        return json.load(response)


def account():
    return call("/jmap/session")["primaryAccounts"][MAIL]


def listing(account_id):
    answer = call(
        "/jmap",
        {
            "using": ["urn:ietf:params:jmap:core", MAIL],
            "methodCalls": [
                ["Email/query", {"accountId": account_id, "sort": [{"property": "receivedAt", "isAscending": False}], "limit": 50}, "q"],
                [
                    "Email/get",
                    {
                        "accountId": account_id,
                        "#ids": {"resultOf": "q", "name": "Email/query", "path": "/ids"},
                        "properties": ["subject", "from", "to", "receivedAt"],
                    },
                    "g",
                ],
            ],
        },
    )
    return answer["methodResponses"][1][1]["list"]


def show(account_id, message_id):
    answer = call(
        "/jmap",
        {
            "using": ["urn:ietf:params:jmap:core", MAIL],
            "methodCalls": [
                [
                    "Email/get",
                    {
                        "accountId": account_id,
                        "ids": [message_id],
                        "properties": ["subject", "from", "to", "receivedAt", "textBody", "bodyValues"],
                        "fetchTextBodyValues": True,
                    },
                    "g",
                ]
            ],
        },
    )
    found = answer["methodResponses"][0][1]["list"]
    if not found:
        sys.exit(f"no message {message_id}")
    message = found[0]
    print(f"Subject: {message['subject']}\nFrom: {message['from']}\nTo: {message['to']}\nReceived: {message['receivedAt']}\n")
    for part in message.get("textBody", []):
        print(message["bodyValues"][part["partId"]]["value"])


def send():
    host, _, port = SMTP.partition(":")
    message = EmailMessage()
    message["From"] = "noreply@wpm.test"
    message["To"] = "anyone@wpm.test"
    message["Subject"] = "A test message from scripts/demo-mailbox.py"
    message.set_content("If you can read this in the inbox, the demo mail server works.")
    with smtplib.SMTP(host, int(port or 25), timeout=15) as server:
        server.send_message(message)
    print(f"sent to anyone@wpm.test through {SMTP}")


def main():
    args = sys.argv[1:]
    if "--send" in args:
        send()
    account_id = account()
    if "--show" in args:
        show(account_id, args[args.index("--show") + 1])
        return
    messages = listing(account_id)
    print(f"{len(messages)} message(s) in {USER}")
    for m in messages:
        who = ", ".join(a["email"] for a in (m.get("from") or []))
        print(f"{m['id']}  {m['receivedAt']}  {who}  {m['subject']}")


if __name__ == "__main__":
    main()
