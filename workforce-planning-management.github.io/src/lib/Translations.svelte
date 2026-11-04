<script lang="ts">
  import { href } from "#lib/href.js";
  import { HOME_LOCALES, LOCALE_LABELS, isRtl } from "#lib/site.js";

  let { label }: { label: string } = $props();

  // The `-001` home of each translated language, each named in itself.
  const worldLocales = HOME_LOCALES.filter((l) => l.endsWith("-001"));
</script>

<p class="translations" data-testid="translations">
  {label}
  {#each worldLocales as l, i (l)}{#if i > 0},{" "}{/if}<a
      href={href(`/${l}/`)}
      lang={l}
      dir={isRtl(l) ? "rtl" : "ltr"}>{LOCALE_LABELS[l]}</a
    >{/each}
</p>
