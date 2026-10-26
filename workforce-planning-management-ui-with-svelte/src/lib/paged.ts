// Reading a list the service serves a page at a time (WPM-T254): 500 rows at most per call, `x-total-count` for the whole.
// Pure, so the rules are tested without a page: how many pages to re-read, and when there is more.

import type { Page } from "#lib/api/client.js";

/** What the service serves per call at most. */
export const PAGE_SIZE = 500;

/** The rows read so far, how many there are in all (`null` when not said), and whether another page is worth asking for. */
export type Paged<T> = { items: T[]; total: number | null; more: boolean };

/** Whether there is more after `items`: the total says so when known, else a full last page does. */
export function hasMore(
  held: number,
  total: number | null,
  lastPageLength: number,
  pageSize = PAGE_SIZE,
): boolean {
  return total !== null ? held < total : lastPageLength >= pageSize;
}

/** The type of a function that reads one page. */
export type PageReader<T> = (limit: number, offset: number) => Promise<Page<T>>;

/**
 * Read the first `pages` pages in turn (stopping early at a short page), so a reload after a change keeps every page
 * the person had asked for.
 */
export async function readPages<T>(
  read: PageReader<T>,
  pages = 1,
  pageSize = PAGE_SIZE,
): Promise<Paged<T>> {
  const items: T[] = [];
  let last = await read(pageSize, 0);
  items.push(...last.items);
  for (let n = 1; n < pages && last.items.length === pageSize; n++) {
    last = await read(pageSize, n * pageSize);
    items.push(...last.items);
  }
  return {
    items,
    total: last.total,
    more: hasMore(items.length, last.total, last.items.length, pageSize),
  };
}

/** Add the next page to what is held. */
export async function readNext<T>(
  read: PageReader<T>,
  held: T[],
  pageSize = PAGE_SIZE,
): Promise<Paged<T>> {
  const next = await read(pageSize, held.length);
  const items = [...held, ...next.items];
  return {
    items,
    total: next.total,
    more: hasMore(items.length, next.total, next.items.length, pageSize),
  };
}
