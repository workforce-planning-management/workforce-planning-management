<script lang="ts">
  import { page } from "$app/state";
  import { benchmarkComparison, listBenchmarks, money } from "#lib/api/wpm.js";
  import { i18n, t, l } from "#lib/i18n.svelte.js";
  import type { Benchmark, ComparisonRow } from "#lib/api/types.js";

  // No switcher: every organization this person can read gets its own
  // comparison section below, fetched in parallel — not a single
  // guessed org. `scope` (not `organizations`) since it's expanded
  // through org confederation: a membership in a parent confederation
  // reads as every descendant too. The benchmark bands themselves are
  // a shared reference table, not organization-scoped, so they stay a
  // single list.
  const organizationRefs = $derived((page.data.scope ?? []) as string[]);

  let benchmarks = $state<Benchmark[] | null>(null);
  let rowsByOrganization = $state<Record<string, ComparisonRow[]> | null>(null);
  let error = $state<string | null>(null);

  $effect(() => {
    const refs = organizationRefs;
    void (async () => {
      try {
        const [bands, entries] = await Promise.all([
          listBenchmarks(),
          Promise.all(
            refs.map(async (ref) => {
              const comparison = await benchmarkComparison(ref);
              return [ref, comparison.rows] as const;
            }),
          ),
        ]);
        benchmarks = bands;
        rowsByOrganization = Object.fromEntries(entries);
      } catch (cause) {
        error = cause instanceof Error ? cause.message : String(cause);
      }
    })();
  });
</script>

<h1>{t("nav.benchmarks")}</h1>

{#if error}
  <p class="error" data-testid="error">{t("common.error")}: {error}</p>
{:else if benchmarks === null || rowsByOrganization === null}
  <p>{t("common.loading")}</p>
{:else}
  <table data-testid="bands">
    <thead>
      <tr><th>{t("common.jobTitle")}</th><th>min</th><th>median</th><th>max</th></tr>
    </thead>
    <tbody>
      {#each benchmarks as band (band.pid)}
        <tr>
          <td>{band.job_title}</td>
          <td>{money(band.min_minor, band.currency, i18n.locale)}</td>
          <td>{money(band.median_minor, band.currency, i18n.locale)}</td>
          <td>{money(band.max_minor, band.currency, i18n.locale)}</td>
        </tr>
      {/each}
    </tbody>
  </table>

  <h2>{t("bench.flag")}</h2>
  {#if organizationRefs.length === 0}
    <p class="muted">{t("org.noOrganizations")}</p>
  {:else}
    {#each organizationRefs as ref (ref)}
      <section class="benchmark-org-section">
        <h3><code>{ref}</code></h3>
        <table data-testid="comparison">
          <tbody>
            {#each rowsByOrganization[ref] ?? [] as row (row.worker_pid)}
              <tr>
                <td><a href={l(`/workers/${row.worker_pid}`)}>{row.worker_pid.slice(0, 8)}</a></td>
                <td>{row.job_title}</td>
                <td>{row.department}</td>
                <td>
                  {#if row.flag}
                    <span class={`chip flag-${row.flag}`}>{row.flag}</span>
                  {:else}
                    <span class="muted">—</span>
                  {/if}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </section>
    {/each}
  {/if}
{/if}

<style>
  .flag-below_min {
    color: var(--red-day, #c0392b);
  }
  .flag-above_max {
    color: var(--state-reserved, #b57e10);
  }
  .benchmark-org-section + .benchmark-org-section {
    margin-top: 1rem;
  }
  .benchmark-org-section h3 {
    font-size: 0.9rem;
    margin: 0.75rem 0 0.5rem;
  }
  .benchmark-org-section h3 code {
    color: var(--muted);
    font-weight: 400;
  }
</style>
