<script lang="ts">
  import { page } from "$app/state";
  import { href } from "#lib/href.js";
  import { formatInteger } from "#lib/locale-format.js";
  import { search, type SearchDoc } from "#lib/search.js";
  import { fill, ui } from "#lib/i18n.js";
  import { LOCALE_LABELS, isBriefLocale, isRtl } from "#lib/site.js";

  let { locale = null }: { locale?: string | null } = $props();
  const s = $derived(ui(locale));

  // The page is prerendered with no query, so the query is read in the browser, after hydration.
  let query = $state("");
  $effect(() => {
    query = page.url.searchParams.get("q")?.trim() ?? "";
  });

  let docs = $state<SearchDoc[] | null>(null);
  let failed = $state(false);
  $effect(() => {
    if (!query || docs || failed) return;
    fetch(href("/search-index.json"))
      .then((r) => (r.ok ? r.json() : Promise.reject(new Error(String(r.status)))))
      .then((d: SearchDoc[]) => (docs = d))
      .catch(() => (failed = true));
  });

  const hits = $derived(docs ? search(docs, query) : []);
  const languageOf = (locale: string) => (isBriefLocale(locale) ? LOCALE_LABELS[locale] : "English");
</script>

<svelte:head><title>{s.search.title} · {s.brand}</title></svelte:head>

<div lang={s.locale}>
<h1>{s.search.title}</h1>
<form role="search" action={href(locale ? `/${locale}/search/` : "/search/")} class="search-form">
  <label for="search-query">{s.search.label}</label>
  <input id="search-query" name="q" type="search" value={query} />
  <button type="submit">{s.search.submit}</button>
</form>
<noscript><p>{s.search.noscript}</p></noscript>

<div aria-live="polite">
  {#if failed}
    <p data-testid="search-failed">{s.search.failed}</p>
  {:else if query && docs}
    <p data-testid="search-count">
      {fill(s.search.count, { query, count: formatInteger(hits.length, s.locale) })}
    </p>
  {/if}
</div>
</div>

{#if hits.length > 0}
  <ol class="hits">
    {#each hits as hit (hit.route)}
      <li>
        <a href={href(hit.route)} lang={hit.locale} dir={isRtl(hit.locale) ? "rtl" : "ltr"}>{hit.title}</a>
        <span class="hit-language">{languageOf(hit.locale)}</span>
        <p lang={hit.locale} dir={isRtl(hit.locale) ? "rtl" : "ltr"}>{hit.snippet}</p>
      </li>
    {/each}
  </ol>
{/if}

<style>
  .search-form {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem;
    margin-block-end: 1rem;
  }
  .search-form input {
    flex: 1 1 12rem;
    min-height: 2.5rem;
    padding-inline: 0.5rem;
    font: inherit;
    color: var(--ink);
    background: var(--panel);
    border: 1px solid var(--muted);
    border-radius: var(--radius);
  }
  .search-form button {
    min-height: 2.5rem;
    padding-inline: 0.75rem;
    font: inherit;
    color: var(--accent-fg);
    background: var(--accent-fill);
    border: 0;
    border-radius: var(--radius);
    cursor: pointer;
  }
  .hits {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .hits li {
    padding-block: 0.75rem;
    border-block-end: 1px solid var(--line);
  }
  .hits a {
    font-weight: 600;
    text-decoration: underline;
  }
  .hit-language {
    margin-inline-start: 0.5rem;
    color: var(--muted);
    font-size: 0.875rem;
  }
  .hits p {
    margin-block: 0.25rem 0;
    overflow-wrap: anywhere;
  }
</style>
