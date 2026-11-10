// Finding one's own 360° appraisals (WPM-T485): `/me` links straight to the person's own record at its appraisals, where requests
// to rate someone else and their own appraisals are. The API is stubbed per test; an unstubbed /api/proxy call 404s loudly.

import { expect, test } from "@playwright/test";
import { stubSignedIn, WORKER } from "./support";

test("My profile links to my own record's 360° appraisals", async ({
  page,
}) => {
  await stubSignedIn(page);
  await page.goto("/me");
  const link = page.getByTestId("my-record-link");
  await expect(link).toHaveText(
    "My record: leave, payslips, reviews and 360° appraisals",
  );
  await expect(link).toHaveAttribute(
    "href",
    `/en-001/workers/${WORKER.pid}#appraisals`,
  );
});
