// A paged list for a page (WPM-T254, WPM-T478): the rows read so far, the "more" flag, and a guard so a slow answer to an
// older load never replaces a newer one. `load()` re-reads every page already asked for; `showMore()` adds the next.

import { latestOnly, messageOf } from "#lib/loader.svelte.js";
import {
  readNext,
  readPages,
  type PageReader,
  type Paged,
} from "#lib/paged.js";

export function createPagedLoader<T>(reader: () => PageReader<T>) {
  let items = $state<T[] | null>(null) as T[] | null;
  let more = $state(false);
  /** Where the next page starts; not shown, so not state. */
  let next: string | null = null;
  let loadingMore = $state(false);
  let error = $state<string | null>(null);
  let moreError = $state<string | null>(null);
  let pages = 1;
  const guard = latestOnly();

  return {
    get items() {
      return items;
    },
    get more() {
      return more;
    },
    get loadingMore() {
      return loadingMore;
    },
    get error() {
      return error;
    },
    /** A failed "show more", apart from a failed load: the rows already shown stay. */
    get moreError() {
      return moreError;
    },
    /** Read again from the first page, keeping as many pages as have been asked for. Call from an `$effect` that reads its inputs first. */
    async load() {
      const current = guard.begin();
      const read = reader();
      try {
        const got = await readPages(read, pages);
        if (!current()) return;
        items = got.items;
        next = got.next;
        more = got.more;
        error = null;
        moreError = null;
      } catch (cause) {
        if (current()) error = messageOf(cause);
      }
    },
    /** Start over from page one (a different filter is a different list). */
    reset() {
      pages = 1;
    },
    async showMore() {
      if (items === null) return;
      const current = guard.begin();
      loadingMore = true;
      moreError = null;
      try {
        const held: Paged<T> = { items, next, more };
        const got = await readNext(reader(), held);
        if (!current()) return;
        items = got.items;
        next = got.next;
        more = got.more;
        pages += 1;
      } catch (cause) {
        if (current()) moreError = messageOf(cause);
      } finally {
        if (current()) loadingMore = false;
      }
    },
  };
}
