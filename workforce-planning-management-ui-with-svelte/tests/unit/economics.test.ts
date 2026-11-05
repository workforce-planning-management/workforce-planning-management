// Every input the service can name as missing has words in the default locale (WPM-D172).

import { describe, expect, it } from "vitest";
import { inputNames } from "#lib/economics.js";

const TOKENS = [
  "benefit",
  "fee",
  "discount_rate",
  "one_currency",
  "pay",
  "pay_withheld",
  "on_cost",
  "full_time_annual_hours",
  "candidate_pay",
  "hours_cost",
  "learning_ramp_up_days",
  "recruitment_cost",
  "vacancy_days",
  "recruitment_ramp_up_days",
  "seat_cost",
  "seat_skills",
  "declared_skills",
];

describe("inputNames", () => {
  it("has words for every token", () => {
    for (const token of TOKENS) {
      expect(inputNames([token])).not.toContain("economics.input");
    }
  });

  it("joins several", () => {
    expect(inputNames(["fee", "pay"])).toBe("a course fee, pay on record");
  });
});
