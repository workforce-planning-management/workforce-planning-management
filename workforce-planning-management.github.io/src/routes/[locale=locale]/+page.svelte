<script lang="ts">
  import Doc from "#lib/Doc.svelte";
  import Translations from "#lib/Translations.svelte";
  import { ui } from "#lib/i18n.js";
  import type { PageProps } from "./$types";

  let { data }: PageProps = $props();
  const s = $derived(ui(data.locale));
</script>

<svelte:head><title>{s.brand}</title></svelte:head>

<Doc landing hero={data.hero} html={data.rest} path={data.path} locale={data.locale}>
  <Translations label={s.translations} />
</Doc>
