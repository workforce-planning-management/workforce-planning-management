// The site's search: GitHub Pages has no server, so the prerendered page /search/ fetches the static index
// /search-index.json and matches in the browser. Pure and safe for the browser; the index itself is built only on the
// server side, during prerendering (`#lib/server/content.ts`).

/** One page in the index: its route, title and locale ("en" for the English documents) and its text. */
export type SearchDoc = { route: string; title: string; locale: string; text: string };

export type SearchHit = { route: string; title: string; locale: string; snippet: string };

const fold = (s: string) => s.normalize("NFKC").toLocaleLowerCase();

/** The words of a query, folded for matching; a query of spaces has none. */
export function searchTerms(query: string): string[] {
  return fold(query).split(/\s+/).filter(Boolean);
}

/** The text around a word's first match, with an ellipsis on each side that was cut. */
function snippetOf(text: string, at: number, radius = 90): string {
  const start = Math.max(0, at - radius);
  const end = Math.min(text.length, at + radius);
  return `${start > 0 ? "… " : ""}${text.slice(start, end).trim()}${end < text.length ? " …" : ""}`;
}

/**
 * The pages holding every word of the query (in the title or the text), a page with more of them in its title first,
 * then the page with the most matches. Ties keep the index's order, which is documents first.
 */
export function search(docs: readonly SearchDoc[], query: string): SearchHit[] {
  const words = searchTerms(query);
  if (words.length === 0) return [];
  const hits: { hit: SearchHit; inTitle: number; matches: number }[] = [];
  for (const doc of docs) {
    const title = fold(doc.title);
    const text = fold(doc.text);
    if (!words.every((w) => title.includes(w) || text.includes(w))) continue;
    const first = words.map((w) => text.indexOf(w)).find((i) => i >= 0) ?? 0;
    hits.push({
      // Folding can change a string's length, rarely; the snippet is then a little off, never wrong.
      hit: { route: doc.route, title: doc.title, locale: doc.locale, snippet: snippetOf(doc.text, first) },
      inTitle: words.filter((w) => title.includes(w)).length,
      matches: words.reduce((n, w) => n + text.split(w).length - 1, 0),
    });
  }
  return hits.sort((a, b) => b.inTitle - a.inTitle || b.matches - a.matches).map((h) => h.hit);
}
