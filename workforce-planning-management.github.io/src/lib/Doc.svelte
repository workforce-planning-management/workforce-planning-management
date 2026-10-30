<script lang="ts">
  import { REPO_URL } from "#lib/site.js";

  let { html, path }: { html: string; path: string } = $props();

  // A brief's path has translated (non-ASCII) directory names: encode each segment for the GitHub URL.
  const sourceUrl = $derived(`${REPO_URL}/blob/main/${path.split("/").map(encodeURIComponent).join("/")}`);
</script>

<!-- The HTML is rendered at build time from the repository's own Markdown, with raw HTML escaped
     (#lib/server/content.ts), so nothing a reader supplies reaches it. -->
<article class="doc">
  <!-- eslint-disable-next-line svelte/no-at-html-tags -->
  {@html html}
</article>
<p class="source" lang="en" dir="ltr">
  Source: <a href={sourceUrl}><code>{path}</code></a>
</p>

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
