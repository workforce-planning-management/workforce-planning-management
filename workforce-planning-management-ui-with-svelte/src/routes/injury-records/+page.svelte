<!--
  Injury and illness records (`/injury-records`, WPM-T421): HR records an incident and each report it owes, and sees where each report
  stands against the times HR entered. A row proves a report and points to the statutory record, which stays outside WPM (WPM-D179). So
  the page has no description box, and a reference takes no spaces. It decides no duty and files nothing. The service decides who may.
-->
<script lang="ts">
  import { page } from "$app/state";
  import { attempt, errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { formatDate, formatDateTime } from "#lib/locale-format.js";
  import { t } from "#lib/i18n.svelte.js";
  import {
    addInjuryRecord,
    addInjuryReport,
    listInjuryRecords,
    listWorkers,
    markInjuryReportSent,
  } from "#lib/api/wpm.js";
  import type { InjuryRecord, InjuryReport, Worker } from "#lib/api/types.js";

  const KINDS = ["accident", "illness", "dangerous_occurrence"] as const;
  const CLASSES = ["no_injury", "no_absence", "absence", "serious", "fatal"] as const;
  const RECIPIENTS = ["safety_authority", "labour_inspectorate", "insurer", "social_insurance", "health_authority"] as const;
  const METHODS = ["online", "phone", "form"] as const;
  const REFERENCE = "[A-Za-z0-9_.\\/:#\\-]+";

  const organizations = $derived((page.data.scope ?? []) as string[]);
  let organization = $state("");
  let records = $state<InjuryRecord[] | null>(null);
  let people = $state<Worker[]>([]);
  let error = $state<string | null>(null);
  let message = $state<string | null>(null);
  let saved = $state(false);
  // Bumped after HR changes something, so the list is read again.
  let reads = $state(0);

  let jurisdiction = $state("");
  let kind = $state("accident");
  let worker = $state("");
  let occurredAt = $state("");
  let learnedAt = $state("");
  let incidentClass = $state("absence");
  let days = $state("");
  let affected = $state("1");
  let recordRef = $state("");
  let keepUntil = $state("");

  // One small form per incident and per report, kept by pid.
  let recipient = $state<Record<string, string>>({});
  let dueAt = $state<Record<string, string>>({});
  let sentAt = $state<Record<string, string>>({});
  let method = $state<Record<string, string>>({});
  let reference = $state<Record<string, string>>({});

  $effect(() => {
    const chosen = organization;
    void reads;
    error = null;
    if (!chosen) {
      records = null;
      return;
    }
    let stale = false;
    void (async () => {
      try {
        const [found, staff] = await Promise.all([listInjuryRecords(chosen), listWorkers()]);
        if (stale) return;
        records = found;
        people = staff.filter((w) => w.organization_ref === chosen);
      } catch (cause) {
        if (!stale) error = errorText(cause);
      }
    })();
    return () => {
      stale = true;
    };
  });

  const instant = (local: string) => (local ? new Date(local).toISOString() : undefined);
  const count = (text: string) => {
    const n = Number.parseInt(text, 10);
    return Number.isFinite(n) ? n : undefined;
  };

  async function done(action: () => Promise<unknown>) {
    saved = false;
    const failed = await attempt(action);
    message = failed?.message ?? null;
    if (failed === null) {
      saved = true;
      reads += 1;
    }
    return failed === null;
  }

  async function add(event: SubmitEvent) {
    event.preventDefault();
    const ok = await done(() =>
      addInjuryRecord({
        organization,
        jurisdiction: jurisdiction.trim(),
        kind,
        class: incidentClass,
        worker_pid: worker || undefined,
        occurred_at: instant(occurredAt) ?? "",
        learned_at: instant(learnedAt),
        absence_days: count(days),
        people_affected: count(affected),
        statutory_record_ref: recordRef.trim() || undefined,
        keep_until: keepUntil || undefined,
      }),
    );
    if (ok) {
      jurisdiction = worker = occurredAt = learnedAt = days = recordRef = keepUntil = "";
      affected = "1";
    }
  }

  function addReport(event: SubmitEvent, pid: string) {
    event.preventDefault();
    void done(() =>
      addInjuryReport(pid, {
        recipient: recipient[pid] ?? "safety_authority",
        due_at: instant(dueAt[pid] ?? ""),
      }),
    );
  }

  function markSent(event: SubmitEvent, report: InjuryReport) {
    event.preventDefault();
    void done(() =>
      markInjuryReportSent(report.pid, {
        sent_at: instant(sentAt[report.pid] ?? "") ?? "",
        method: method[report.pid] ?? "online",
        reference: reference[report.pid]?.trim() || undefined,
      }),
    );
  }
</script>

<svelte:head><title>{page.data.title}</title></svelte:head>

<h1>{t("injuryRecords.title")}</h1>
<p class="muted">{t("injuryRecords.hint")}</p>
<label>
  {t("injuryRecords.organization")}
  <select bind:value={organization} data-testid="injury-organization">
    <option value=""></option>
    {#each organizations as o (o)}<option value={o}>{o}</option>{/each}
  </select>
</label>
{#if error}<ErrorNote testid="error" prefixed message={error} />{/if}

{#if organization}
  {#if records}
    {#if records.length === 0}
      <p data-testid="injury-none">{t("injuryRecords.none")}</p>
    {:else}
      {#each records as r (r.pid)}
        <section class="panel" data-testid="injury-row">
          <h2>{formatDateTime(r.occurred_at)} · {r.jurisdiction}</h2>
          <dl>
            <dt>{t("injuryRecords.kind")}</dt>
            <dd>{t(`injuryRecords.${r.kind}` as "injuryRecords.accident")}</dd>
            <dt>{t("injuryRecords.class")}</dt>
            <dd data-testid="injury-class">{t(`injuryRecords.${r.class}` as "injuryRecords.absence")}</dd>
            <dt>{t("injuryRecords.worker")}</dt>
            <dd>{r.worker_name ?? "—"}</dd>
            <dt>{t("injuryRecords.learnedAt")}</dt>
            <dd>{formatDateTime(r.learned_at)}</dd>
            <dt>{t("injuryRecords.absenceDays")}</dt>
            <dd>{r.absence_days ?? "—"}</dd>
            <dt>{t("injuryRecords.people")}</dt>
            <dd>{r.people_affected}</dd>
            <dt>{t("injuryRecords.recordRef")}</dt>
            <dd data-testid="injury-ref">{r.statutory_record_ref ?? t("injuryRecords.noRecordRef")}</dd>
            <dt>{t("injuryRecords.keepUntil")}</dt>
            <dd>{r.keep_until ? formatDate(r.keep_until) : t("injuryRecords.keepUnknown")}</dd>
          </dl>
          <h3>{t("injuryRecords.reports")}</h3>
          {#if r.reports.length === 0}
            <p class="muted">{t("injuryRecords.noReports")}</p>
          {:else}
            <ul>
              {#each r.reports as report (report.pid)}
                <li data-testid="injury-report">
                  {t(`injuryRecords.${report.recipient}` as "injuryRecords.insurer")}:
                  <strong data-testid="injury-standing">{t(`injuryRecords.${report.standing}` as "injuryRecords.waiting")}</strong>
                  {#if report.due_at}· {t("injuryRecords.dueAt")} {formatDateTime(report.due_at)}{/if}
                  {#if report.sent_at}
                    · {t("injuryRecords.sentAt")} {formatDateTime(report.sent_at)}
                    {#if report.reference}· {report.reference}{/if}
                  {:else}
                    <form class="stack" onsubmit={(event) => markSent(event, report)}>
                      <label>{t("injuryRecords.sentAt")}
                        <input type="datetime-local" required bind:value={sentAt[report.pid]} data-testid="injury-sent-at" />
                      </label>
                      <label>{t("injuryRecords.method")}
                        <select bind:value={method[report.pid]} data-testid="injury-method">
                          {#each METHODS as m (m)}<option value={m}>{t(`injuryRecords.${m}`)}</option>{/each}
                        </select>
                      </label>
                      <label>{t("injuryRecords.reference")}
                        <input bind:value={reference[report.pid]} pattern={REFERENCE} maxlength="200" data-testid="injury-reference" />
                      </label>
                      <button type="submit" data-testid="injury-mark-sent">{t("injuryRecords.markSent")}</button>
                    </form>
                  {/if}
                </li>
              {/each}
            </ul>
          {/if}
          <form class="stack" onsubmit={(event) => addReport(event, r.pid)}>
            <label>{t("injuryRecords.recipient")}
              <select bind:value={recipient[r.pid]} data-testid="injury-recipient">
                {#each RECIPIENTS as to (to)}<option value={to}>{t(`injuryRecords.${to}`)}</option>{/each}
              </select>
            </label>
            <label>{t("injuryRecords.dueAt")}
              <input type="datetime-local" bind:value={dueAt[r.pid]} data-testid="injury-due-at" />
            </label>
            <button type="submit" data-testid="injury-add-report">{t("injuryRecords.addReport")}</button>
          </form>
        </section>
      {/each}
    {/if}
  {/if}

  <section class="panel" data-testid="injury-admin">
    <h2>{t("injuryRecords.addTitle")}</h2>
    <p class="muted">{t("injuryRecords.addHint")}</p>
    <form class="stack" onsubmit={add}>
      <label>{t("injuryRecords.jurisdiction")}
        <input bind:value={jurisdiction} required maxlength="6" data-testid="inj-jurisdiction" />
      </label>
      <label>{t("injuryRecords.kind")}
        <select bind:value={kind} data-testid="inj-kind">
          {#each KINDS as k (k)}<option value={k}>{t(`injuryRecords.${k}`)}</option>{/each}
        </select>
      </label>
      <label>{t("injuryRecords.class")}
        <select bind:value={incidentClass} data-testid="inj-class">
          {#each CLASSES as c (c)}<option value={c}>{t(`injuryRecords.${c}`)}</option>{/each}
        </select>
      </label>
      <label>{t("injuryRecords.worker")}
        <select bind:value={worker} data-testid="inj-worker">
          <option value="">—</option>
          {#each people as w (w.pid)}<option value={w.pid}>{w.display_name}</option>{/each}
        </select>
      </label>
      <label>{t("injuryRecords.occurredAt")}
        <input type="datetime-local" required bind:value={occurredAt} data-testid="inj-occurred" />
      </label>
      <label>{t("injuryRecords.learnedAt")}
        <input type="datetime-local" bind:value={learnedAt} data-testid="inj-learned" />
      </label>
      <label>{t("injuryRecords.absenceDays")}
        <input bind:value={days} inputmode="numeric" data-testid="inj-days" />
      </label>
      <label>{t("injuryRecords.people")}
        <input bind:value={affected} inputmode="numeric" data-testid="inj-people" />
      </label>
      <label>{t("injuryRecords.recordRef")}
        <input bind:value={recordRef} pattern={REFERENCE} maxlength="200" data-testid="inj-ref" />
      </label>
      <label>{t("injuryRecords.keepUntil")}
        <input type="date" bind:value={keepUntil} data-testid="inj-keep" />
      </label>
      <button type="submit" data-testid="inj-save">{t("injuryRecords.save")}</button>
    </form>
  </section>
  {#if message}
    <ErrorNote testid="inj-error" message={message} />
  {:else if saved}
    <p role="status" data-testid="inj-saved">{t("injuryRecords.saved")}</p>
  {/if}
{/if}

<style>
  .stack {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem 1rem;
    align-items: flex-end;
    margin-top: 0.75rem;
  }
  .stack label {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    min-width: 0;
    max-width: 100%;
  }
  .stack input,
  .stack select {
    max-width: 100%;
  }
</style>
