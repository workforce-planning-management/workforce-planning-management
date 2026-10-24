<!--
  360° appraisals (WPM-R77): the requests to respond to someone else's, and the worker's own appraisals with
  their raters, responses and, once shared, the report. A group of fewer than three responses is withheld.
-->
<script lang="ts">
  import {
    appraisalReport,
    appraisalStatus,
    createAppraisal,
    getAppraisal,
    listWorkers,
    nominateRater,
    respondAppraisal,
    type AppraisalRequest,
    type AppraisalSummary,
  } from "#lib/api/wpm.js";
  import type { Worker } from "#lib/api/types.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { mean } from "#lib/format.js";
  import { t, tv } from "#lib/i18n.svelte.js";
  import { errorText } from "#lib/write.js";
  import { untrack } from "svelte";

  let {
    pid,
    appraisals,
    requests,
    requestsError = null,
    appraisalsError = null,
    onchange,
    onfail,
  }: {
    pid: string;
    appraisals: AppraisalSummary[];
    requests: AppraisalRequest[];
    requestsError?: string | null;
    appraisalsError?: string | null;
    /** Called after a change that the page should read again. */
    onchange: () => Promise<void>;
    /** Called with the words for a failure, or `null` when an action starts. */
    onfail: (message: string | null) => void;
  } = $props();

  let requestScores = $state<Record<string, number>>({});
  let requestComment = $state("");
  let openRequest = $state<string | null>(null);
  let openAppraisal = $state<Awaited<ReturnType<typeof getAppraisal>> | null>(null);
  let openReport = $state<Awaited<ReturnType<typeof appraisalReport>> | null>(null);
  let colleagues = $state<Worker[]>([]);
  let nomineePid = $state("");
  let nomineeGroup = $state<"manager" | "peer" | "report">("peer");
  let raterPid = $state("");
  let raterScores = $state<Record<string, number>>({});
  let raterComment = $state("");

  async function toggleAppraisal(appraisalPid: string, keepOpen = false) {
    onfail(null);
    if (!keepOpen && openAppraisal?.pid === appraisalPid) {
      openAppraisal = null;
      openReport = null;
      return;
    }
    try {
      openAppraisal = await getAppraisal(appraisalPid);
      openReport = openAppraisal.status === "shared" ? await appraisalReport(appraisalPid) : null;
      if (!colleagues.length) colleagues = await listWorkers();
      raterScores = Object.fromEntries(openAppraisal.competencies.map((c) => [c, 3]));
    } catch (cause) {
      onfail(errorText(cause));
    }
  }

  // Whenever the page reads the appraisals again, the one that is open is read again with them.
  $effect(() => {
    void appraisals;
    const open = untrack(() => openAppraisal);
    if (open) void toggleAppraisal(open.pid, true);
  });

  async function act(action: () => Promise<unknown>) {
    onfail(null);
    try {
      await action();
      await onchange();
    } catch (cause) {
      onfail(errorText(cause));
    }
  }
</script>

{#if requestsError}
  <h2>{t("ap.requests")}</h2>
  <ErrorNote testid="panel-error-requests" prefixed message={requestsError} />
{/if}
{#if requests.length}
  <h2>{t("ap.requests")}</h2>
  <ul class="panel" data-testid="appraisal-requests">
    {#each requests as request (request.appraisal_pid)}
      <li>
        <strong>{request.subject ?? request.subject_pid.slice(0, 8)}</strong>
        <span class="chip">{request.group}</span>
        {#if openRequest === request.appraisal_pid}
          {#each request.competencies as competency (competency)}
            <label>
              {competency}
              <select bind:value={requestScores[competency]}>
                {#each [1, 2, 3, 4, 5] as score (score)}
                  <option value={score}>{score}</option>
                {/each}
              </select>
            </label>
          {/each}
          <input placeholder={t("labels.comment")} aria-label={t("labels.comment")} bind:value={requestComment} />
          <button
            onclick={() => void act(async () => {
              await respondAppraisal(request.appraisal_pid, pid, requestScores, requestComment.trim() || undefined);
              openRequest = null;
            })}
          >{t("ap.respond")}</button>
        {:else}
          <button
            onclick={() => {
              openRequest = request.appraisal_pid;
              requestScores = Object.fromEntries(request.competencies.map((c) => [c, 3]));
              requestComment = "";
            }}
          >{t("ap.respond")}</button>
        {/if}
      </li>
    {/each}
  </ul>
{/if}

<h2>{t("ap.title")}</h2>
{#if appraisalsError}<ErrorNote testid="panel-error-appraisals" prefixed message={appraisalsError} />{/if}
<div class="panel" data-testid="appraisals">
  <button onclick={() => void act(() => createAppraisal(pid, ["communication", "collaboration", "delivery"]))}>
    {t("ap.new")}
  </button>
  {#each appraisals as appraisal (appraisal.pid)}
    <div>
      <button onclick={() => void toggleAppraisal(appraisal.pid)}>
        <span class="chip">{tv(appraisal.status)}</span>
        {appraisal.competencies.join(" · ")}
        ({appraisal.responded}/{appraisal.nominated})
      </button>
      {#if appraisal.status === "draft"}
        <button onclick={() => void act(() => appraisalStatus(appraisal.pid, "collecting"))}>{t("ap.start")}</button>
      {:else if appraisal.status === "collecting"}
        <button onclick={() => void act(() => appraisalStatus(appraisal.pid, "shared"))}>{t("ap.share")}</button>
      {/if}
      {#if openAppraisal?.pid === appraisal.pid}
        <ul>
          {#each openAppraisal.nominations as nomination (nomination.pid)}
            <li>
              {nomination.display_name ?? nomination.rater_pid.slice(0, 8)}
              <span class="chip">{nomination.group}</span>
              {#if nomination.responded}✓{/if}
            </li>
          {/each}
        </ul>
        {#if openAppraisal.status === "draft"}
          <select aria-label={t("labels.nominee")} bind:value={nomineePid}>
            <option value="">—</option>
            {#each colleagues.filter((c) => c.pid !== pid) as colleague (colleague.pid)}
              <option value={colleague.pid}>{colleague.display_name}</option>
            {/each}
          </select>
          <select aria-label={t("labels.nomineeGroup")} bind:value={nomineeGroup}>
            <option value="manager">{t("pages.workersPid.manager")}</option>
            <option value="peer">{t("pages.workersPid.peer")}</option>
            <option value="report">{t("pages.workersPid.report")}</option>
          </select>
          <button
            disabled={!nomineePid}
            onclick={() => void act(() => nominateRater(appraisal.pid, nomineePid, nomineeGroup))}
          >{t("ap.nominate")}</button>
        {:else if openAppraisal.status === "collecting"}
          <select aria-label={t("labels.rater")} bind:value={raterPid}>
            <option value="">—</option>
            {#each openAppraisal.nominations.filter((n) => !n.responded) as nomination (nomination.pid)}
              <option value={nomination.rater_pid}>{nomination.display_name ?? nomination.rater_pid.slice(0, 8)}</option>
            {/each}
          </select>
          {#each openAppraisal.competencies as competency (competency)}
            <label>
              {competency}
              <select bind:value={raterScores[competency]}>
                {#each [1, 2, 3, 4, 5] as score (score)}
                  <option value={score}>{score}</option>
                {/each}
              </select>
            </label>
          {/each}
          <input placeholder={t("labels.comment")} aria-label={t("labels.comment")} bind:value={raterComment} />
          <button
            disabled={!raterPid}
            onclick={() => void act(() => respondAppraisal(appraisal.pid, raterPid, raterScores, raterComment.trim() || undefined))}
          >{t("ap.respond")}</button>
        {/if}
        {#if openReport}
          <h3>{t("ap.report")}</h3>
          {#each openReport.groups as group (group.group)}
            <div>
              <strong>{group.group}</strong>
              {#if group.withheld}
                <span class="muted">{t("ap.withheld")}</span>
              {:else}
                {#each Object.entries(group.competencies ?? {}) as [competency, cell] (competency)}
                  <span class="chip">{competency}: {mean(cell.mean)} (n={cell.count})</span>
                {/each}
                {#each group.comments ?? [] as comment (comment)}
                  <p class="muted">“{comment}”</p>
                {/each}
              {/if}
            </div>
          {/each}
          <p class="muted">{openReport.derivation}</p>
        {/if}
      {/if}
    </div>
  {/each}
</div>

