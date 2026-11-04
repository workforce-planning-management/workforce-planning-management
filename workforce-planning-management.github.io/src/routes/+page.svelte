<script lang="ts">
  import Doc from "#lib/Doc.svelte";
  import Translations from "#lib/Translations.svelte";
  import { ui } from "#lib/i18n.js";
  import type { PageProps } from "./$types";

  let { data }: PageProps = $props();
</script>

<svelte:head><title>{data.title}</title></svelte:head>

<Doc landing hero={data.hero} html={data.rest} path={data.path}>
  <Translations label={ui(null).translations} />
</Doc>
