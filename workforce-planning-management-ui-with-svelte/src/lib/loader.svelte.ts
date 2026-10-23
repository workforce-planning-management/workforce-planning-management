// One way to load: the data, the error and the loading flag in one place, and a guard so a slow
// response to an earlier request can never overwrite the answer to a later one (WPM-T260).
// Navigating from one worker to another, or changing a filter, starts a new load; whatever the
// first one brings back is dropped.

/** Hands out a ticket per load; only the newest ticket is still current. */
export function latestOnly() {
  let newest = 0;
  return {
    /** Start a load. The returned function says whether this load is still the newest. */
    begin(): () => boolean {
      const mine = ++newest;
      return () => mine === newest;
    },
  };
}

/** The message a person sees for a failed load. */
export function messageOf(cause: unknown): string {
  return cause instanceof Error ? cause.message : String(cause);
}

export interface Loader<T> {
  /** The last answer that is still current; `null` until one arrives. */
  readonly data: T | null;
  /** The message of the newest load, when it failed. */
  readonly error: string | null;
  /** True while the newest load is in flight. */
  readonly loading: boolean;
  /** Fetch again. A load that is overtaken by a newer one is discarded, answer or error. */
  load(): Promise<void>;
  /** Replace the data (an optimistic edit, or a write that returns the new row). */
  set(value: T): void;
}

/**
 * Own the loading, error and data of one fetch. Call `load()` from an `$effect` that reads
 * what the fetch depends on, and from the handlers that change it.
 *
 * The fetch runs untracked by the effect only if it reads its inputs before the first `await`;
 * read them in the effect body, as in `$effect(() => { void pid; void loader.load(); })`.
 */
export function createLoader<T>(fetcher: () => Promise<T>): Loader<T> {
  let data = $state<T | null>(null) as T | null;
  let error = $state<string | null>(null);
  let loading = $state(false);
  const guard = latestOnly();

  return {
    get data() {
      return data;
    },
    get error() {
      return error;
    },
    get loading() {
      return loading;
    },
    set(value: T) {
      data = value;
    },
    async load() {
      const current = guard.begin();
      loading = true;
      try {
        const answer = await fetcher();
        if (!current()) return;
        data = answer;
        error = null;
      } catch (cause) {
        if (!current()) return;
        error = messageOf(cause);
      } finally {
        if (current()) loading = false;
      }
    },
  };
}

export type Settled<T> = { value: T; error: null } | { value: null; error: string };

/**
 * Run independent loads together and keep each one's outcome apart: a failed secondary panel
 * reports its own error and the others still show.
 */
export async function settleEach<T extends Record<string, () => Promise<unknown>>>(
  fetchers: T,
): Promise<{ [K in keyof T]: Settled<Awaited<ReturnType<T[K]>>> }> {
  const out = {} as { [K in keyof T]: Settled<Awaited<ReturnType<T[K]>>> };
  await Promise.all(
    (Object.keys(fetchers) as (keyof T)[]).map(async (key) => {
      try {
        out[key] = { value: (await fetchers[key]!()) as never, error: null };
      } catch (cause) {
        out[key] = { value: null, error: messageOf(cause) };
      }
    }),
  );
  return out;
}
