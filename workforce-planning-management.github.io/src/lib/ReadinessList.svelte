<script lang="ts">
  import { Slider } from "@lilydesignsystem/svelte-headless";
  import { fill, ui } from "#lib/i18n.js";
  import { formatNumber } from "#lib/locale-format.js";
  import { isRtl } from "#lib/site.js";
  import type { ReadinessItem } from "#lib/site.js";

  // One readiness list: each item's text, and a Lily slider from 0 to 100 per cent that starts at the author's
  // estimate. A reader may move it to try their own view; nothing is saved or sent, and a reload restores the estimate.
  let { items, locale }: { items: ReadinessItem[]; locale: string } = $props();
  const s = $derived(ui(locale));

  // The reader's values, by item, starting at the author's estimates.
  const own = $state<Record<number, number>>({});
  const valueOf = (i: number) => own[i] ?? items[i]?.estimate ?? 0;
  const percent = (n: number) => formatNumber(n / 100, locale, { style: "percent", maximumFractionDigits: 0 });
</script>

<ul class="readiness">
  {#each items as item, i (i)}
    {@const id = `readiness-${item.title.replace(/[^\p{L}\p{N}]+/gu, "-").toLowerCase()}-${i}`}
    <li>
      <div class="estimate">
        <Slider
          label={fill(s.readiness.label, { name: item.title })}
          bind:value={() => valueOf(i), (v) => (own[i] = Number(v))}
          min={0}
          max={100}
          step={5}
          id={id}
          aria-valuetext={percent(valueOf(i))}
        />
        <output for={id}>
          <strong>{percent(valueOf(i))}</strong>
          {#if valueOf(i) !== item.estimate}
            <span class="author" lang={s.locale} dir={isRtl(s.locale) ? "rtl" : "ltr"}>({fill(s.readiness.author, { value: percent(item.estimate) })})</span>
          {/if}
        </output>
      </div>
      <!-- Rendered at build time from the repository's Markdown, raw HTML escaped (#lib/server/content.ts). -->
      <!-- eslint-disable-next-line svelte/no-at-html-tags -->
      <div class="text">{@html item.html}</div>
    </li>
  {/each}
</ul>

<style>
  .readiness {
    list-style: none;
    padding-inline-start: 0;
    display: grid;
    gap: 1rem;
  }
  .readiness li {
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: 0.5rem;
    padding: 0.875rem 1rem;
  }
  .estimate {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem 0.75rem;
    margin-block-end: 0.5rem;
  }
  .estimate :global(input.slider) {
    flex: 1 1 12rem;
    min-inline-size: 0;
    accent-color: var(--accent-fill);
  }
  output {
    min-inline-size: 4ch;
    font-variant-numeric: tabular-nums;
  }
  .author {
    color: var(--muted);
    font-size: 0.875rem;
  }
  .text :global(p:first-child) {
    margin-block-start: 0;
  }
</style>
