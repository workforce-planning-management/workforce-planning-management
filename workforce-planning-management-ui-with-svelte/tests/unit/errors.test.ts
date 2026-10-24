// Unit tests: a failed call is described in the catalogue's words, by status (WPM-T261).

import { describe, expect, it } from "vitest";

import { ApiError } from "../../src/lib/api/client";
import { i18n } from "../../src/lib/i18n.svelte";
import { attempt, errorKey, errorText, failure } from "../../src/lib/write";

describe("errorKey", () => {
  it("maps each status the UI distinguishes", () => {
    const key = (status: number) => errorKey(new ApiError(status, null));
    expect(key(401)).toBe("errors.unauthorized");
    expect(key(403)).toBe("errors.forbidden");
    expect(key(404)).toBe("errors.notFound");
    expect(key(409)).toBe("errors.conflict");
    expect(key(422)).toBe("errors.invalid");
    expect(key(500)).toBe("errors.server");
    expect(key(503)).toBe("errors.server");
    expect(key(418)).toBe("errors.generic");
  });

  it("calls a network failure by its own name, and anything else generic", () => {
    expect(errorKey(new TypeError("Failed to fetch"))).toBe("errors.network");
    expect(errorKey(new Error("boom"))).toBe("errors.generic");
    expect(errorKey("a string")).toBe("errors.generic");
  });
});

describe("errorText", () => {
  it("adds the service's message only for a refusal that names a field or a state", () => {
    const invalid = new ApiError(422, { description: "end date is before start date" });
    expect(errorText(invalid)).toMatch(/: end date is before start date$/);
    const conflict = new ApiError(409, { description: "already approved" });
    expect(errorText(conflict)).toMatch(/: already approved$/);
    // A refusal for want of a role says which rule, after the words.
    const forbidden = new ApiError(403, { description: "only the line manager decides" });
    expect(errorText(forbidden)).toMatch(/ only the line manager decides$/);
    // No service message: just the words.
    expect(errorText(new ApiError(422, null))).not.toContain("API error");
  });

  it("never passes a server error's detail on", () => {
    const text = errorText(new ApiError(500, { description: "relation \"workers\" does not exist" }));
    expect(text).not.toContain("workers");
  });

  it("never shows a raw exception message", () => {
    expect(errorText(new Error("TypeError at line 3"))).not.toContain("line 3");
  });

  it("follows the locale", () => {
    const before = i18n.locale;
    i18n.set("fr-001");
    const french = errorText(new ApiError(403, null));
    i18n.set("en-001");
    const english = errorText(new ApiError(403, null));
    i18n.set(before);
    expect(french).not.toBe(english);
  });
});

describe("failure", () => {
  it("keeps the refusal for want of the right apart from the words", () => {
    expect(failure(new ApiError(403, null))).toEqual({ notAllowed: true, message: null });
    const other = failure(new ApiError(404, null));
    expect(other.notAllowed).toBe(false);
    expect(other.message).toBe(errorText(new ApiError(404, null)));
  });
});

describe("attempt", () => {
  it("is null when the write went through, and says why when it did not", async () => {
    expect(await attempt(async () => 1)).toBeNull();
    expect(await attempt(() => Promise.reject(new ApiError(403, null)))).toEqual({
      notAllowed: true,
      message: null,
    });
    const refused = await attempt(() => Promise.reject(new ApiError(422, { description: "no" })));
    expect(refused?.notAllowed).toBe(false);
    expect(refused?.message).toContain("no");
  });
});
