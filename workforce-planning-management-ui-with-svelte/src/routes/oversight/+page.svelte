<!--
  Oversight (`/oversight`, WPM-R65 to R69): where an organization stands on its oversight scale, the
  exit criteria for that level with the evidence for each measure, and the monthly packs. A pack is
  frozen once and never changed; a mistaken one is withdrawn with a reason and stays on the record,
  with the mark shown beside it. The service never changes a level. Freezing and withdrawing are for an
  administrator of the organization; anyone else gets the reason, not a silent failure.
-->
<script lang="ts">
  import { ApiError } from "#lib/api/client.js";
  import {
    freezeOversightPack,
    getOversightPack,
    listOversightPacks,
    oversightStatus,
    withdrawOversightPack,
    type OversightPack,
    type OversightStatus,
    type PackSummary,
  } from "#lib/api/wpm.js";
  import CapacityHeatmap from "#lib/components/CapacityHeatmap.svelte";
  import OrgPicker from "#lib/components/OrgPicker.svelte";
  import OversightCriteria from "#lib/components/OversightCriteria.svelte";
  import { t, tf } from "#lib/i18n.svelte.js";

  let org = $state("");
  let status = $state<OversightStatus | null>(null);
  let packs = $state<PackSummary[] | null>(null);
  let open = $state<OversightPack | null>(null);
  let error = $state<string | null>(null);
  let notAllowed = $state(false);
  let reason = $state("");

  const message = (cause: unknown) => (cause instanceof Error ? cause.message : String(cause));

  async function load(chosen: string) {
    try {
      const [s, p] = await Promise.all([oversightStatus(chosen), listOversightPacks(chosen)]);
      if (chosen !== org) return;
      status = s;
      packs = p.packs;
      error = null;
    } catch (cause) {
      if (chosen !== org) return;
      status = null;
      packs = null;
      error = message(cause);
    }
  }

  $effect(() => {
    const chosen = org;
    open = null;
    if (chosen) void load(chosen);
  });

  /** Run a write; a refusal for want of the administrator role is said in words. */
  async function write(action: () => Promise<unknown>) {
    notAllowed = false;
    error = null;
    try {
      await action();
      await load(org);
    } catch (cause) {
      if (cause instanceof ApiError && cause.status === 403) notAllowed = true;
      else error = message(cause);
    }
  }

  async function toggle(pid: string) {
    if (open?.pid === pid) {
      open = null;
      return;
    }
    try {
      open = await getOversightPack(pid);
    } catch (cause) {
      error = message(cause);
    }
  }

  async function withdraw(pid: string) {
    await write(async () => {
      await withdrawOversightPack(pid, reason);
      reason = "";
      open = await getOversightPack(pid);
    });
  }
</script>

<svelte:head><title>{t("titles.oversight")}</title></svelte:head>

<h1>{t("ovs.title")}</h1>
<p class="muted">{t("ovs.hint")}</p>

<div class="form"><OrgPicker bind:value={org} /></div>

{#if error}
  <p class="error" data-testid="error">{t("common.error")}: {error}</p>
{/if}

{#if !org}
  <p class="muted">{t("common.noOrganization")}</p>
{:else if status === null || packs === null}
  {#if !error}<p>{t("common.loading")}</p>{/if}
{:else}
  <section class="panel" data-testid="position">
    <h2>{t("ovs.level")}</h2>
    {#if status.current}
      <p>
        <strong data-testid="level">{status.current.level}</strong>
        <span class="muted">· {tf("ovs.since", { date: status.current.effective_on })}</span>
      </p>
      <p class="muted">
        {#if status.current.deciding_body}{t("ovs.decidingBody")}: {status.current.deciding_body} ·
        {/if}{t("ovs.reason")}: {status.current.reason}
      </p>
    {:else}
      <p class="muted" data-testid="no-position">{t("ovs.none")}</p>
    {/if}
    <h2>{t("ovs.criteria")}</h2>
    <p class="muted">{tf("ovs.asOf", { month: status.as_of_month })}</p>
    <OversightCriteria criteria={status.criteria} />
    <details>
      <summary>{t("cap.basis")}</summary>
      <p class="muted">{status.basis}</p>
    </details>
  </section>

  <section class="panel" data-testid="packs">
    <h2>{t("ovs.packs")}</h2>
    <p class="muted">{t("ovs.packsHint")}</p>
    <button type="button" data-testid="freeze" onclick={() => write(() => freezeOversightPack(org))}>
      {t("ovs.freeze")}
    </button>
    {#if notAllowed}
      <p class="error" data-testid="not-allowed">{t("ovs.notAllowed")}</p>
    {/if}
    {#if packs.length === 0}
      <p class="muted" data-testid="no-packs">{t("ovs.noPacks")}</p>
    {:else}
      <ul class="packs">
        {#each packs as p (p.pid)}
          <li data-testid="pack">
            <button type="button" onclick={() => toggle(p.pid)}>
              {open?.pid === p.pid ? t("ovs.close") : t("ovs.open")}
            </button>
            {tf("ovs.frozen", { date: p.as_of, month: p.month.slice(0, 7) })}
            {#if p.withdrawn}
              <span class="chip danger" data-testid="withdrawn">
                {tf("ovs.withdrawn", { date: p.withdrawn.on, reason: p.withdrawn.reason })}
              </span>
            {/if}
          </li>
        {/each}
      </ul>
    {/if}
    {#if open}
      {@const pack = open.pack}
      <article class="frozen" data-testid="pack-detail">
        {#if open.withdrawn}
          <p class="chip danger">
            {tf("ovs.withdrawn", { date: open.withdrawn.on, reason: open.withdrawn.reason })}
          </p>
        {:else}
          <form
            onsubmit={(e) => {
              e.preventDefault();
              void withdraw(open!.pid);
            }}
          >
            <label>
              {t("ovs.withdrawReason")}
              <input bind:value={reason} required data-testid="withdraw-reason" />
            </label>
            <button type="submit" data-testid="withdraw">{t("ovs.withdraw")}</button>
          </form>
        {/if}
        <h3>{t("ovs.level")}</h3>
        <p>
          <strong>{pack.oversight.current?.level ?? t("ovs.none")}</strong>
          {#if pack.oversight.current}
            <span class="muted">· {tf("ovs.since", { date: pack.oversight.current.effective_on })}</span>
          {/if}
        </p>
        <OversightCriteria criteria={pack.oversight.criteria} />
        <h3>{t("ovs.packCapacity")}</h3>
        <CapacityHeatmap months={pack.capacity.months} pools={pack.capacity.pools} />
        <p>{tf("res.spof", { count: pack.key_person.single_points_of_failure })}</p>
        <p>
          {tf("res.seniorHours", { hours: pack.governance.senior_hours_per_month.toFixed(1) })}
          <span class="muted">
            · {tf("res.allHours", { hours: pack.governance.all_hours_per_month.toFixed(1) })}
          </span>
        </p>
        <p class="muted">{pack.basis}</p>
      </article>
    {/if}
  </section>
{/if}

<style>
  .packs {
    list-style: none;
    padding: 0;
  }
  .packs li {
    margin: 0.4rem 0;
  }
  .frozen {
    border-top: 1px solid var(--line);
    margin-top: 0.75rem;
    padding-top: 0.5rem;
  }
</style>
