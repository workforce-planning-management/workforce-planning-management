<!--
  The consultation register (`/consultations`): one record per proposal that may trigger a collective duty. It holds the body
  consulted, the proposal as counts and dates, topics, a document REFERENCE, when consultation began and ended and how. Never an
  attendee, a representative or what was said (WPM-D68). The timeline is a suggestion with its evidence, the deployer's to
  confirm; it is not a verdict. HR only.
-->
<script lang="ts">
  import { formatDate } from "#lib/locale-format.js";
  import { page } from "$app/state";
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import {
    CONSULTATION_OUTCOMES,
    CONSULTATION_PROFILES,
    CONSULTATION_TOPICS,
    beginConsultation,
    closeConsultation,
    consultationTimeline,
    listConsultations,
    listRecognisedBodies,
    openConsultation,
  } from "#lib/api/wpm.js";
  import type {
    Consultation,
    ConsultationTimeline,
    RecognisedBody,
    Timeliness,
  } from "#lib/api/wpm.js";
  import { t, tf, tp, tv } from "#lib/i18n.svelte.js";

  const organizations = $derived((page.data.scope ?? []) as string[]);
  let organization = $state("");
  $effect(() => {
    if (!organization && organizations.length > 0) organization = organizations[0] ?? "";
  });

  let rows = $state<Consultation[] | null>(null);
  let bodies = $state<RecognisedBody[]>([]);
  let error = $state<string | null>(null);
  const load = guardedLoad(
    async () => {
      const [found, known] = await Promise.all([
        listConsultations(organization),
        listRecognisedBodies(organization),
      ]);
      return { found, known };
    },
    ({ found, known }) => {
      rows = found.consultations;
      bodies = known.bodies;
      error = null;
    },
    (cause) => {
      error = errorText(cause);
    },
  );
  $effect(() => {
    if (organization) void load();
  });

  const bodyName = (pid: string) => bodies.find((b) => b.pid === pid)?.name ?? "";

  let bodyPid = $state("");
  $effect(() => {
    if (!bodyPid && bodies.length > 0) bodyPid = bodies[0]?.pid ?? "";
  });
  let establishment = $state("");
  let profile = $state<string>(CONSULTATION_PROFILES[0]);
  let size = $state(0);
  let dismissals = $state(0);
  let terminationOn = $state("");
  let topics = $state<string[]>([]);
  let documentRef = $state("");
  async function open() {
    error = null;
    try {
      await openConsultation({
        body_pid: bodyPid || (bodies[0]?.pid ?? ""),
        establishment,
        profile,
        establishment_size: size,
        dismissals,
        earliest_termination_on: terminationOn,
        topics,
        ...(documentRef ? { document_ref: documentRef } : {}),
      });
      establishment = documentRef = "";
      topics = [];
      await load();
    } catch (cause) {
      error = errorText(cause);
    }
  }

  let shown = $state<string | null>(null);
  let timeline = $state<ConsultationTimeline | null>(null);
  let rowError = $state<string | null>(null);
  async function showTimeline(pid: string) {
    shown = pid;
    rowError = null;
    try {
      timeline = await consultationTimeline(pid);
    } catch (cause) {
      timeline = null;
      rowError = errorText(cause);
    }
  }

  let startedOn = $state("");
  let outcome = $state<string>(CONSULTATION_OUTCOMES[0]);
  let endedOn = $state("");
  async function act(pid: string, run: () => Promise<unknown>) {
    rowError = null;
    try {
      await run();
      await load();
      await showTimeline(pid);
    } catch (cause) {
      rowError = errorText(cause);
    }
  }

  const timelinessText = (x: Timeliness) =>
    tf(`consultations.timeliness.${x.kind}`, {
      n: "days_left" in x ? x.days_left : "days_late" in x ? x.days_late : 0,
    });
  const checkText = (token: string) => tp(`consultations.check.${token}`) ?? token.replaceAll("_", " ");
</script>

<svelte:head><title>{tf("pages.consultations.page_title", { consultations: t("nav.consultations") })}</title></svelte:head>

<h1>{t("nav.consultations")}</h1>
<p class="muted">{t("consultations.intro")}</p>
{#if error}<ErrorNote testid="error" prefixed message={error} />{/if}

{#if organizations.length === 0}
  <p>{t("common.noOrganization")}</p>
{:else}
  {#if organizations.length > 1}
    <label>{t("bargaining.organization")}
      <select bind:value={organization} data-testid="cs-organization">
        {#each organizations as o (o)}<option value={o}>{o}</option>{/each}
      </select>
    </label>
  {/if}

  {#if rows}
    {#if rows.length === 0}
      <p data-testid="cs-none">{t("consultations.none")}</p>
    {:else}
      <ul class="plain" data-testid="cs-rows">
        {#each rows as c (c.pid)}
          <li class="panel" data-testid="cs-row">
            <h2>{c.establishment} · {bodyName(c.body_pid)}</h2>
            <p>
              {tv(c.profile)} · {c.topics.map((x) => tv(x)).join(", ")} ·
              {tf("consultations.counts", { size: c.establishment_size, dismissals: c.dismissals })} ·
              {tf("consultations.earliest", { day: formatDate(c.earliest_termination_on) })} ·
              <span data-testid="cs-outcome">{tv(c.outcome)}</span>
              {#if c.started_on} · {tf("consultations.began", { day: formatDate(c.started_on) })}{/if}
              {#if c.ended_on} · {tf("consultations.ended", { day: formatDate(c.ended_on) })}{/if}
            </p>
            <button type="button" data-testid="cs-timeline" onclick={() => showTimeline(c.pid)}>{t("consultations.timeline")}</button>
            {#if shown === c.pid}
              {#if rowError}<ErrorNote testid="row-error" prefixed message={rowError} />{/if}
              {#if timeline}
                <div data-testid="cs-assessment">
                  <p>
                    {timeline.assessment.may_apply ? t("consultations.may_apply") : t("consultations.nothing_met")}
                    ({tf("consultations.evidence", { counted: timeline.assessment.evidence.counted, needed: timeline.assessment.evidence.needed, days: timeline.assessment.evidence.window_days })})
                  </p>
                  {#if timeline.assessment.latest_consultation_start}
                    <p>{tf("consultations.latest_start", { day: formatDate(timeline.assessment.latest_consultation_start) })}</p>
                  {/if}
                  {#if timeline.assessment.latest_authority_notice}
                    <p>{tf("consultations.latest_notice", { day: timeline.assessment.latest_authority_notice })}</p>
                  {/if}
                  <p data-testid="cs-timeliness">{timelinessText(timeline.timeliness)}</p>
                  <ul>{#each timeline.assessment.checklist as item (item)}<li>{checkText(item)}</li>{/each}</ul>
                  <p class="muted">{t("consultations.confirm_note")}</p>
                  <p class="muted">{timeline.assessment.source}</p>
                </div>
              {/if}
              {#if c.outcome === "open"}
                {#if !c.started_on}
                  <form data-testid="cs-begin-form" onsubmit={(e) => { e.preventDefault(); void act(c.pid, () => beginConsultation(c.pid, { started_on: startedOn })); }}>
                    <label>{t("consultations.began_on")} <input type="date" bind:value={startedOn} required data-testid="cs-began" /></label>
                    <button type="submit" data-testid="cs-begin">{t("consultations.record_begin")}</button>
                  </form>
                {/if}
                <form data-testid="cs-close-form" onsubmit={(e) => { e.preventDefault(); void act(c.pid, () => closeConsultation(c.pid, { outcome, ended_on: endedOn })); }}>
                  <label>{t("consultations.outcome")}
                    <select bind:value={outcome} data-testid="cs-outcome-select">{#each CONSULTATION_OUTCOMES as o (o)}<option value={o}>{tv(o)}</option>{/each}</select>
                  </label>
                  <label>{t("consultations.ended_on")} <input type="date" bind:value={endedOn} required data-testid="cs-ended" /></label>
                  <button type="submit" data-testid="cs-close">{t("consultations.record_close")}</button>
                </form>
              {/if}
            {/if}
          </li>
        {/each}
      </ul>
    {/if}
  {:else if !error}
    <p>{t("common.loading")}</p>
  {/if}

  {#if bodies.length === 0}
    <p data-testid="cs-need-body">{t("consultations.need_body")}</p>
  {:else}
    <form class="panel" data-testid="cs-form" onsubmit={(e) => { e.preventDefault(); void open(); }}>
      <h2>{t("consultations.open")}</h2>
      <p class="muted">{t("consultations.open_note")}</p>
      <label>{t("consultations.body")}
        <select bind:value={bodyPid} data-testid="cs-body">{#each bodies as b (b.pid)}<option value={b.pid}>{b.name}</option>{/each}</select>
      </label>
      <label>{t("consultations.establishment")} <input bind:value={establishment} maxlength="80" required data-testid="cs-establishment" /></label>
      <label>{t("consultations.profile")}
        <select bind:value={profile} data-testid="cs-profile">{#each CONSULTATION_PROFILES as p (p)}<option value={p}>{tv(p)}</option>{/each}</select>
      </label>
      <label>{t("consultations.size")} <input type="number" min="0" bind:value={size} required data-testid="cs-size" /></label>
      <label>{t("consultations.dismissals")} <input type="number" min="0" bind:value={dismissals} required data-testid="cs-dismissals" /></label>
      <label>{t("consultations.termination")} <input type="date" bind:value={terminationOn} required data-testid="cs-termination" /></label>
      <fieldset>
        <legend>{t("consultations.topics")}</legend>
        {#each CONSULTATION_TOPICS as topic (topic)}
          <label><input type="checkbox" bind:group={topics} value={topic} data-testid={`cs-topic-${topic}`} /> {tv(topic)}</label>
        {/each}
      </fieldset>
      <label>{t("consultations.document_ref")} <input bind:value={documentRef} maxlength="200" data-testid="cs-document" /></label>
      <button type="submit" data-testid="cs-open">{t("consultations.open")}</button>
    </form>
  {/if}
{/if}

<style>
  .plain { list-style: none; padding: 0; }
</style>
