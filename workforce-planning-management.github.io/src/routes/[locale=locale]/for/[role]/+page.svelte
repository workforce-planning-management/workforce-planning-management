<script lang="ts">
  import Doc from "#lib/Doc.svelte";
  import { ui } from "#lib/i18n.js";
  import { href } from "#lib/href.js";
  import type { PageProps } from "./$types";

  let { data }: PageProps = $props();
</script>

<svelte:head>
  <title>{data.title} · {ui(data.locale).brand}</title>
  {#each Object.entries(data.alternates) as [l, route] (l)}
    <link rel="alternate" hreflang={l} href={href(route)} />
  {/each}
</svelte:head>

<p class="up"><a href={href(`/${data.locale}/for/`)} data-testid="up">{data.listTitle}</a></p>
<Doc html={data.html} path={data.path} locale={data.locale} />

<style>
  .up {
    margin-block: 0 1rem;
    font-size: 0.875rem;
  }
</style>
