<script lang="ts">
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { page } from "$app/state";
  import CalendarPanel from "#lib/components/CalendarPanel.svelte";
  import RoleGap from "#lib/components/RoleGap.svelte";
  import Mobility from "#lib/components/Mobility.svelte";
  import Backups from "#lib/components/Backups.svelte";
  import SkillGaps from "#lib/components/SkillGaps.svelte";
  import DottedLinePanel from "#lib/components/DottedLinePanel.svelte";
  import ExpenseClaims from "#lib/components/ExpenseClaims.svelte";
  import PayPosition from "#lib/components/PayPosition.svelte";
  import EngagementPanel from "#lib/components/EngagementPanel.svelte";
  import ConversionPanel from "#lib/components/ConversionPanel.svelte";
  import JobLevel from "#lib/components/JobLevel.svelte";
  import EmergencyContacts from "#lib/components/EmergencyContacts.svelte";
  import GroupsPanel from "#lib/components/GroupsPanel.svelte";
  import TeamAspirations from "#lib/components/TeamAspirations.svelte";
  import Aspirations from "#lib/components/Aspirations.svelte";
  import CareerHistory from "#lib/components/CareerHistory.svelte";
  import FrameworkRolePanel from "#lib/components/FrameworkRolePanel.svelte";
  import {
    ADJUSTMENT_CATEGORIES,
    acknowledgeWellbeing,
    answerErgonomicItem,
    createAdjustmentRequest,
    decideAdjustment,
    listAdjustmentRequests,
    appraisalReport,
    appraisalRequests,
    appraisalStatus,
    completeErgonomicAssessment,
    createErgonomicAssessment,
    changeStatus,
    completeItem,
    createAppraisal,
    workerPayslips,
    eraseWorker,
    workerWellbeingPrompts,
    getAppraisal,
    getWorker,
    listAppraisals,
    listWorkers,
    listEntitlements,
    listErgonomicAssessments,
    listLeaveRequests,
    listNotifications,
    listOnboarding,
    listPulseSurveys,
    listReviews,
    listTraining,
    markNotificationRead,
    money,
    nominateRater,
    respondAppraisal,
    submitPulse,
    type AdjustmentRequest,
    type AppraisalRequest,
    type AppraisalSummary,
    type ErgonomicAssessment,
    type Notification,
    type PulseSurvey,
    type WellbeingPrompt,
  } from "#lib/api/wpm.js";
  import { mean } from "#lib/format.js";
  import { latestOnly, messageOf, settleEach, type Settled } from "#lib/loader.svelte.js";
  import { i18n, t, tf, tv } from "#lib/i18n.svelte.js";
  import DataTable from "#lib/components/DataTable.svelte";
  import type {
    DataGridColumn,
    DataGridRow,
  } from "@lilydesignsystem/svelte-data-grid";
  import type {
    Worker,
    LeaveEntitlement,
    LeaveRequest,
    OnboardingItem,
    Payslip,
    Review,
    TrainingEnrollment,
  } from "#lib/api/types.js";

  let worker = $state<Worker | null>(null);
  let onboarding = $state<OnboardingItem[]>([]);
  let balances = $state<LeaveEntitlement[]>([]);
  let leave = $state<LeaveRequest[]>([]);
  let payslips = $state<Payslip[]>([]);
  let reviews = $state<Review[]>([]);
  let training = $state<TrainingEnrollment[]>([]);
  let wellbeing = $state<WellbeingPrompt[]>([]);
  let pulseSurveys = $state<PulseSurvey[]>([]);
  let pulseThanks = $state<Set<string>>(new Set());
  let appraisals = $state<AppraisalSummary[]>([]);
  let myRequests = $state<AppraisalRequest[]>([]);
  let notifications = $state<Notification[]>([]);
  let ergonomics = $state<ErgonomicAssessment[]>([]);
  let workstation = $state("");
  let adjustments = $state<AdjustmentRequest[]>([]);
  let adjCategory = $state("quieter_workspace");
  let adjBarrier = $state("");
  let adjImpact = $state("");
  let adjChange = $state("");
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
  let error = $state<string | null>(null);
  let actionError = $state<string | null>(null);

  const pid = $derived(page.params.pid ?? "");

  // A slow answer for the worker you just left never overwrites the one you are on, and a failed
  // secondary panel says so in its own place while the rest still shows (WPM-T260).
  const guard = latestOnly();
  let panelErrors = $state<Record<string, string>>({});

  async function load() {
    const current = guard.begin();
    try {
      const found = await getWorker(pid);
      if (!current()) return;
      worker = found;
      error = null;
    } catch (cause) {
      if (current()) error = messageOf(cause);
      return;
    }
    const got = await settleEach({
      onboarding: () => listOnboarding(pid),
      balances: () => listEntitlements(pid),
      leave: () => listLeaveRequests(pid),
      payslips: () => workerPayslips(pid),
      reviews: () => listReviews(pid),
      training: () => listTraining(pid),
      wellbeing: async () => (await workerWellbeingPrompts(pid)).prompts,
      pulse: async () => (await listPulseSurveys()).filter((s) => s.open),
      appraisals: () => listAppraisals(pid),
      requests: () => appraisalRequests(pid),
      notifications: () => listNotifications(pid),
      ergonomics: () => listErgonomicAssessments(pid),
      adjustments: () => listAdjustmentRequests(pid),
    });
    if (!current()) return;
    const errors: Record<string, string> = {};
    const take = <T,>(name: string, outcome: Settled<T>, empty: T): T => {
      if (outcome.error !== null) {
        errors[name] = outcome.error;
        return empty;
      }
      return outcome.value as T;
    };
    onboarding = take("onboarding", got.onboarding, []);
    balances = take("balances", got.balances, []);
    leave = take("leave", got.leave, []);
    payslips = take("payslips", got.payslips, []);
    reviews = take("reviews", got.reviews, []);
    training = take("training", got.training, []);
    wellbeing = take("wellbeing", got.wellbeing, []);
    pulseSurveys = take("pulse", got.pulse, []);
    appraisals = take("appraisals", got.appraisals, []);
    myRequests = take("requests", got.requests, []);
    notifications = take("notifications", got.notifications, []);
    ergonomics = take("ergonomics", got.ergonomics, []);
    adjustments = take("adjustments", got.adjustments, []);
    panelErrors = errors;
    try {
      if (openAppraisal) await toggleAppraisal(openAppraisal.pid, true);
    } catch (cause) {
      if (current()) actionError = messageOf(cause);
    }
  }

  $effect(() => {
    if (pid) void load();
  });

  async function transition(to: string) {
    actionError = null;
    try {
      await changeStatus(pid, to);
      await load();
    } catch (cause) {
      actionError = errorText(cause);
    }
  }

  async function complete(itemPid: string) {
    actionError = null;
    try {
      await completeItem(itemPid);
      await load();
    } catch (cause) {
      actionError = errorText(cause);
    }
  }

  async function respond(
    entitlementPid: string,
    response: "booked" | "done" | "declined" | "dismissed",
  ) {
    actionError = null;
    try {
      await acknowledgeWellbeing(pid, entitlementPid, response);
      await load();
    } catch (cause) {
      actionError = errorText(cause);
    }
  }

  async function sendPulse(surveyPid: string, score: number) {
    actionError = null;
    try {
      await submitPulse(surveyPid, pid, score);
      pulseThanks = new Set([...pulseThanks, surveyPid]);
    } catch (cause) {
      actionError = errorText(cause);
    }
  }

  async function toggleAppraisal(appraisalPid: string, keepOpen = false) {
    actionError = null;
    if (!keepOpen && openAppraisal?.pid === appraisalPid) {
      openAppraisal = null;
      openReport = null;
      return;
    }
    try {
      openAppraisal = await getAppraisal(appraisalPid);
      openReport =
        openAppraisal.status === "shared" ? await appraisalReport(appraisalPid) : null;
      if (!colleagues.length) colleagues = await listWorkers();
      raterScores = Object.fromEntries(openAppraisal.competencies.map((c) => [c, 3]));
    } catch (cause) {
      actionError = errorText(cause);
    }
  }

  async function act(action: () => Promise<unknown>) {
    actionError = null;
    try {
      await action();
      await load();
    } catch (cause) {
      actionError = errorText(cause);
    }
  }

  const balanceRow = (row: DataGridRow) => row as unknown as LeaveEntitlement;
  const balanceColumns = $derived<DataGridColumn[]>([
    {
      id: "kind",
      header: t("columns.typeYear"),
      sortable: true,
      accessor: (r) => `${tv(balanceRow(r).kind)} ${balanceRow(r).year}`,
    },
    {
      id: "remaining",
      header: t("columns.remaining"),
      sortable: true,
      accessor: (r) => balanceRow(r).entitled_days - balanceRow(r).used_days,
      format: (_v, r) =>
        `${balanceRow(r).entitled_days - balanceRow(r).used_days} / ${balanceRow(r).entitled_days} ${t("common.days")}`,
    },
  ]);
  const leaveColumns = $derived<DataGridColumn[]>([
    { id: "kind", header: t("columns.kind"), sortable: true, accessor: (r) => tv(String(r.kind)) },
    {
      id: "start_on",
      header: t("columns.dates"),
      sortable: true,
      format: (_v, r) =>
        `${r.start_on} → ${r.end_on} (${r.days} ${t("common.days")})`,
    },
    { id: "status", header: t("common.status"), sortable: true, cell: statusChip },
  ]);
  const payslipColumns = $derived<DataGridColumn[]>([
    {
      id: "gross_minor",
      header: t("pay.gross"),
      sortable: true,
      align: "end",
      format: (_v, r) => money(Number(r.gross_minor), String(r.currency), i18n.locale),
    },
    {
      id: "net_minor",
      header: t("pay.net"),
      sortable: true,
      align: "end",
      format: (_v, r) => money(Number(r.net_minor), String(r.currency), i18n.locale),
    },
  ]);
  const reviewColumns = $derived<DataGridColumn[]>([
    { id: "status", header: t("common.status"), sortable: true, cell: statusChip },
    { id: "rating", header: t("columns.rating"), sortable: true },
    { id: "content", header: t("columns.content"), format: (v) => (v == null ? "" : String(v)) },
  ]);
  const trainingColumns = $derived<DataGridColumn[]>([
    { id: "course_ref", header: t("columns.course"), sortable: true, cell: courseCell },
    { id: "status", header: t("common.status"), sortable: true, cell: statusChip },
    {
      id: "certificate_expires_on",
      header: t("columns.certificateExpires"),
      sortable: true,
      format: (v) => (v == null ? "" : String(v)),
    },
  ]);
</script>

<svelte:head><title>{page.data.title}</title></svelte:head>

{#snippet courseCell({ row, value }: { row: DataGridRow; value: unknown })}
  {#if row.lms_url}
    <a href={String(row.lms_url)} target="_blank" rel="noopener noreferrer" title={t("pages.cpd.lms_open")}>{String(value)} ↗</a>
  {:else}
    {String(value)}
  {/if}
{/snippet}

{#snippet statusChip({ value }: { value: unknown })}
  <span class="chip">{tv(String(value))}</span>
{/snippet}

{#snippet panelError(name: string)}
  {#if panelErrors[name]}<ErrorNote testid="panel-error-{name}" prefixed message={panelErrors[name]} />{/if}
{/snippet}

{#if error}
  <ErrorNote testid="error" prefixed message={error} />
{:else if worker === null}
  <p>{t("common.loading")}</p>
{:else}
  <h1>{worker.display_name} <span class="muted">({worker.worker_number})</span></h1>
  <div class="panel" data-testid="facts">
    <p>
      {worker.job_title} · {worker.department} ·
      {#if worker.location}{worker.location} ·{/if}
      <span class={`chip status-${worker.status}`}>{tv(worker.status)}</span>
      {tf("pages.workersPid.fte", { fte_percent: worker.fte_percent })}
    </p>
    <p>
      {t("emp.salary")}:
      {#if worker.salary_minor === null}
        <span class="muted" data-testid="salary-masked">{t("common.masked")}</span>
      {:else}
        <span data-testid="salary">{money(worker.salary_minor, worker.salary_currency, i18n.locale)}</span>
      {/if}
    </p>
    {#if worker.status === "onboarding"}
      <button onclick={() => void transition("active")}>{tf("pages.workersPid.active", { actions: t("common.actions") })}</button>
    {/if}
    {#if actionError}
      <ErrorNote testid="action-error" message={actionError} />
    {/if}
    <p>
      <a href={`/api/proxy/workers/${pid}/subject-access`} target="_blank" rel="noreferrer" data-testid="subject-access">
        {t("emp.subjectAccess")}
      </a>
      {#if worker.status === "terminated" || worker.status === "retired"}
        <button
          data-testid="erase"
          onclick={() => {
            if (window.confirm(t("emp.erase") + "?")) {
              void act(async () => {
                await eraseWorker(pid);
                window.location.assign("/workers");
              });
            }
          }}
        >{t("emp.erase")}</button>
      {/if}
    </p>
  </div>

  {#if panelErrors["wellbeing"]}
    <h2>{t("wb.prompts")}</h2>
    {@render panelError("wellbeing")}
  {/if}
  {#if wellbeing.length}
    <h2>{t("wb.prompts")}</h2>
    <ul class="panel" data-testid="wellbeing-prompts">
      {#each wellbeing as prompt (prompt.entitlement_pid)}
        <li>
          <strong>{prompt.name}</strong>
          <span class="chip">{prompt.entitlement_kind === "benefit" ? t("wb.kind.benefit") : t("wb.kind.health")}</span>
          {#if prompt.kind === "reminder"}<span class="chip">{t("wb.reminder")}</span>{/if}
          <br />
          {prompt.description}
          {#if prompt.info_url}
            · <a href={prompt.info_url} target="_blank" rel="noreferrer">{t("wb.info")}</a>
          {/if}
          <br />
          <button onclick={() => void respond(prompt.entitlement_pid, "booked")}>{t("wb.booked")}</button>
          <button onclick={() => void respond(prompt.entitlement_pid, "done")}>{t("wb.done")}</button>
          <button onclick={() => void respond(prompt.entitlement_pid, "declined")}>{t("wb.declined")}</button>
          <button onclick={() => void respond(prompt.entitlement_pid, "dismissed")}>{t("wb.dismissed")}</button>
        </li>
      {/each}
    </ul>
  {/if}

  {#if panelErrors["pulse"]}
    <h2>{t("wb.pulse")}</h2>
    {@render panelError("pulse")}
  {/if}
  {#if pulseSurveys.length}
    <h2>{t("wb.pulse")}</h2>
    <ul class="panel" data-testid="pulse-surveys">
      {#each pulseSurveys as survey (survey.pid)}
        <li>
          <strong>{survey.name}</strong><br />
          {survey.question}<br />
          {#if pulseThanks.has(survey.pid)}
            <span class="muted">{t("wb.pulseThanks")}</span>
          {:else}
            {#each [1, 2, 3, 4, 5] as score (score)}
              <button onclick={() => void sendPulse(survey.pid, score)}>{score}</button>
            {/each}
          {/if}
        </li>
      {/each}
    </ul>
  {/if}

  {#if panelErrors["onboarding"]}
    <h2>{t("emp.onboarding")}</h2>
    {@render panelError("onboarding")}
  {/if}
  {#if onboarding.length}
    <h2>{t("emp.onboarding")}</h2>
    <ul class="panel">
      {#each onboarding as item (item.pid)}
        <li>
          {item.name}
          <span class="chip">{tv(item.status)}</span>
          {#if item.mandatory}<strong>·</strong>{/if}
          {#if item.status === "pending"}
            <button onclick={() => void complete(item.pid)}>✓</button>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}

  <h2>{t("emp.balances")}</h2>
  {@render panelError("balances")}
  <DataTable
    label={t("emp.balances")}
    columns={balanceColumns}
    rows={balances as unknown as DataGridRow[]}
    rowId={(row) => String(row.pid)}
  />

  <h2>{t("wf.leaveRequests")}</h2>
  {@render panelError("leave")}
  <DataTable
    label={t("wf.leaveRequests")}
    columns={leaveColumns}
    rows={leave as unknown as DataGridRow[]}
    rowId={(row) => String(row.pid)}
  />
  <CalendarPanel
    testid="leave-calendar"
    label={t("calendar.leave")}
    events={leave.map((r) => ({ id: r.pid, start: r.start_on, end: r.end_on, title: tv(r.kind), values: { status: tv(r.status) } }))}
    fields={[{ id: "status", header: t("common.status") }]}
  />

  <h2>{t("emp.payslips")}</h2>
  {@render panelError("payslips")}
  <DataTable
    label={t("emp.payslips")}
    columns={payslipColumns}
    rows={payslips as unknown as DataGridRow[]}
    rowId={(row) => String(row.pid)}
  />

  {#if panelErrors["notifications"]}
    <h2>{t("emp.notifications")}</h2>
    {@render panelError("notifications")}
  {/if}
  {#if notifications.length}
    <h2>{t("emp.notifications")}</h2>
    <ul class="panel" data-testid="notifications">
      {#each notifications as notification (notification.pid)}
        <li>
          {notification.body}
          {#if notification.read_at === null}
            <button onclick={() => void act(() => markNotificationRead(notification.pid))}>
              {t("notif.markRead")}
            </button>
          {:else}
            <span class="muted">✓</span>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}

  {#if panelErrors["requests"]}
    <h2>{t("ap.requests")}</h2>
    {@render panelError("requests")}
  {/if}
  {#if myRequests.length}
    <h2>{t("ap.requests")}</h2>
    <ul class="panel" data-testid="appraisal-requests">
      {#each myRequests as request (request.appraisal_pid)}
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
  {@render panelError("appraisals")}
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

  <h2>{t("adj.title")}</h2>
  {@render panelError("adjustments")}
  <div class="panel" data-testid="adjustments">
    <p class="muted">{t("adj.hint")}</p>
    <select aria-label={t("labels.adjustmentCategory")} bind:value={adjCategory}>
      {#each ADJUSTMENT_CATEGORIES as category (category)}
        <option value={category}>{tv(category)}</option>
      {/each}
    </select>
    <input placeholder={t("adj.barrier")} aria-label={t("adj.barrier")} bind:value={adjBarrier} />
    <input placeholder={t("adj.impact")} aria-label={t("adj.impact")} bind:value={adjImpact} />
    <input placeholder={t("adj.change")} aria-label={t("adj.change")} bind:value={adjChange} />
    <button
      disabled={!adjBarrier.trim() || !adjImpact.trim() || !adjChange.trim()}
      onclick={() => void act(async () => {
        await createAdjustmentRequest(pid, {
          category: adjCategory,
          barrier: adjBarrier.trim(),
          impact: adjImpact.trim(),
          adjustment: adjChange.trim(),
        });
        adjBarrier = adjImpact = adjChange = "";
      })}
    >{t("adj.new")}</button>
    <ul>
      {#each adjustments as request (request.pid)}
        <li>
          <span class="chip">{tv(request.category)}</span>
          <span class="chip">{tv(request.status)}</span>
          {#if !request.words_withheld}
            <br />{request.barrier} → {request.adjustment}
            {#if request.decision_note}<br /><span class="muted">{request.decision_note}</span>{/if}
          {/if}
          {#if request.status === "requested"}
            <button
              onclick={() => {
                const note = window.prompt(t("adj.note")) ?? "";
                void act(() => decideAdjustment(request.pid, "agreed", note.trim() || undefined));
              }}
            >{t("adj.agree")}</button>
            <button onclick={() => void act(() => decideAdjustment(request.pid, "declined"))}>{t("adj.decline")}</button>
          {:else if request.status === "agreed"}
            <button onclick={() => void act(() => decideAdjustment(request.pid, "in_place"))}>{t("adj.inPlace")}</button>
          {/if}
        </li>
      {/each}
    </ul>
  </div>

  <h2>{t("erg.title")}</h2>
  {@render panelError("ergonomics")}
  <div class="panel" data-testid="ergonomics">
    <input placeholder={t("erg.workstation")} aria-label={t("erg.workstation")} bind:value={workstation} />
    <button
      disabled={!workstation.trim()}
      onclick={() => void act(async () => {
        await createErgonomicAssessment(pid, workstation.trim());
        workstation = "";
      })}
    >{t("erg.new")}</button>
    {#each ergonomics as assessment (assessment.pid)}
      <div>
        <strong>{assessment.workstation}</strong>
        <span class="chip">{tv(assessment.status)}</span>
        {#if assessment.status === "open"}
          <button onclick={() => void act(() => completeErgonomicAssessment(assessment.pid))}>
            {t("erg.complete")}
          </button>
        {:else if assessment.open_issues > 0}
          <span class="chip">{assessment.open_issues} ⚠</span>
        {/if}
        <ul>
          {#each assessment.items as item (item.pid)}
            <li>
              {item.name}
              {#if item.ok === true}✓{:else if item.ok === false}✗ <span class="muted">{item.note}</span>{/if}
              {#if assessment.status === "open" && item.ok === null}
                <button onclick={() => void act(() => answerErgonomicItem(item.pid, true))}>{t("erg.ok")}</button>
                <button
                  onclick={() => {
                    const note = window.prompt(t("erg.issue")) ?? "";
                    void act(() => answerErgonomicItem(item.pid, false, note.trim() || undefined));
                  }}
                >{t("erg.issue")}</button>
              {/if}
            </li>
          {/each}
        </ul>
      </div>
    {/each}
  </div>

  <h2>{t("emp.reviews")}</h2>
  {@render panelError("reviews")}
  <DataTable
    label={t("emp.reviews")}
    columns={reviewColumns}
    rows={reviews as unknown as DataGridRow[]}
    rowId={(row) => String(row.pid)}
  />

  <h2>{t("emp.training")}</h2>
  {@render panelError("training")}
  <DataTable
    label={t("emp.training")}
    columns={trainingColumns}
    rows={training as unknown as DataGridRow[]}
    rowId={(row) => String(row.pid)}
  />
  <RoleGap workerPid={worker.pid} />
  <Mobility workerPid={worker.pid} />
  {@const ownPid = worker.pid}
  {@const mine = ((page.data.organizations ?? []) as Array<{ worker_pid: string | null }>).some(
    (m) => m.worker_pid === ownPid,
  )}
  <FrameworkRolePanel
    workerPid={worker.pid}
    framework="uk-gdad-pcf"
    title={t("pages.workersPid.role_and_skills_uk_gdad_pcf")}
    actingFor={mine ? undefined : worker.display_name}
  />
  <FrameworkRolePanel
    workerPid={worker.pid}
    framework="esco"
    title={t("pages.workersPid.role_and_skills_esco")}
    actingFor={mine ? undefined : worker.display_name}
  />
  <CareerHistory workerPid={worker.pid} />
  <Aspirations workerPid={worker.pid} />
  <SkillGaps workerPid={worker.pid} />
  <EngagementPanel workerPid={worker.pid} />
  <ConversionPanel
    workerPid={worker.pid}
    employmentType={worker.employment_type}
    terminated={worker.terminated_on !== null}
    own={mine}
  />
  <JobLevel workerPid={worker.pid} />
  <PayPosition workerPid={worker.pid} />
  <ExpenseClaims workerPid={worker.pid} />
  <EmergencyContacts workerPid={worker.pid} />
  <Backups workerPid={worker.pid} />
  <DottedLinePanel workerPid={worker.pid} />
  <GroupsPanel workerPid={worker.pid} />
  <TeamAspirations workerPid={worker.pid} />
{/if}
