<!--
  Decision gates (`/decision-gates`): which kind of change points at which recognised body to consult. The check says what a change
  needs and ADVISES; it never blocks. Whether consultation is required is for the law and the agreements to say. HR only.
-->
<script lang="ts">
  import { page } from "$app/state";
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import {
    checkDecisionGate,
    createDecisionGate,
    listDecisionGates,
    listRecognisedBodies,
  } from "#lib/api/wpm.js";
  import type { DecisionGate, GateCheck, RecognisedBody } from "#lib/api/wpm.js";
  import { t, tf, tv } from "#lib/i18n.svelte.js";

  const organizations = $derived((page.data.scope ?? []) as string[]);
  let organization = $state("");
  $effect(() => {
    if (!organization && organizations.length > 0) organization = organizations[0] ?? "";
  });

  let changes = $state<string[]>([]);
  let gates = $state<DecisionGate[] | null>(null);
  let bodies = $state<RecognisedBody[]>([]);
  let error = $state<string | null>(null);
  const load = guardedLoad(
    async () => {
      const [found, known] = await Promise.all([
        listDecisionGates(organization),
        listRecognisedBodies(organization),
      ]);
      return { found, known };
    },
    ({ found, known }) => {
      changes = found.changes;
      gates = found.gates;
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

  let changeKind = $state("");
  $effect(() => {
    if (!changeKind && changes.length > 0) changeKind = changes[0] ?? "";
  });
  let bodyPid = $state("");
  $effect(() => {
    if (!bodyPid && bodies.length > 0) bodyPid = bodies[0]?.pid ?? "";
  });
  let note = $state("");
  async function add() {
    error = null;
    try {
      await createDecisionGate({
        organization_ref: organization,
        change_kind: changeKind || (changes[0] ?? ""),
        body_pid: bodyPid || (bodies[0]?.pid ?? ""),
        ...(note ? { note } : {}),
      });
      note = "";
      await load();
    } catch (cause) {
      error = errorText(cause);
    }
  }

  let checked = $state<GateCheck | null>(null);
  async function check(change: string) {
    error = null;
    try {
      checked = await checkDecisionGate(organization, change);
    } catch (cause) {
      checked = null;
      error = errorText(cause);
    }
  }
</script>

<svelte:head><title>{tf("pages.decisionGates.page_title", { decisionGates: t("nav.decisionGates") })}</title></svelte:head>

<h1>{t("nav.decisionGates")}</h1>
<p class="muted">{t("gates.intro")}</p>
{#if error}<ErrorNote testid="error" prefixed message={error} />{/if}

{#if organizations.length === 0}
  <p>{t("common.noOrganization")}</p>
{:else}
  {#if organizations.length > 1}
    <label>{t("bargaining.organization")}
      <select bind:value={organization} data-testid="dg-organization">
        {#each organizations as o (o)}<option value={o}>{o}</option>{/each}
      </select>
    </label>
  {/if}

  {#if gates}
    {#if gates.length === 0}
      <p data-testid="dg-none">{t("gates.none")}</p>
    {:else}
      <ul data-testid="dg-gates">
        {#each gates as g (g.pid)}
          <li data-testid="dg-gate">
            {tv(g.change_kind)} → {bodyName(g.body_pid)}{#if g.note} · {g.note}{/if}
            <button type="button" data-testid="dg-check" onclick={() => check(g.change_kind)}>{t("gates.check")}</button>
          </li>
        {/each}
      </ul>
    {/if}
  {:else if !error}
    <p>{t("common.loading")}</p>
  {/if}

  {#if checked}
    <div class="panel" data-testid="dg-result">
      <h2>{tv(checked.change)}</h2>
      <p data-testid="dg-standing">
        {tf(`gates.standing.${checked.standing.kind}`, { n: checked.standing.kind === "reference_needed" ? checked.standing.bodies : 0 })}
      </p>
      {#if checked.equality_impact_template}<p>{tf("gates.template", { section: checked.equality_impact_template })}</p>{/if}
      <p class="muted">{t("gates.advice")}</p>
    </div>
  {/if}

  {#if bodies.length === 0}
    <p data-testid="dg-need-body">{t("consultations.need_body")}</p>
  {:else}
    <form class="panel" data-testid="dg-form" onsubmit={(e) => { e.preventDefault(); void add(); }}>
      <h2>{t("gates.add")}</h2>
      <label>{t("gates.change")}
        <select bind:value={changeKind} data-testid="dg-change">{#each changes as c (c)}<option value={c}>{tv(c)}</option>{/each}</select>
      </label>
      <label>{t("consultations.body")}
        <select bind:value={bodyPid} data-testid="dg-body">{#each bodies as b (b.pid)}<option value={b.pid}>{b.name}</option>{/each}</select>
      </label>
      <label>{t("gates.note")} <input bind:value={note} maxlength="200" data-testid="dg-note" /></label>
      <button type="submit" data-testid="dg-add">{t("common.add")}</button>
    </form>
  {/if}
{/if}
