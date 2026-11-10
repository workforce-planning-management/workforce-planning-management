// What is required of me (WPM-R245, WPM-D192): the order, what needs attention, and which held qualification answers a
// requirement of the role (mirroring the service's match, never deciding the standing).

import { describe, expect, it } from "vitest";
import {
  answering,
  certificationNeedsAttention,
  learningNeedsAttention,
  matches,
  orderCertifications,
  orderLearning,
} from "../../src/lib/required";
import type {
  MyLearningRow,
  Qualification,
  QualificationGapRow,
} from "../../src/lib/api/types";

const row = (
  subject: string,
  status: MyLearningRow["status"],
): MyLearningRow => ({
  requirement_pid: subject,
  subject,
  basis: "mandatory",
  source: null,
  status,
  last_completed_on: null,
  expires_on: null,
});

const held = (over: Partial<Qualification>): Qualification => ({
  pid: over.title ?? "q",
  kind: "professional",
  title: "First aid",
  awarding_body: null,
  framework: null,
  level: null,
  awarded_on: null,
  expires_on: null,
  status: "verified",
  expired: false,
  verified_on: null,
  evidence_ref: null,
  ...over,
});

const req = (over: Partial<QualificationGapRow> = {}): QualificationGapRow => ({
  title: "First aid",
  kind: null,
  framework: null,
  min_level: null,
  standing: "met",
  ...over,
});

describe("the learning list", () => {
  it("puts the most urgent first, leaves out what does not apply, then sorts by subject", () => {
    const ordered = orderLearning([
      row("Zeta", "compliant"),
      row("Fire", "due_soon"),
      row("Alpha", "not_applicable"),
      row("Data", "expired"),
      row("Moving", "unknown"),
      row("Basic", "never_completed"),
      row("Aid", "expired"),
    ]);
    expect(ordered.map((r) => r.subject)).toEqual([
      "Aid",
      "Data",
      "Basic",
      "Fire",
      "Moving",
      "Zeta",
    ]);
  });

  it("asks the person to act on expired, never completed and due soon; unknown is HR's to fix", () => {
    expect(learningNeedsAttention("expired")).toBe(true);
    expect(learningNeedsAttention("never_completed")).toBe(true);
    expect(learningNeedsAttention("due_soon")).toBe(true);
    expect(learningNeedsAttention("unknown")).toBe(false);
    expect(learningNeedsAttention("compliant")).toBe(false);
  });
});

describe("the certifications list", () => {
  it("puts expired and missing before declared and met", () => {
    const ordered = orderCertifications([
      req({ title: "B", standing: "met" }),
      req({ title: "A", standing: "declared_only" }),
      req({ title: "C", standing: "missing" }),
      req({ title: "D", standing: "expired" }),
    ]);
    expect(ordered.map((r) => r.standing)).toEqual([
      "expired",
      "missing",
      "declared_only",
      "met",
    ]);
    expect(certificationNeedsAttention("met")).toBe(false);
    expect(certificationNeedsAttention("declared_only")).toBe(true);
  });

  it("matches a title ignoring case and space, a kind, and a minimum level on the same framework", () => {
    expect(matches(held({ title: "  first AID " }), req())).toBe(true);
    expect(matches(held({ title: "Fire safety" }), req())).toBe(false);
    expect(
      matches(held({ kind: "academic" }), req({ kind: "professional" })),
    ).toBe(false);
    const levelled = req({ framework: "RQF", min_level: 3 });
    expect(matches(held({ framework: "rqf", level: 3 }), levelled)).toBe(true);
    expect(matches(held({ framework: "RQF", level: 2 }), levelled)).toBe(false);
    expect(matches(held({ framework: "EQF", level: 5 }), levelled)).toBe(false);
    expect(matches(held({ framework: "RQF", level: null }), levelled)).toBe(
      false,
    );
  });

  it("answers with the verified one in date before a declared or an expired one, and the longest lasting among equals", () => {
    const today = "2026-11-10";
    const expired = held({ pid: "x", expires_on: "2026-01-01" });
    const declared = held({
      pid: "d",
      status: "declared",
      expires_on: "2027-01-01",
    });
    const verified = held({ pid: "v", expires_on: "2027-06-30" });
    const later = held({ pid: "l", expires_on: "2028-06-30" });
    const forever = held({ pid: "f", expires_on: null });
    expect(answering(req(), [expired], today)?.pid).toBe("x");
    expect(answering(req(), [expired, declared], today)?.pid).toBe("d");
    expect(answering(req(), [declared, verified, expired], today)?.pid).toBe(
      "v",
    );
    expect(answering(req(), [verified, later], today)?.pid).toBe("l");
    expect(answering(req(), [later, forever], today)?.pid).toBe("f");
    expect(answering(req(), [forever, later], today)?.pid).toBe("f");
    expect(answering(req(), [held({ title: "Other" })], today)).toBeNull();
  });

  it("counts the last day as in date", () => {
    expect(
      answering(
        req(),
        [
          held({ pid: "e", expires_on: "2026-11-10" }),
          held({ pid: "d", status: "declared" }),
        ],
        "2026-11-10",
      )?.pid,
    ).toBe("e");
  });
});
