// Unit tests: the loader keeps the newest answer, whatever order the responses arrive in
// (WPM-T260).

import { describe, expect, it } from "vitest";

import { createLoader, latestOnly, messageOf, settleEach } from "../../src/lib/loader.svelte";

/** A promise settled by hand, so a test chooses the order responses arrive in. */
function deferred<T>() {
  let resolve!: (v: T) => void;
  let reject!: (e: unknown) => void;
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

describe("latestOnly", () => {
  it("makes only the newest ticket current", () => {
    const guard = latestOnly();
    const first = guard.begin();
    expect(first()).toBe(true);
    const second = guard.begin();
    expect(first()).toBe(false);
    expect(second()).toBe(true);
  });
});

describe("createLoader", () => {
  it("starts empty, loads, and clears an earlier error", async () => {
    let fail = true;
    const loader = createLoader(async () => {
      if (fail) throw new Error("down");
      return 7;
    });
    expect(loader.data).toBeNull();
    expect(loader.error).toBeNull();
    expect(loader.loading).toBe(false);
    await loader.load();
    expect(loader.error).toBe("down");
    expect(loader.data).toBeNull();
    fail = false;
    await loader.load();
    expect(loader.data).toBe(7);
    expect(loader.error).toBeNull();
    expect(loader.loading).toBe(false);
  });

  it("drops a slow earlier response that arrives after a later one", async () => {
    const calls = [deferred<string>(), deferred<string>()];
    let n = 0;
    const loader = createLoader(() => calls[n++].promise);
    const first = loader.load();
    const second = loader.load();
    calls[1].resolve("worker B");
    await second;
    expect(loader.data).toBe("worker B");
    calls[0].resolve("worker A");
    await first;
    expect(loader.data).toBe("worker B");
    expect(loader.loading).toBe(false);
  });

  it("drops a slow earlier error too, and stays loading until the newest answers", async () => {
    const calls = [deferred<string>(), deferred<string>()];
    let n = 0;
    const loader = createLoader(() => calls[n++].promise);
    const first = loader.load();
    const second = loader.load();
    calls[0].reject(new Error("late failure"));
    await first;
    expect(loader.error).toBeNull();
    expect(loader.loading).toBe(true);
    calls[1].resolve("ok");
    await second;
    expect(loader.data).toBe("ok");
    expect(loader.loading).toBe(false);
  });

  it("set replaces the data", async () => {
    const loader = createLoader(async () => 1);
    loader.set(5);
    expect(loader.data).toBe(5);
  });
});

describe("settleEach", () => {
  it("keeps each outcome apart, so one failure hides only its own panel", async () => {
    const out = await settleEach({
      leave: async () => [1, 2],
      payslips: async () => {
        throw new Error("payslips down");
      },
    });
    expect(out.leave).toEqual({ value: [1, 2], error: null });
    expect(out.payslips).toEqual({ value: null, error: "payslips down" });
  });

  it("says what a non-Error rejection was", () => {
    expect(messageOf("plain")).toBe("plain");
  });
});
