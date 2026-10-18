// The header's picker bar (Lily 0.3): the link picker (the pages), the search picker (the directory) and
// that a hover tooltip never moves a button out from under the pointer.

import { expect, test, type Page } from "@playwright/test";

async function signIn(page: Page) {
  await page.context().addCookies([
    {
      name: "__Host-mxi_session",
      value: "pickers-spec-session",
      domain: "localhost",
      path: "/",
      httpOnly: true,
      secure: true,
      sameSite: "Lax",
    },
  ]);
  await page.route("**/api/proxy/**", (route) =>
    route.fulfill({ status: 404, body: "unstubbed: " + route.request().url() }),
  );
  await page.route("**/api/proxy/me/organizations**", (route) =>
    route.fulfill({ json: [] }),
  );
}

test("the link picker lists the pages in the reader's language, marks this one, and keeps the locale", async ({
  page,
}) => {
  await signIn(page);
  await page.route("**/api/proxy/directory**", (route) =>
    route.fulfill({ json: [] }),
  );
  await page.goto("/cy-001/directory");
  const button = page.locator("nav.top .link-picker-button");
  await button.click();
  const list = page.locator("nav.top .link-picker-list");
  await expect(list).toBeVisible();
  await expect(list.locator('a[aria-current="page"]')).toHaveCount(1);
  await expect(list.locator('a[aria-current="page"]')).toHaveAttribute(
    "href",
    "/cy-001/directory",
  );
  await list.locator('a[href="/cy-001/capacity"]').click();
  await expect(page).toHaveURL(/\/cy-001\/capacity$/);
});

test("the search picker sends the query to the directory, which opens with it filled in", async ({
  page,
}) => {
  await signIn(page);
  const asked: string[] = [];
  await page.route("**/api/proxy/directory**", (route) => {
    asked.push(new URL(route.request().url()).searchParams.get("q") ?? "");
    return route.fulfill({ json: [] });
  });
  await page.goto("/workers");
  await page.locator("nav.top .search-picker-button").click();
  await page
    .getByRole("searchbox")
    .or(page.getByLabel("Name, title or department"))
    .first()
    .fill("ana  lyst");
  await page.keyboard.press("Enter");
  await expect(page).toHaveURL(/\/en-001\/directory\?q=ana%20%20lyst$/);
  await expect(page.getByTestId("directory-search")).toHaveValue("ana  lyst");
  await expect.poll(() => asked.includes("ana  lyst")).toBe(true);
});

test("hovering a picker button does not move it (the tooltip is out of flow), so the click lands", async ({
  page,
}) => {
  await signIn(page);
  await page.goto("/workers");
  for (const name of [
    "locale",
    "theme",
    "text-size",
    "share",
    "link",
    "search",
  ]) {
    const button = page.locator(`nav.top .${name}-picker-button`);
    const before = await button.boundingBox();
    await button.hover();
    await expect(page.locator(`nav.top .${name}-picker-tooltip`)).toBeVisible();
    const after = await button.boundingBox();
    expect(after, name).toEqual(before);
    await button.click();
    await expect(button, name).toHaveAttribute("aria-expanded", "true");
    await page.keyboard.press("Escape");
    await page.mouse.click(5, 400);
  }
});
