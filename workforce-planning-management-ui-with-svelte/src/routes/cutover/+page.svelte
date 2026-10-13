<!--
  Cutover readiness (`/cutover`, WPM-R70): for each planned go-live of an organization, the people
  side — trained staff, hypercare cover, key-person risk — each `ready`, `at risk` or `unknown`, and
  the worst of them. An unknown is never ready. The reasons are written by the service, in English.
  Whether to go live is a human decision outside this service. No person is named.
-->
<script lang="ts">
  import {
    cutoverReadiness,
    listCutovers,
    type Cutover,
    type CutoverReadiness,
    type Readiness,
  } from "#lib/api/wpm.js";
  import OrgPicker from "#lib/components/OrgPicker.svelte";
  import { t, tf } from "#lib/i18n.svelte.js";
  import { stateLabel } from "#lib/stateLabel.js";

  let org = $state("");
  let cutovers = $state<Cutover[] | null>(null);
  let readiness = $state<Record<string, CutoverReadiness>>({});
  let error = $state<string | null>(null);

  const message = (cause: unknown) => (cause instanceof Error ? cause.message : String(cause));

  $effect(() => {
    const chosen = org;
    if (!chosen) return;
    let current = true;
    cutovers = null;
    readiness = {};
    void (async () => {
      try {
        const list = await listCutovers(chosen);
        const loaded = await Promise.all(list.map((c) => cutoverReadiness(c.pid)));
        if (!current) return;
        cutovers = list;
        readiness = Object.fromEntries(loaded.map((r) => [r.cutover.pid, r]));
        error = null;
      } catch (cause) {
        if (current) error = message(cause);
      }
    })();
    return () => {
      current = false;
    };
  });

  const CHECKS = [
    ["trained_staff", "cut.trainedStaff"],
    ["hypercare_cover", "cut.hypercare"],
    ["key_person_risk", "cut.keyPerson"],
  ] as const;

  /** A mark as well as a colour, so the state is never colour alone. */
  const MARK: Record<Readiness, string> = { ready: "✓", at_risk: "✕", unknown: "?" };
</script>

<svelte:head><title>{t("titles.cutover")}</title></svelte:head>

<h1>{t("cut.title")}</h1>
<p class="muted">{t("cut.hint")}</p>

<div class="form"><OrgPicker bind:value={org} /></div>

{#if error}
  <p class="error" data-testid="error">{t("common.error")}: {error}</p>
{:else if !org}
  <p class="muted">{t("common.noOrganization")}</p>
{:else if cutovers === null}
  <p>{t("common.loading")}</p>
{:else if cutovers.length === 0}
  <p class="muted" data-testid="no-cutovers">{t("cut.none")}</p>
{:else}
  <p class="muted small">{t("cut.reasonsEnglish")}</p>
  {#each cutovers as c (c.pid)}
    {@const r = readiness[c.pid]}
    <section class="panel" data-testid="cutover">
      <h2>
        {c.name}
        {#if r}
          <span class="state {r.readiness}" data-testid="overall" data-status={r.readiness}>
            <span aria-hidden="true">{MARK[r.readiness]}</span>
            {t("cut.overall")}: {stateLabel(r.readiness)}
          </span>
        {/if}
      </h2>
      <p class="muted">
        {tf("cut.site", { site: c.site })} · {tf("cut.window", {
          start: c.window_start,
          end: c.window_end,
        })}
      </p>
      {#if r}
        <ul class="checks">
          {#each CHECKS as [key, label] (key)}
            {@const check = r.checks[key]}
            <li data-testid="check-{key}" data-status={check.status}>
              <span class="state {check.status}">
                <span aria-hidden="true">{MARK[check.status]}</span>
                {t(label)}: {stateLabel(check.status)}
              </span>
              {#if check.reasons.length === 0}
                <span class="muted">{t("cut.noConcerns")}</span>
              {:else}
                <ul>
                  {#each check.reasons as reason (reason)}<li>{reason}</li>{/each}
                </ul>
              {/if}
            </li>
          {/each}
        </ul>
        <details>
          <summary>{t("cap.basis")}</summary>
          <p class="muted">{r.basis}</p>
        </details>
      {/if}
    </section>
  {/each}
{/if}

<style>
  .small {
    font-size: 0.82rem;
  }
  .checks {
    list-style: none;
    padding: 0;
  }
  .checks > li {
    margin-bottom: 0.6rem;
  }
  .checks ul {
    margin: 0.2rem 0 0;
    padding-inline-start: 1.1rem;
  }
  .state {
    display: inline-block;
    border: 1px solid var(--line);
    border-radius: 0.4rem;
    padding: 0.05rem 0.5rem;
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--ink);
  }
  h2 .state {
    margin-inline-start: 0.5rem;
  }
  .ready {
    background: color-mix(in srgb, var(--state-available) 20%, var(--panel));
    border-color: var(--state-available);
  }
  .at_risk {
    background: color-mix(in srgb, var(--state-closed) 25%, var(--panel));
    border-color: var(--state-closed);
  }
  .unknown {
    border: 1px dashed var(--state-reserved);
    background: color-mix(in srgb, var(--state-reserved) 18%, var(--panel));
  }
</style>
