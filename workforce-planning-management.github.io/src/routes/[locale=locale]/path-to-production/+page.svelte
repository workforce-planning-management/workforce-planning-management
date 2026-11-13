<script lang="ts">
  import Doc from "#lib/Doc.svelte";
  import ReadinessList from "#lib/ReadinessList.svelte";
  import { ui } from "#lib/i18n.js";
  import { isRtl } from "#lib/site.js";
  import type { PageProps } from "./$types";

  let { data }: PageProps = $props();
  const s = $derived(ui(data.locale));
  const firstList = $derived(data.segments.findIndex((x) => x.kind === "readiness"));
</script>

<svelte:head><title>{data.title} · {s.brand}</title></svelte:head>

<Doc html="" path={data.path} locale={data.locale}>
  {#snippet body()}
    {#each data.segments as segment, i (i)}
      {#if segment.kind === "html"}
        <!-- Rendered at build time from the repository's Markdown, raw HTML escaped (#lib/server/content.ts). -->
        <!-- eslint-disable-next-line svelte/no-at-html-tags -->
        {@html segment.html}
      {:else}
        {#if i === firstList}<p class="readiness-note" lang={s.locale} dir={isRtl(s.locale) ? "rtl" : "ltr"}>{s.readiness.note}</p>{/if}
        <ReadinessList items={segment.items} locale={data.locale} />
      {/if}
    {/each}
  {/snippet}
</Doc>

<style>
  .readiness-note {
    color: var(--muted);
    font-size: 0.9375rem;
  }
</style>
