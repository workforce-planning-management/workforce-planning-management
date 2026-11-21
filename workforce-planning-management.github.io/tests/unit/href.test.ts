import { describe, expect, it, vi } from "vitest";

// `$app/paths` exists only inside SvelteKit; stand in for it with a base path of "/base".
vi.mock("$app/paths", () => ({ resolve: (route: string) => `/base${route}` }));

import { href } from "../../src/lib/href.js";

describe("href", () => {
  it("passes a route through the base-path resolver", () => {
    expect(href("/install/")).toBe("/base/install/");
  });
});
