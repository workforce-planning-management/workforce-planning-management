<script lang="ts">
  import SearchPage from "#lib/SearchPage.svelte";
  import type { PageProps } from "./$types";

  let { data }: PageProps = $props();
</script>

<SearchPage locale={data.locale} />
