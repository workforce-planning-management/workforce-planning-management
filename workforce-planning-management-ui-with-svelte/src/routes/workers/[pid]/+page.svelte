<script lang="ts">
  import WellbeingPrompts from "#lib/components/worker/WellbeingPrompts.svelte";
  import Appraisals from "#lib/components/worker/Appraisals.svelte";
  import AdjustmentRequests from "#lib/components/worker/AdjustmentRequests.svelte";
  import ErgonomicAssessments from "#lib/components/worker/ErgonomicAssessments.svelte";
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
  import EquipmentPanel from "#lib/components/EquipmentPanel.svelte";
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
    listAdjustmentRequests,
    appraisalRequests,
    changeStatus,
    completeItem,
    workerPayslips,
    eraseWorker,
    workerWellbeingPrompts,
    getWorker,
    listAppraisals,
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
    type AdjustmentRequest,
    type AppraisalRequest,
    type AppraisalSummary,
    type ErgonomicAssessment,
    type Notification,
    type PulseSurvey,
    type WellbeingPrompt,
  } from "#lib/api/wpm.js";
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
  let appraisals = $state<AppraisalSummary[]>([]);
  let myRequests = $state<AppraisalRequest[]>([]);
  let notifications = $state<Notification[]>([]);
  let ergonomics = $state<ErgonomicAssessment[]>([]);
  let adjustments = $state<AdjustmentRequest[]>([]);
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

  <WellbeingPrompts
    {pid}
    prompts={wellbeing}
    surveys={pulseSurveys}
    promptsError={panelErrors["wellbeing"] ?? null}
    pulseError={panelErrors["pulse"] ?? null}
    onchange={load}
    onfail={(message) => (actionError = message)}
  />

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

  <Appraisals
    {pid}
    {appraisals}
    requests={myRequests}
    requestsError={panelErrors["requests"] ?? null}
    appraisalsError={panelErrors["appraisals"] ?? null}
    onchange={load}
    onfail={(message) => (actionError = message)}
  />

  <AdjustmentRequests
    {pid}
    requests={adjustments}
    error={panelErrors["adjustments"] ?? null}
    onchange={load}
    onfail={(message) => (actionError = message)}
  />

  <ErgonomicAssessments
    {pid}
    assessments={ergonomics}
    error={panelErrors["ergonomics"] ?? null}
    onchange={load}
    onfail={(message) => (actionError = message)}
  />

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
  <EquipmentPanel workerPid={worker.pid} organizationRef={worker.organization_ref} />
  <ExpenseClaims workerPid={worker.pid} />
  <EmergencyContacts workerPid={worker.pid} />
  <Backups workerPid={worker.pid} />
  <DottedLinePanel workerPid={worker.pid} />
  <GroupsPanel workerPid={worker.pid} />
  <TeamAspirations workerPid={worker.pid} />
{/if}
