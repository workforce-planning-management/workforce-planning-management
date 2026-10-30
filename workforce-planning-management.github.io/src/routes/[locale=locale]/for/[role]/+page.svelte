<script lang="ts">
  import Doc from "#lib/Doc.svelte";
  import { href } from "#lib/href.js";
  import type { PageProps } from "./$types";

  let { data }: PageProps = $props();
</script>

<svelte:head>
  <title>{data.title} · Workforce Planning Management</title>
  {#each Object.entries(data.alternates) as [l, route] (l)}
    <link rel="alternate" hreflang={l} href={href(route)} />
  {/each}
</svelte:head>

<p class="up"><a href={href(`/${data.locale}/for/`)} data-testid="up">{data.listTitle}</a></p>
<Doc html={data.html} path={data.path} />

<style>
  .up {
    margin-block: 0 1rem;
    font-size: 0.875rem;
  }
</style>
