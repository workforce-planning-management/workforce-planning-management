// The role picker asks for a framework's role profiles once. It once asked again after every answer when
// the answer was empty (it read the list's length in its own effect), which on a service with no profiles
// for the framework sent hundreds of requests a second.

import { render, waitFor } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";

import RolePicker from "../../src/lib/components/RolePicker.svelte";

afterEach(() => vi.unstubAllGlobals());

describe("RolePicker", () => {
  it("asks for the profiles once, even when there are none", async () => {
    let asked = 0;
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => {
        asked += 1;
        return new Response("[]", {
          status: 200,
          headers: { "content-type": "application/json" },
        });
      }),
    );
    render(RolePicker, { framework: "uk-gdad-pcf", onpick: () => {} });
    await waitFor(() => expect(asked).toBeGreaterThan(0));
    // Time enough for a loop to show: dozens of answers would have come back by now.
    await new Promise((resolve) => setTimeout(resolve, 300));
    expect(asked).toBe(1);
  });
});
