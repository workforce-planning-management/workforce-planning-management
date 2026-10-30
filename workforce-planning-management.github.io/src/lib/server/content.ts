// Server-only (prerender time): the repository documents and their Markdown, rendered to HTML.
import { Marked, type Tokens } from "marked";
import docs from "../../content/docs.json" with { type: "json" };
import { BRIEF_LOCALES, REPO_URL, briefRoute, isBriefLocale, routeOfRepoPath, type BriefLocale } from "#lib/site.js";
import { formatDate, formatDateRange, isCivilDate } from "#lib/locale-format.js";

type Entry = { locale: string; path: string; peer: string | null };
type Brief = Entry & { slug: string };

const FILES: Record<string, string> = docs.files;
const LISTS: Entry[] = docs.lists;
const BRIEFS: Brief[] = docs.briefs;

// The copy and the site must name the same locales: a locale added to `locales/` without a line in
// `BRIEF_LOCALES` (or the reverse) fails the build here instead of silently missing from the switcher.
{
  const copied = LISTS.map((l) => l.locale).join(",");
  if (copied !== BRIEF_LOCALES.join(","))
    throw new Error(`src/content/docs.json has locales ${copied}; src/lib/site.ts BRIEF_LOCALES has ${BRIEF_LOCALES.join(",")}`);
}

export function source(path: string): string {
  const text = FILES[path];
  if (text === undefined) throw new Error(`not in src/content/docs.json: ${path} (run pnpm run sync)`);
  return text;
}

/** The briefs of one locale, in directory order. */
export function briefsOf(locale: BriefLocale): Brief[] {
  return BRIEFS.filter((b) => b.locale === locale);
}

/** Every (locale, slug) pair, for prerendering. */
export function allBriefs(): { locale: BriefLocale; role: string }[] {
  return BRIEFS.filter((b) => isBriefLocale(b.locale)).map((b) => ({ locale: b.locale as BriefLocale, role: b.slug }));
}

export function hasBrief(locale: BriefLocale, slug: string): boolean {
  return BRIEFS.some((b) => b.locale === locale && b.slug === slug);
}

export function findBrief(locale: BriefLocale, slug: string): Brief | undefined {
  return BRIEFS.find((b) => b.locale === locale && b.slug === slug);
}

/**
 * The same page in every locale that has it, by `.locale-peer-id` (a brief's directory name is translated, its peer
 * id is not): locale → route. For a locale's list, `slug` is null.
 */
export function alternates(locale: BriefLocale, slug: string | null): Partial<Record<BriefLocale, string>> {
  const out: Partial<Record<BriefLocale, string>> = {};
  if (slug === null) {
    for (const l of LISTS) if (isBriefLocale(l.locale)) out[l.locale] = briefRoute(l.locale, null);
    return out;
  }
  const peer = findBrief(locale, slug)?.peer;
  for (const l of BRIEF_LOCALES) {
    const match = peer ? BRIEFS.find((b) => b.locale === l && b.peer === peer) : undefined;
    // A locale without this brief falls back to its list, so the switcher never leads to a missing page.
    out[l] = match ? briefRoute(l, match.slug) : briefRoute(l, null);
  }
  return out;
}

function escapeHtml(s: string): string {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}

/** A heading anchor in the way GitHub makes one (lower case; letters, marks, digits, `_` and `-`; spaces to `-`). */
export function slug(text: string): string {
  let h = text.trim().toLowerCase();
  h = h.replace(/`|\*|_(?=\W|$)/g, "");
  h = h.replace(/\[([^\]]*)\]\([^)]*\)/g, "$1");
  h = h.replace(/[^\p{L}\p{N}\p{M}_\- ]/gu, "");
  return h.replace(/ /g, "-");
}

/** Join a relative link onto the directory of `from` (a repository path); null if it climbs out of the repository. */
export function joinRepoPath(from: string, href: string): string | null {
  const parts = href.startsWith("/") ? [] : from.split("/").slice(0, -1);
  for (const seg of href.split("/")) {
    if (seg === "" || seg === ".") continue;
    if (seg === "..") {
      if (parts.length === 0) return null;
      parts.pop();
    } else parts.push(seg);
  }
  return parts.join("/");
}

/**
 * Where a link in the document at `from` should go on the site. A link to another document the site shows goes to
 * its page; any other repository path goes to the file on GitHub; external links and in-page anchors are kept.
 * `route` turns a site route ("/install/") into an href that honours the base path.
 */
export function rewriteHref(from: string, href: string, route: (r: string) => string): string {
  if (href.startsWith("#") || /^[a-zA-Z][a-zA-Z0-9+.-]*:/.test(href) || href.startsWith("//")) return href;
  const [rawPath = "", frag] = href.split("#", 2);
  let decoded = rawPath;
  try {
    decoded = decodeURIComponent(rawPath);
  } catch {
    // A malformed escape is kept as written.
  }
  const target = rawPath === "" ? from : joinRepoPath(from, decoded);
  const hash = frag ? `#${frag}` : "";
  if (target === null) return `${REPO_URL}${hash}`;
  const page = routeOfRepoPath(target, hasBrief);
  if (page) return route(page) + hash;
  const kind = /\.[^/]+$/.test(target) ? "blob" : "tree";
  return `${REPO_URL}/${kind}/main/${target.split("/").map(encodeURIComponent).join("/")}${hash}`;
}

// An ISO date in running text ("Written on 2026-10-11 from …"), or a range of two ("2026-10-01 to 2026-10-30",
// "2026-10-01–2026-10-30"). Dates inside code are not text tokens and are left alone; so is a date that is part of a
// timestamp, an id or a longer number.
const DATE = String.raw`\d{4}-\d{2}-\d{2}`;
const DATES = new RegExp(
  String.raw`(?<![\w-])(${DATE})(?:(\s*[–—]\s*|\s+to\s+)(${DATE}))?(?![\w:-])`,
  "g",
);

/**
 * Plain text, escaped, with each ISO date shown in the page's locale the way the WPM UI shows one (`formatDate`,
 * a range as `formatDateRange`), inside `<time datetime>` so the ISO value is still there for a machine.
 */
export function localiseDates(text: string, locale: string): string {
  let out = "";
  let at = 0;
  for (const m of text.matchAll(DATES)) {
    const [whole, start = "", , end] = m;
    const i = m.index ?? 0;
    if (!isCivilDate(start) || (end !== undefined && !isCivilDate(end))) continue;
    out += escapeHtml(text.slice(at, i));
    if (end !== undefined && start <= end) {
      out += `<time datetime="${start}/${end}">${escapeHtml(formatDateRange(start, end, locale))}</time>`;
    } else if (end !== undefined) {
      // A "range" that runs backwards is two dates, each shown as one, with the text between them as written.
      const between = whole.slice(start.length, whole.length - end.length);
      out += `<time datetime="${start}">${escapeHtml(formatDate(start, locale))}</time>${escapeHtml(between)}`;
      out += `<time datetime="${end}">${escapeHtml(formatDate(end, locale))}</time>`;
    } else {
      out += `<time datetime="${start}">${escapeHtml(formatDate(start, locale))}</time>`;
    }
    at = i + whole.length;
  }
  return out + escapeHtml(text.slice(at));
}

export type Rendered = { title: string; html: string };

/**
 * Render one repository document. Raw HTML in the Markdown is shown as text, never passed through. `locale` is the
 * page's locale: the dates in its text are shown in that locale's form.
 */
export function render(path: string, route: (r: string) => string, locale: string): Rendered {
  let text = source(path);
  text = text.replace(/^---\n[\s\S]*?\n---\n/, "");
  const seen = new Map<string, number>();
  let title = "";
  const md = new Marked({
    gfm: true,
    walkTokens(token) {
      if (token.type === "link") (token as Tokens.Link).href = rewriteHref(path, (token as Tokens.Link).href, route);
    },
    renderer: {
      html(token) {
        return escapeHtml(token.text);
      },
      text(token) {
        if ("tokens" in token && token.tokens) return this.parser.parseInline(token.tokens);
        // Always escaped here: marked marks text inside raw HTML as `escaped` and would pass it through as HTML.
        return localiseDates(token.text, locale);
      },
      heading(token) {
        const inner = this.parser.parseInline(token.tokens);
        const base = slug(token.text);
        const n = seen.get(base) ?? 0;
        seen.set(base, n + 1);
        const id = n === 0 ? base : `${base}-${n}`;
        if (token.depth === 1 && !title) title = token.text.replace(/[`*_]/g, "");
        return `<h${token.depth} id="${escapeHtml(id)}">${inner}</h${token.depth}>\n`;
      },
      checkbox(token) {
        // A task list's box is a record, not a control: disabled, with a name so it is not an unlabelled input (axe).
        const state = token.checked ? "checked " : "";
        return `<input type="checkbox" disabled ${state}aria-label="${token.checked ? "Done" : "Not done"}"> `;
      },
      code(token) {
        // tabindex: a block that scrolls sideways must be reachable from the keyboard (axe scrollable-region-focusable).
        const lang = token.lang ? ` class="language-${escapeHtml(token.lang.split(/\s/)[0] ?? "")}"` : "";
        return `<pre tabindex="0"><code${lang}>${escapeHtml(token.text)}</code></pre>\n`;
      },
      table(token) {
        // A wide table scrolls inside its own region instead of widening the page (phones, long paths).
        const head = token.header.map((c) => `<th>${this.parser.parseInline(c.tokens)}</th>`).join("");
        const body = token.rows
          .map((row) => `<tr>${row.map((c) => `<td>${this.parser.parseInline(c.tokens)}</td>`).join("")}</tr>`)
          .join("\n");
        return `<div class="table-scroll" tabindex="0"><table><thead><tr>${head}</tr></thead><tbody>${body}</tbody></table></div>\n`;
      },
    },
  });
  const html = md.parse(text, { async: false });
  return { title: title || path, html };
}

/** How much the site shows: the documents, the role briefs and their locales. */
export function counts(): { documents: number; briefs: number; locales: number } {
  return { documents: Object.keys(FILES).length, briefs: BRIEFS.length, locales: LISTS.length };
}

/** A document's title: its first level-one heading, or its path. */
export function titleOf(path: string): string {
  return /^# (.+)$/m.exec(source(path))?.[1]?.replace(/[`*_]/g, "").trim() ?? path;
}
