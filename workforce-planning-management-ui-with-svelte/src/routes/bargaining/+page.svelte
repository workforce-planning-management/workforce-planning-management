<!--
  Recognised bodies and bargaining units (`/bargaining`): the unions, works councils and staff forums the
  organization recognises, each with a contact ROLE, and the units they cover as rules over department,
  location, employment type and job title. How many a unit covers is a count from the floor; under it,
  nothing. Nothing here says who belongs to a body or represents one (WPM-D68). HR only.
-->
<script lang="ts">
  import { formatDate } from "#lib/locale-format.js";
  import { page } from "$app/state";
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import {
    BODY_BASES,
    BODY_KINDS,
    addBargainingUnit,
    createRecognisedBody,
    addFacilityTime,
    listBargainingUnits,
    listFacilityTime,
    listRecognisedBodies,
    readInformationPack,
  } from "#lib/api/wpm.js";
  import type {
    BargainingUnit,
    FacilityPeriod,
    InformationPack,
    PackCell,
    PackGroup,
    RecognisedBody,
    UnitCoverage,
  } from "#lib/api/wpm.js";
  import { t, tf, tv } from "#lib/i18n.svelte.js";

  const organizations = $derived((page.data.scope ?? []) as string[]);
  let organization = $state("");
  $effect(() => {
    if (!organization && organizations.length > 0) organization = organizations[0] ?? "";
  });

  let bodies = $state<RecognisedBody[] | null>(null);
  let error = $state<string | null>(null);
  const load = guardedLoad(
    () => listRecognisedBodies(organization),
    (found) => {
      bodies = found.bodies;
      error = null;
    },
    (cause) => {
      error = errorText(cause);
    },
  );
  $effect(() => {
    if (organization) void load();
  });

  let name = $state("");
  let kind = $state<string>(BODY_KINDS[0]);
  let basis = $state<string>(BODY_BASES[0]);
  let recognisedOn = $state("");
  let contactRole = $state("");
  async function addBody() {
    error = null;
    try {
      await createRecognisedBody({
        organization_ref: organization,
        name,
        kind,
        basis,
        recognised_on: recognisedOn,
        contact_role: contactRole,
      });
      name = "";
      contactRole = "";
      await load();
    } catch (cause) {
      error = errorText(cause);
    }
  }

  let open = $state<string | null>(null);
  let units = $state<BargainingUnit[]>([]);
  let floor = $state(0);
  let unitError = $state<string | null>(null);
  let periods = $state<FacilityPeriod[]>([]);
  let totalMinutes = $state(0);
  let pack = $state<{ unit: string; data: InformationPack } | null>(null);
  async function showUnits(pid: string) {
    open = pid;
    unitError = null;
    pack = null;
    try {
      const found = await listBargainingUnits(pid);
      units = found.units;
      floor = found.floor;
      const time = await listFacilityTime(pid);
      periods = time.periods;
      totalMinutes = time.total_minutes;
    } catch (cause) {
      units = [];
      periods = [];
      unitError = errorText(cause);
    }
  }

  async function readPack(unit: string) {
    unitError = null;
    try {
      pack = { unit, data: await readInformationPack(unit) };
    } catch (cause) {
      pack = null;
      unitError = errorText(cause);
    }
  }

  let periodStart = $state("");
  let periodEnd = $state("");
  let minutes = $state(0);
  let budget = $state<number | null>(null);
  async function addPeriod() {
    if (!open) return;
    unitError = null;
    try {
      await addFacilityTime(open, {
        period_start: periodStart,
        period_end: periodEnd,
        minutes,
        ...(budget ? { budget_minutes: budget } : {}),
      });
      await showUnits(open);
    } catch (cause) {
      unitError = errorText(cause);
    }
  }

  const sections = (p: InformationPack["pack"]): { title: string; groups: PackGroup[] }[] => [
    { title: t("bargaining.by_department"), groups: p.by_department },
    { title: t("bargaining.by_employment_type"), groups: p.by_employment_type },
    { title: t("bargaining.by_pay_band"), groups: p.by_pay_band },
    { title: t("bargaining.contingent"), groups: p.contingent },
  ].filter((section) => section.groups.length > 0);

  const cellText = (c: PackCell) =>
    c.kind === "shown" ? String(c.count) : tf("bargaining.pack_withheld", { floor });

  let unitName = $state("");
  let department = $state("");
  let location = $state("");
  let employmentType = $state("");
  let jobTitle = $state("");
  async function addUnit() {
    if (!open) return;
    unitError = null;
    try {
      await addBargainingUnit(open, {
        name: unitName,
        ...(department ? { department } : {}),
        ...(location ? { location } : {}),
        ...(employmentType ? { employment_type: employmentType } : {}),
        ...(jobTitle ? { job_title: jobTitle } : {}),
      });
      unitName = department = location = employmentType = jobTitle = "";
      await showUnits(open);
      await load();
    } catch (cause) {
      unitError = errorText(cause);
    }
  }

  const coverageText = (c: UnitCoverage) =>
    c.kind === "shown"
      ? tf("bargaining.covered", { n: c.covered })
      : c.kind === "withheld"
        ? tf("bargaining.withheld", { floor })
        : t("bargaining.no_one");
</script>

<svelte:head><title>{tf("pages.bargaining.page_title", { bargaining: t("nav.bargaining") })}</title></svelte:head>

<h1>{t("nav.bargaining")}</h1>
<p class="muted">{t("bargaining.intro")}</p>
{#if error}<ErrorNote testid="error" prefixed message={error} />{/if}

{#if organizations.length === 0}
  <p>{t("common.noOrganization")}</p>
{:else}
  {#if organizations.length > 1}
    <label>{t("bargaining.organization")}
      <select bind:value={organization} data-testid="bg-organization">
        {#each organizations as o (o)}<option value={o}>{o}</option>{/each}
      </select>
    </label>
  {/if}

  {#if bodies}
    {#if bodies.length === 0}
      <p data-testid="bg-none">{t("bargaining.none")}</p>
    {:else}
      <ul class="plain" data-testid="bg-bodies">
        {#each bodies as b (b.pid)}
          <li class="panel" data-testid="bg-body">
            <h2>{b.name}</h2>
            <p>
              {tv(b.kind)} · {tv(b.basis)} · {tf("bargaining.since", { day: formatDate(b.recognised_on) })} ·
              {tf("bargaining.contact", { role: b.contact_role })} · {tf("bargaining.unit_count", { n: b.units })}
            </p>
            <button type="button" data-testid="bg-show-units" onclick={() => showUnits(b.pid)}>{t("bargaining.units")}</button>
            {#if open === b.pid}
              {#if unitError}<ErrorNote testid="unit-error" prefixed message={unitError} />{/if}
              <p class="muted">{tf("bargaining.floor_note", { floor })}</p>
              {#if units.length === 0}
                <p data-testid="bg-no-units">{t("bargaining.units_none")}</p>
              {:else}
                <ul class="plain" data-testid="bg-units">
                  {#each units as u (u.pid)}
                    <li data-testid="bg-unit">
                      <strong>{u.name}</strong>:
                      {[u.department, u.location, u.employment_type ? tv(u.employment_type) : null, u.job_title].filter(Boolean).join(" · ")}
                      — <span data-testid="bg-coverage">{coverageText(u.coverage)}</span>
                      <button type="button" data-testid="bg-pack" onclick={() => readPack(u.pid)}>{t("bargaining.pack")}</button>
                    </li>
                  {/each}
                </ul>
              {/if}
              {#if pack}
                {@const p = pack.data.pack}
                <div class="panel" data-testid="bg-pack-result">
                  <h3>{t("bargaining.pack")}</h3>
                  <p class="muted">{t("bargaining.pack_logged")}</p>
                  {#if p.headcount === null}
                    <p data-testid="bg-pack-nothing">{tf("bargaining.pack_under_floor", { floor: pack.data.floor })}</p>
                  {:else}
                    <p>{tf("bargaining.pack_headcount", { n: p.headcount, day: formatDate(p.as_of) })}</p>
                    {#each sections(p) as section (section.title)}
                      <h4>{section.title}</h4>
                      <ul>{#each section.groups as g (g.group)}<li>{tv(g.group)}: {cellText(g.headcount)}</li>{/each}</ul>
                    {/each}
                    <p>{t("bargaining.starters")}: {cellText(p.starters_last_year)} · {t("bargaining.leavers")}: {cellText(p.leavers_last_year)}</p>
                    <p data-testid="bg-vacancies">
                      {pack.data.vacancies.kind === "shown"
                        ? tf("bargaining.vacancies", { n: pack.data.vacancies.open_posts })
                        : t("bargaining.vacancies_unknown")}
                    </p>
                  {/if}
                </div>
              {/if}
              <h3>{t("bargaining.facility")}</h3>
              <p class="muted">{t("bargaining.facility_note")}</p>
              {#if periods.length === 0}
                <p data-testid="bg-no-facility">{t("bargaining.facility_none")}</p>
              {:else}
                <ul data-testid="bg-facility">
                  {#each periods as f (f.pid)}
                    <li data-testid="bg-period">
                      {formatDate(f.period_start)} – {formatDate(f.period_end)}: {tf("bargaining.minutes_hours", { minutes: f.minutes, hours: f.hours })}
                      {#if f.budget_minutes !== null} · {tf("bargaining.budget", { minutes: f.budget_minutes })}{/if}
                      {#if f.over_budget} · <strong data-testid="bg-over">{t("bargaining.over_budget")}</strong>{/if}
                    </li>
                  {/each}
                </ul>
                <p>{tf("bargaining.facility_total", { minutes: totalMinutes })}</p>
              {/if}
              <form data-testid="bg-facility-form" onsubmit={(e) => { e.preventDefault(); void addPeriod(); }}>
                <label>{t("bargaining.period_start")} <input type="date" bind:value={periodStart} required data-testid="bg-fstart" /></label>
                <label>{t("bargaining.period_end")} <input type="date" bind:value={periodEnd} required data-testid="bg-fend" /></label>
                <label>{t("bargaining.minutes")} <input type="number" min="0" bind:value={minutes} required data-testid="bg-fminutes" /></label>
                <label>{t("bargaining.budget_label")} <input type="number" min="0" bind:value={budget} data-testid="bg-fbudget" /></label>
                <button type="submit" data-testid="bg-fadd">{t("bargaining.add_period")}</button>
              </form>
              <form data-testid="bg-unit-form" onsubmit={(e) => { e.preventDefault(); void addUnit(); }}>
                <p class="muted">{t("bargaining.unit_rules")}</p>
                <label>{t("common.name")} <input bind:value={unitName} maxlength="120" required data-testid="bg-unit-name" /></label>
                <label>{t("common.department")} <input bind:value={department} maxlength="80" data-testid="bg-unit-department" /></label>
                <label>{t("bargaining.location")} <input bind:value={location} maxlength="80" data-testid="bg-unit-location" /></label>
                <label>{t("bargaining.employment_type")} <input bind:value={employmentType} maxlength="80" data-testid="bg-unit-type" /></label>
                <label>{t("common.jobTitle")} <input bind:value={jobTitle} maxlength="80" data-testid="bg-unit-title" /></label>
                <button type="submit" data-testid="bg-unit-add">{t("bargaining.add_unit")}</button>
              </form>
            {/if}
          </li>
        {/each}
      </ul>
    {/if}
  {:else if !error}
    <p>{t("common.loading")}</p>
  {/if}

  <form class="panel" data-testid="bg-form" onsubmit={(e) => { e.preventDefault(); void addBody(); }}>
    <h2>{t("bargaining.add_body")}</h2>
    <p class="muted">{t("bargaining.contact_note")}</p>
    <label>{t("common.name")} <input bind:value={name} maxlength="120" required data-testid="bg-name" /></label>
    <label>{t("bargaining.kind")}
      <select bind:value={kind} data-testid="bg-kind">{#each BODY_KINDS as k (k)}<option value={k}>{tv(k)}</option>{/each}</select>
    </label>
    <label>{t("bargaining.basis")}
      <select bind:value={basis} data-testid="bg-basis">{#each BODY_BASES as k (k)}<option value={k}>{tv(k)}</option>{/each}</select>
    </label>
    <label>{t("bargaining.recognised_on")} <input type="date" bind:value={recognisedOn} required data-testid="bg-date" /></label>
    <label>{t("bargaining.contact_role")} <input bind:value={contactRole} maxlength="80" required data-testid="bg-role" /></label>
    <button type="submit" data-testid="bg-add">{t("common.add")}</button>
  </form>
{/if}

<style>
  .plain { list-style: none; padding: 0; }
</style>
