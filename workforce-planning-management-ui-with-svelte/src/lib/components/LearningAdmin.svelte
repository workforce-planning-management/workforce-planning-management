<!--
  Statutory and mandatory learning, for HR (WPM-R170, WPM-T315): add a requirement, retire one, and record a completion of
  training done elsewhere. The service decides who may; a refusal is said in the catalogue's words. A statutory requirement
  must name its source. A completion holds a date and where it was done, never a certificate.
-->
<script lang="ts">
  import { attempt } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { t } from "#lib/i18n.svelte.js";
  import {
    addLearningRequirement,
    listLearningRequirements,
    listWorkers,
    recordLearningCompletion,
    retireLearningRequirement,
  } from "#lib/api/wpm.js";
  import type { Worker } from "#lib/api/types.js";

  let { organization, changed }: { organization: string; changed: () => void } = $props();

  let requirements = $state<{ pid: string; subject: string; basis: string }[]>([]);
  let people = $state<Worker[]>([]);
  let message = $state<string | null>(null);
  let saved = $state(false);

  let subject = $state("");
  let basis = $state("mandatory");
  let source = $state("");
  let refresher = $state("");
  let staffGroups = $state("");
  let jobRoles = $state("");
  let areas = $state("");

  let requirementPid = $state("");
  let workerPid = $state("");
  let completedOn = $state("");
  let whereDone = $state("");

  async function reload() {
    const [reqs, workers] = await Promise.all([
      listLearningRequirements(organization),
      listWorkers({ status: "active" }),
    ]);
    requirements = reqs;
    people = workers.filter((w) => w.organization_ref === organization);
  }

  $effect(() => {
    void organization;
    void reload().catch(() => {
      requirements = [];
      people = [];
    });
  });

  const list = (text: string) =>
    text
      .split(",")
      .map((s) => s.trim())
      .filter(Boolean);

  async function done(action: () => Promise<unknown>) {
    saved = false;
    const failed = await attempt(action);
    message = failed?.message ?? null;
    if (failed === null) {
      saved = true;
      await reload().catch(() => undefined);
      changed();
    }
    return failed === null;
  }

  async function add(event: SubmitEvent) {
    event.preventDefault();
    const months = Number.parseInt(refresher, 10);
    const ok = await done(() =>
      addLearningRequirement({
        organization,
        subject,
        basis,
        source: source.trim() || undefined,
        refresher_months: Number.isFinite(months) ? months : undefined,
        staff_groups: list(staffGroups),
        job_roles: list(jobRoles),
        areas_of_work: list(areas),
      }),
    );
    if (ok) {
      subject = "";
      source = "";
      refresher = "";
      staffGroups = jobRoles = areas = "";
    }
  }

  async function record(event: SubmitEvent) {
    event.preventDefault();
    const ok = await done(() =>
      recordLearningCompletion(requirementPid, {
        worker_pid: workerPid,
        completed_on: completedOn,
        source: whereDone.trim() || undefined,
      }),
    );
    if (ok) whereDone = "";
  }
</script>

<section class="panel" data-testid="learning-admin">
  <h2>{t("mandatoryLearning.addTitle")}</h2>
  <form onsubmit={add}>
    <label>{t("mandatoryLearning.subject")} <input bind:value={subject} required maxlength="120" data-testid="req-subject" /></label>
    <label
      >{t("mandatoryLearning.basis")}
      <select bind:value={basis} data-testid="req-basis">
        <option value="mandatory">{t("mandatoryLearning.mandatory")}</option>
        <option value="statutory">{t("mandatoryLearning.statutory")}</option>
      </select>
    </label>
    <label>{t("mandatoryLearning.source")} <input bind:value={source} maxlength="200" data-testid="req-source" required={basis === "statutory"} /></label>
    <label>{t("mandatoryLearning.refresher")} <input bind:value={refresher} inputmode="numeric" data-testid="req-refresher" /></label>
    <label>{t("mandatoryLearning.staffGroups")} <input bind:value={staffGroups} data-testid="req-staff-groups" /></label>
    <label>{t("mandatoryLearning.jobRoles")} <input bind:value={jobRoles} /></label>
    <label>{t("mandatoryLearning.areasOfWork")} <input bind:value={areas} /></label>
    <button type="submit" data-testid="req-save">{t("mandatoryLearning.save")}</button>
  </form>

  {#if requirements.length > 0}
    <h2>{t("mandatoryLearning.requirements")}</h2>
    <ul>
      {#each requirements as r (r.pid)}
        <li data-testid="req-row">
          {r.subject}
          <span class="chip">{t(`mandatoryLearning.${r.basis}` as "mandatoryLearning.mandatory")}</span>
          <button type="button" onclick={() => void done(() => retireLearningRequirement(r.pid))}>
            {t("mandatoryLearning.retire")}
          </button>
        </li>
      {/each}
    </ul>

    <h2>{t("mandatoryLearning.recordTitle")}</h2>
    <form onsubmit={record}>
      <label
        >{t("mandatoryLearning.requirement")}
        <select bind:value={requirementPid} required data-testid="comp-requirement">
          <option value=""></option>
          {#each requirements as r (r.pid)}<option value={r.pid}>{r.subject}</option>{/each}
        </select>
      </label>
      <label
        >{t("mandatoryLearning.worker")}
        <select bind:value={workerPid} required data-testid="comp-worker">
          <option value=""></option>
          {#each people as w (w.pid)}<option value={w.pid}>{w.display_name}</option>{/each}
        </select>
      </label>
      <label>{t("mandatoryLearning.completedOn")} <input type="date" bind:value={completedOn} required data-testid="comp-date" /></label>
      <label>{t("mandatoryLearning.whereDone")} <input bind:value={whereDone} maxlength="120" /></label>
      <button type="submit" data-testid="comp-save">{t("mandatoryLearning.save")}</button>
    </form>
  {/if}
  {#if message}<ErrorNote testid="admin-error" message={message} />{/if}
  {#if saved}<p role="status" data-testid="admin-saved">{t("mandatoryLearning.saved")}</p>{/if}
</section>
