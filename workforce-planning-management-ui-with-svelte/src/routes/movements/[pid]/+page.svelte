<!--
  One joiner / leaver record (`/movements/{pid}`): the dated checklist (done,
  skipped, overdue, due today, upcoming) and — for a leaver — the handover:
  everything they still hold as of their last day, each item reassigned or
  closed, with an audit trail. A leaver's record cannot be completed while
  checklist items are open or anything they hold is unassigned (the service
  says what is left).
-->
<script lang="ts">
  import { page } from "$app/state";
  import {
    addMovementItem,
    closeMovement,
    getMovement,
    handOverAll,
    handoverTrail,
    leaverHandover,
    listWorkers,
    movementItemAction,
    reassignHeld,
    skipMovementItem,
  } from "#lib/api/wpm.js";
  import type { HandoverAction, HeldItem, Movement, MovementItem } from "#lib/api/types.js";
  import { t } from "#lib/i18n.svelte.js";

  const pid = $derived(page.params.pid ?? "");

  let movement = $state<(Movement & { items: MovementItem[] }) | null>(null);
  let held = $state<HeldItem[]>([]);
  let remaining = $state(0);
  let trail = $state<HandoverAction[]>([]);
  let people = $state<Array<{ pid: string; display_name: string; organization_ref: string }>>([]);
  let error = $state<string | null>(null);
  let successor = $state("");
  let skipping = $state<string | null>(null);
  let skipReason = $state("");
  let newTitle = $state("");
  let newDue = $state("");
  // The chosen new holder per held item.
  let target = $state<Record<string, string>>({});

  const message = (cause: unknown) => (cause instanceof Error ? cause.message : String(cause));
  const colleagues = $derived(
    people.filter((p) => p.organization_ref === movement?.organization_ref && p.pid !== movement?.worker_pid),
  );

  async function load() {
    try {
      const m = await getMovement(pid);
      movement = m;
      people = await listWorkers();
      if (m.kind === "leaver") {
        const [h, tr] = await Promise.all([leaverHandover(pid), handoverTrail(pid)]);
        held = h.items;
        remaining = h.remaining;
        trail = tr;
      }
    } catch (cause) {
      error = message(cause);
    }
  }
  $effect(() => {
    void pid;
    void load();
  });

  async function run(action: () => Promise<unknown>) {
    error = null;
    try {
      await action();
      await load();
    } catch (cause) {
      error = message(cause);
    }
  }

  const stateLabel = (s: MovementItem["state"]) =>
    ({
      done: t("movements.stateDone"),
      skipped: t("movements.stateSkipped"),
      overdue: t("movements.stateOverdue"),
      due_today: t("movements.stateDueToday"),
      upcoming: t("movements.stateUpcoming"),
    })[s];
  const stateMark = (s: MovementItem["state"]) =>
    ({ done: "✓", skipped: "–", overdue: "▲", due_today: "●", upcoming: "○" })[s];

  const kindLabel = (k: string) =>
    ({
      direct_report: t("movements.kindDirectReport"),
      dotted_line: t("movements.kindDottedLine"),
      group_lead: t("movements.kindGroupLead"),
      rota_membership: t("movements.kindRotaMembership"),
      rota_swap: t("movements.kindRotaSwap"),
      backup: t("movements.kindBackup"),
      mentorship: t("movements.kindMentorship"),
      shift: t("movements.kindShift"),
      task: t("movements.kindTask"),
      access: t("movements.kindAccess"),
    })[k] ?? k;
</script>

<svelte:head><title>{t("nav.movements")} — WPM</title></svelte:head>

{#if error}<p class="error" data-testid="error">{t("common.error")}: {error}</p>{/if}

{#if movement}
  <h1>
    {movement.worker_name}
    <span class="chip">{movement.kind === "leaver" ? t("movements.kindLeaver") : t("movements.kindJoiner")}</span>
    <span class="chip" data-testid="movement-status">
      {movement.status === "open" ? t("movements.statusOpen") : movement.status === "completed" ? t("movements.statusCompleted") : t("movements.statusCancelled")}
    </span>
  </h1>
  <p class="muted">
    {movement.job_title}, {movement.department} ·
    {movement.kind === "leaver" ? t("movements.lastDay") : t("movements.startDay")}: {movement.effective_on}
    {#if movement.reason}· {movement.reason}{/if}
  </p>

  <h2>{t("movements.checklist")} ({movement.progress.closed}/{movement.progress.total})</h2>
  <table data-testid="movement-items">
    <thead>
      <tr>
        <th></th><th>{t("movements.itemTitle")}</th><th>{t("movements.dueOn")}</th><th>{t("movements.assignee")}</th><th></th>
      </tr>
    </thead>
    <tbody>
      {#each movement.items as item (item.pid)}
        <tr data-state={item.state}>
          <td title={stateLabel(item.state)}><span aria-hidden="true">{stateMark(item.state)}</span> <span class="muted">{stateLabel(item.state)}</span></td>
          <td>{item.title}{#if item.skipped_reason} <span class="muted">— {item.skipped_reason}</span>{/if}</td>
          <td>{item.due_on}</td>
          <td>{item.assignee_name ?? "—"}</td>
          <td>
            {#if movement.status === "open"}
              {#if item.state === "done" || item.state === "skipped"}
                <button type="button" onclick={() => void run(() => movementItemAction(item.pid, "reopen"))}>{t("movements.reopen")}</button>
              {:else if skipping === item.pid}
                <input bind:value={skipReason} placeholder={t("movements.skipReason")} data-testid="skip-reason" />
                <button
                  type="button"
                  onclick={() =>
                    void run(async () => {
                      await skipMovementItem(item.pid, skipReason);
                      skipping = null;
                      skipReason = "";
                    })}
                >{t("movements.skip")}</button>
              {:else}
                <button type="button" data-testid="item-done" onclick={() => void run(() => movementItemAction(item.pid, "done"))}>{t("movements.done")}</button>
                <button type="button" onclick={() => (skipping = item.pid)}>{t("movements.skip")}</button>
              {/if}
            {/if}
          </td>
        </tr>
      {/each}
    </tbody>
  </table>

  {#if movement.status === "open"}
    <form
      onsubmit={(event) => {
        event.preventDefault();
        void run(async () => {
          await addMovementItem(pid, { title: newTitle.trim(), due_on: newDue });
          newTitle = newDue = "";
        });
      }}
    >
      <label>{t("movements.itemTitle")} <input bind:value={newTitle} required maxlength="200" /></label>
      <label>{t("movements.dueOn")} <input type="date" bind:value={newDue} required /></label>
      <button type="submit">{t("movements.addItem")}</button>
    </form>
  {/if}

  {#if movement.kind === "leaver"}
    <h2>{t("movements.handover")}</h2>
    <p class="muted">{t("movements.handoverHint")}</p>
    {#if held.length === 0}
      <p class="muted" data-testid="held-none">{t("movements.nothingHeld")}</p>
    {:else}
      <table data-testid="held-items">
        <tbody>
          {#each held as h (h.kind + h.subject_pid)}
            <tr>
              <td><span class="chip">{kindLabel(h.kind)}</span></td>
              <td>{h.label}</td>
              <td>
                {#if movement.status === "open"}
                  {#if h.can_reassign}
                    <select bind:value={target[h.subject_pid]} aria-label={t("movements.assignTo")}>
                      <option value="">{t("movements.assignTo")}…</option>
                      {#each colleagues as c (c.pid)}<option value={c.pid}>{c.display_name}</option>{/each}
                    </select>
                    <button
                      type="button"
                      disabled={!target[h.subject_pid]}
                      onclick={() =>
                        void run(() =>
                          reassignHeld(pid, { kind: h.kind, subject_pid: h.subject_pid, to_worker_pid: target[h.subject_pid] }),
                        )}
                    >{t("movements.reassign")}</button>
                  {/if}
                  {#if !h.needs_new_holder}
                    <button type="button" onclick={() => void run(() => reassignHeld(pid, { kind: h.kind, subject_pid: h.subject_pid }))}>
                      {h.kind === "access" ? t("movements.revoke") : t("movements.close")}
                    </button>
                  {/if}
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
      {#if movement.status === "open"}
        <p>
          <select bind:value={successor} data-testid="hand-all-to" aria-label={t("movements.handAll")}>
            <option value="">{t("movements.handAll")}…</option>
            {#each colleagues as c (c.pid)}<option value={c.pid}>{c.display_name}</option>{/each}
          </select>
          <button type="button" disabled={!successor} data-testid="hand-all" onclick={() => void run(() => handOverAll(pid, successor))}>
            {t("movements.handAll")}
          </button>
        </p>
      {/if}
    {/if}

    {#if trail.length > 0}
      <h3>{t("movements.trail")}</h3>
      <ul data-testid="handover-trail">
        {#each trail as a (a.performed_at + a.subject_pid)}
          <li>
            <span class="chip">{kindLabel(a.kind)}</span> {a.label}
            — {a.action}{#if a.to_worker_name} → {a.to_worker_name}{/if}
            <span class="muted">· {a.performed_at.slice(0, 16).replace("T", " ")}</span>
          </li>
        {/each}
      </ul>
    {/if}
  {/if}

  {#if movement.status === "open"}
    <p>
      <button type="button" data-testid="movement-complete" onclick={() => void run(() => closeMovement(pid, "complete"))}>{t("movements.complete")}</button>
      <button type="button" onclick={() => void run(() => closeMovement(pid, "cancel"))}>{t("movements.cancel")}</button>
    </p>
  {/if}
{:else if !error}
  <p>{t("common.loading")}</p>
{/if}
