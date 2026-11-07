// Reading a list the service serves a page at a time (WPM-T254, WPM-T478): 500 rows at most per call, and a signed
// cursor for the next page while more rows follow. There is no total.
// Pure, so the rules are tested without a page: how many pages to re-read, and when there is more.

import type { Page } from "#lib/api/client.js";

/** What the service serves per call at most. */
export const PAGE_SIZE = 500;

/** The rows read so far, and the cursor of the next page (`null` when there is no more). */
export type Paged<T> = { items: T[]; next: string | null; more: boolean };

/** The type of a function that reads one page: the first with `cursor` `null`, the rest with the cursor they were given. */
export type PageReader<T> = (
  limit: number,
  cursor: string | null,
) => Promise<Page<T>>;

const held = <T>(items: T[], next: string | null): Paged<T> => ({
  items,
  next,
  more: next !== null,
});

/**
 * Read the first `pages` pages in turn (stopping early at the last), so a reload after a change keeps every page the
 * person had asked for.
 */
export async function readPages<T>(
  read: PageReader<T>,
  pages = 1,
  pageSize = PAGE_SIZE,
): Promise<Paged<T>> {
  const items: T[] = [];
  let last = await read(pageSize, null);
  items.push(...last.items);
  for (let n = 1; n < pages && last.next !== null; n++) {
    last = await read(pageSize, last.next);
    items.push(...last.items);
  }
  return held(items, last.next);
}

/** Add the page `from` points to after what is held. */
export async function readNext<T>(
  read: PageReader<T>,
  from: Paged<T>,
  pageSize = PAGE_SIZE,
): Promise<Paged<T>> {
  if (from.next === null) return from;
  const next = await read(pageSize, from.next);
  return held([...from.items, ...next.items], next.next);
}
