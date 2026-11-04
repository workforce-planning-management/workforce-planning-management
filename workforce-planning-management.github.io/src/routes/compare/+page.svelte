<script lang="ts">
  import Doc from "#lib/Doc.svelte";
  import { ui } from "#lib/i18n.js";
  import type { PageProps } from "./$types";

  let { data }: PageProps = $props();
</script>

<svelte:head><title>{data.title} · {ui(null).brand}</title></svelte:head>

<Doc html={data.html} path={data.path} />
