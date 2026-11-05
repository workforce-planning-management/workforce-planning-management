<script lang="ts">
  import type { Snippet } from "svelte";
  import { REPO_URL, isRtl } from "#lib/site.js";
  import { ui } from "#lib/i18n.js";

  // A landing page passes its `hero` and the rest as `html`, and the `Translations` line as children, between the two.
  let {
    html,
    path,
    locale = null,
    hero = "",
    landing = false,
    children,
  }: {
    html: string;
    path: string;
    locale?: string | null;
    hero?: string;
    landing?: boolean;
    children?: Snippet;
  } = $props();
  const s = $derived(ui(locale));

  // A brief's path has translated (non-ASCII) directory names: encode each segment for the GitHub URL.
  const sourceUrl = $derived(`${REPO_URL}/blob/main/${path.split("/").map(encodeURIComponent).join("/")}`);
</script>

<!-- The HTML is rendered at build time from the repository's own Markdown, with raw HTML escaped
     (#lib/server/content.ts), so nothing a reader supplies reaches it. -->
<article class="doc" class:landing>
  {#if hero}
    <div class="landing-hero">
      <!-- eslint-disable-next-line svelte/no-at-html-tags -->
      {@html hero}
      {@render children?.()}
    </div>
  {/if}
  <!-- eslint-disable-next-line svelte/no-at-html-tags -->
  {@html html}
</article>
{#if !landing}
  <p class="source" lang={s.locale} dir={isRtl(s.locale) ? "rtl" : "ltr"}>
    {s.source} <a href={sourceUrl}><code dir="ltr">{path}</code></a>
  </p>
{/if}

<style>
  .source {
    margin-block-start: 2.5rem;
    padding-block-start: 0.75rem;
    border-block-start: 1px solid var(--line);
    color: var(--muted);
    font-size: 0.875rem;
    overflow-wrap: anywhere;
  }
</style>
