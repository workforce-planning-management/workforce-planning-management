<!--
  Oversight (`/oversight`, WPM-R65 to R69): where an organization stands on its oversight scale, the
  exit criteria for that level with the evidence for each measure, and the monthly packs. A pack is
  frozen once and never changed; a mistaken one is withdrawn with a reason and stays on the record,
  with the mark shown beside it. The service never changes a level. Freezing and withdrawing are for an
  administrator of the organization; anyone else gets the reason, not a silent failure.
-->
<script lang="ts">
  import { formatDate, formatDecimal, formatMonth } from "#lib/locale-format.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import {
    createOversightCriterion,
    createOversightLevel,
    createOversightMeasure,
    freezeOversightPack,
    getOversightPack,
    listOversightDerivations,
    listOversightPacks,
    oversightStatus,
    putOversightReading,
    recordOversightPosition,
    withdrawOversightPack,
    type OversightPack,
    type OversightStatus,
    type PackSummary,
  } from "#lib/api/wpm.js";
  import CapacityHeatmap from "#lib/components/CapacityHeatmap.svelte";
  import OrgPicker from "#lib/components/OrgPicker.svelte";
  import OversightCriteria from "#lib/components/OversightCriteria.svelte";
  import WriteResult from "#lib/components/WriteResult.svelte";
  import { t, tf, tv } from "#lib/i18n.svelte.js";
  import { attempt, errorText, type Failure } from "#lib/write.js";

  let org = $state("");
  let status = $state<OversightStatus | null>(null);
  let packs = $state<PackSummary[] | null>(null);
  let open = $state<OversightPack | null>(null);
  let error = $state<string | null>(null);
  let notAllowed = $state(false);
  let reason = $state("");
  let failed = $state<Failure | null>(null);
  let derivations = $state<Array<{ key: string; unit: string }>>([]);

  // The scale, position and evidence forms.
  let levelOrdinal = $state(1);
  let levelName = $state("");
  let posLevel = $state("");
  let posOn = $state("");
  let posBody = $state("");
  let posReason = $state("");
  let critLevel = $state("");
  let critName = $state("");
  let measCriterion = $state("");
  let measName = $state("");
  let measOrigin = $state<"reported" | "derived">("reported");
  let measDerivation = $state("");
  let measUnit = $state("");
  let measThreshold = $state(0);
  let measDirection = $state<"at_most" | "at_least">("at_most");
  let measSustain = $state(3);
  let readMeasure = $state("");
  let readMonth = $state("");
  let readValue = $state<number | null>(null);
  let readEvidence = $state("");
  let readSource = $state("");

  /** A write on the scale or the evidence, then read the status again. */
  async function change(action: () => Promise<unknown>) {
    failed = await attempt(action);
    if (!failed) await load(org);
  }

  $effect(() => {
    let current = true;
    void listOversightDerivations()
      .then((d) => current && (derivations = d.derivations))
      .catch(() => current && (derivations = []));
    return () => {
      current = false;
    };
  });

  const reportedMeasures = $derived(
    (status?.criteria ?? []).flatMap((c) =>
      c.measures.filter((m) => m.origin === "reported").map((m) => ({ ...m, criterion: c.criterion })),
    ),
  );

  const message = (cause: unknown) => (errorText(cause));

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
    const failed = await attempt(action);
    if (!failed) await load(org);
    else if (failed.notAllowed) notAllowed = true;
    else error = failed.message;
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
  <ErrorNote testid="error" prefixed message={error} />
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
        <span class="muted">· {tf("ovs.since", { date: formatDate(status.current.effective_on) })}</span>
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
      <p class="muted">{t("ovs.basisText")}</p>
      <p class="muted small">{t("common.fromService")}: {status.basis}</p>
    </details>
  </section>

  <section class="panel" data-testid="packs">
    <h2>{t("ovs.packs")}</h2>
    <p class="muted">{t("ovs.packsHint")}</p>
    <button type="button" data-testid="freeze" onclick={() => write(() => freezeOversightPack(org))}>
      {t("ovs.freeze")}
    </button>
    {#if notAllowed}
      <ErrorNote testid="not-allowed" message={t("ovs.notAllowed")} />
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
            {tf("ovs.frozen", { date: formatDate(p.as_of), month: formatMonth(p.month, "long") })}
            {#if p.withdrawn}
              <span class="chip danger" data-testid="withdrawn">
                {tf("ovs.withdrawn", { date: formatDate(p.withdrawn.on), reason: p.withdrawn.reason })}
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
            {tf("ovs.withdrawn", { date: formatDate(open.withdrawn.on), reason: open.withdrawn.reason })}
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
            <span class="muted">· {tf("ovs.since", { date: formatDate(pack.oversight.current.effective_on) })}</span>
          {/if}
        </p>
        <OversightCriteria criteria={pack.oversight.criteria} />
        <h3>{t("ovs.packCapacity")}</h3>
        <CapacityHeatmap months={pack.capacity.months} pools={pack.capacity.pools} />
        <p>{tf("res.spof", { count: pack.key_person.single_points_of_failure })}</p>
        <p>
          {tf("res.seniorHours", { hours: formatDecimal(pack.governance.senior_hours_per_month, 1) })}
          <span class="muted">
            · {tf("res.allHours", { hours: formatDecimal(pack.governance.all_hours_per_month, 1) })}
          </span>
        </p>
      </article>
    {/if}
  </section>

  <section class="panel" data-testid="manage-oversight">
    <h2>{t("ovs.manage")}</h2>
    <h3>{t("ovs.addLevel")}</h3>
    <form
      class="form"
      onsubmit={(e) => {
        e.preventDefault();
        void change(() =>
          createOversightLevel({ organization_ref: org, ordinal: levelOrdinal, name: levelName }),
        );
      }}
    >
      <label>{t("ovs.ordinal")} <input type="number" min="1" max="20" bind:value={levelOrdinal} required data-testid="level-ordinal" /></label>
      <label>{t("ovs.levelName")} <input bind:value={levelName} required data-testid="level-name" /></label>
      <button type="submit" data-testid="add-level">{t("common.add")}</button>
    </form>

    <h3>{t("ovs.recordPosition")}</h3>
    <form
      class="form"
      onsubmit={(e) => {
        e.preventDefault();
        void change(() =>
          recordOversightPosition({
            organization_ref: org,
            level_pid: posLevel,
            effective_on: posOn,
            deciding_body: posBody,
            reason: posReason,
          }),
        );
      }}
    >
      <label>
        {t("ovs.chooseLevel")}
        <select bind:value={posLevel} required data-testid="pos-level">
          <option value="" disabled></option>
          {#each status.scale as l (l.pid)}<option value={l.pid}>{l.ordinal}. {l.name}</option>{/each}
        </select>
      </label>
      <label>{t("ovs.effectiveOn")} <input type="date" bind:value={posOn} required data-testid="pos-on" /></label>
      <label>{t("ovs.decidingBody")} <input bind:value={posBody} required data-testid="pos-body" /></label>
      <label>{t("ovs.reason")} <input bind:value={posReason} required data-testid="pos-reason" /></label>
      <button type="submit" data-testid="record-position">{t("common.save")}</button>
    </form>

    <h3>{t("ovs.addCriterion")}</h3>
    <form
      class="form"
      onsubmit={(e) => {
        e.preventDefault();
        void change(() => createOversightCriterion({ level_pid: critLevel, name: critName }));
      }}
    >
      <label>
        {t("ovs.chooseLevel")}
        <select bind:value={critLevel} required data-testid="crit-level">
          <option value="" disabled></option>
          {#each status.scale as l (l.pid)}<option value={l.pid}>{l.ordinal}. {l.name}</option>{/each}
        </select>
      </label>
      <label>{t("ovs.criterionName")} <input bind:value={critName} required data-testid="crit-name" /></label>
      <button type="submit" data-testid="add-criterion">{t("common.add")}</button>
    </form>

    <h3>{t("ovs.addMeasure")}</h3>
    <form
      class="form"
      onsubmit={(e) => {
        e.preventDefault();
        void change(() =>
          createOversightMeasure({
            criterion_pid: measCriterion,
            name: measName,
            origin: measOrigin,
            ...(measOrigin === "derived" ? { derivation: measDerivation } : {}),
            unit: measUnit,
            threshold: measThreshold,
            direction: measDirection,
            sustain: measSustain,
          }),
        );
      }}
    >
      <label>
        {t("ovs.criterionName")}
        <select bind:value={measCriterion} required data-testid="meas-criterion">
          <option value="" disabled></option>
          {#each status.criteria as c (c.criterion_pid)}<option value={c.criterion_pid}>{c.criterion}</option>{/each}
        </select>
      </label>
      <label>{t("ovs.measure")} <input bind:value={measName} required data-testid="meas-name" /></label>
      <label>
        {t("ovs.origin")}
        <select bind:value={measOrigin} data-testid="meas-origin">
          <option value="reported">{tv("reported")}</option>
          <option value="derived">{tv("derived")}</option>
        </select>
      </label>
      {#if measOrigin === "derived"}
        <label>
          {t("ovs.derivation")}
          <select bind:value={measDerivation} required>
            <option value="" disabled></option>
            {#each derivations as d (d.key)}<option value={d.key}>{tv(d.key)}</option>{/each}
          </select>
        </label>
      {/if}
      <label>{t("ovs.unit")} <input bind:value={measUnit} required data-testid="meas-unit" /></label>
      <label>{t("ovs.threshold")} <input type="number" bind:value={measThreshold} required data-testid="meas-threshold" /></label>
      <label>
        {t("ovs.direction")}
        <select bind:value={measDirection}>
          <option value="at_most">{tv("at_most")}</option>
          <option value="at_least">{tv("at_least")}</option>
        </select>
      </label>
      <label>{t("ovs.sustain")} <input type="number" min="1" bind:value={measSustain} required /></label>
      <button type="submit" data-testid="add-measure">{t("common.add")}</button>
    </form>

    <h3>{t("ovs.enterReading")}</h3>
    <p class="muted small">{t("ovs.readingHint")}</p>
    <form
      class="form"
      onsubmit={(e) => {
        e.preventDefault();
        void change(() =>
          putOversightReading(readMeasure, {
            period_on: readMonth,
            value: readValue,
            ...(readValue === null ? {} : { evidence: readEvidence, source_document: readSource }),
          }),
        );
      }}
    >
      <label>
        {t("ovs.forMeasure")}
        <select bind:value={readMeasure} required data-testid="read-measure">
          <option value="" disabled></option>
          {#each reportedMeasures as m (m.measure_pid)}<option value={m.measure_pid}>{m.criterion}: {m.name}</option>{/each}
        </select>
      </label>
      <label>{t("common.month")} <input type="date" bind:value={readMonth} required data-testid="read-month" /></label>
      <label>{t("ovs.readingValue")} <input type="number" bind:value={readValue} data-testid="read-value" /></label>
      <label>{t("ovs.evidence")} <input bind:value={readEvidence} required={readValue !== null} data-testid="read-evidence" /></label>
      <label>{t("ovs.sourceDocument")} <input bind:value={readSource} required={readValue !== null} data-testid="read-source" /></label>
      <button type="submit" data-testid="save-reading">{t("common.save")}</button>
    </form>
    <WriteResult {failed} />
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
  .small {
    font-size: 0.82rem;
  }
  .frozen {
    border-top: 1px solid var(--line);
    margin-top: 0.75rem;
    padding-top: 0.5rem;
  }
</style>
