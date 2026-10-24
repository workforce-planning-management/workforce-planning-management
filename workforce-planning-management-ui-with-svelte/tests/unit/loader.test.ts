// Unit tests: the loader keeps the newest answer, whatever order the responses arrive in
// (WPM-T260).

import { describe, expect, it } from "vitest";

import { errorText } from "../../src/lib/write";
import { createLoader, guardedLoad, latestOnly, messageOf, settleEach } from "../../src/lib/loader.svelte";

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
    expect(loader.error).toBe(errorText(new Error("down")));
    expect(loader.error).not.toContain("down");
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
    expect(out.payslips).toEqual({ value: null, error: errorText(new Error("payslips down")) });
  });

  it("describes a failure in the catalogue's words, never the raw exception", () => {
    expect(messageOf("plain")).toBe(errorText("plain"));
  });
});

describe("guardedLoad", () => {
  it("applies the newest answer and drops one that was overtaken", async () => {
    const calls = [deferred<string>(), deferred<string>()];
    let n = 0;
    const seen: string[] = [];
    const load = guardedLoad(
      () => calls[n++]!.promise,
      (answer) => seen.push(answer),
      () => seen.push("failed"),
    );
    const first = load();
    const second = load();
    calls[1]!.resolve("B");
    await second;
    calls[0]!.resolve("A");
    await first;
    expect(seen).toEqual(["B"]);
  });

  it("reports a failure of the newest, and drops the failure of an overtaken one", async () => {
    const calls = [deferred<string>(), deferred<string>()];
    let n = 0;
    const seen: unknown[] = [];
    const load = guardedLoad(
      () => calls[n++]!.promise,
      (answer) => seen.push(answer),
      (cause) => seen.push(messageOf(cause)),
    );
    const first = load();
    const second = load();
    calls[0]!.reject(new Error("late"));
    await first;
    expect(seen).toEqual([]);
    calls[1]!.reject(new Error("now"));
    await second;
    expect(seen).toHaveLength(1);
  });

  it("an error thrown while applying is a failure too", async () => {
    const seen: string[] = [];
    await guardedLoad(
      async () => 1,
      () => {
        throw new Error("apply broke");
      },
      () => seen.push("failed"),
    )();
    expect(seen).toEqual(["failed"]);
  });
});
