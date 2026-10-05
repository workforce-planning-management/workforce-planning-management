<!--
  Backups: the colleague(s) who cover for the worker when they are out sick
  or on leave, in the order to ask, optionally for a dated window. Visible
  to anyone who can see the worker; the worker and HR change them. Also
  says who is covering today, or that nobody can — never a guess.
-->
<script lang="ts">
  import {
    addBackup,
    listBackups,
    listWorkers,
    removeBackup,
    workerCover,
  } from "#lib/api/wpm.js";
  import type { Backup, Cover } from "#lib/api/types.js";
  import { t } from "#lib/i18n.svelte.js";

  let { workerPid }: { workerPid: string } = $props();

  let backups = $state<Backup[]>([]);
  let cover = $state<Cover | null>(null);
  let everyone = $state<Array<{ pid: string; display_name: string }>>([]);
  let error = $state<string | null>(null);
  let choice = $state("");
  let from = $state("");
  let until = $state("");
  let note = $state("");

  const message = (cause: unknown) => (cause instanceof Error ? cause.message : String(cause));
  const candidates = $derived(
    everyone.filter((w) => w.pid !== workerPid && !backups.some((b) => b.backup_pid === w.pid)),
  );

  async function load() {
    try {
      const [rows, today, workers] = await Promise.all([
        listBackups(workerPid),
        workerCover(workerPid),
        listWorkers(),
      ]);
      backups = rows;
      cover = today;
      everyone = workers;
    } catch (cause) {
      error = message(cause);
    }
  }

  $effect(() => {
    void workerPid;
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
</script>

<section class="panel" data-testid="backups">
  <h2>{t("backups.title")}</h2>
  <p class="muted">{t("backups.hint")}</p>
  {#if error}<p class="error" data-testid="error">{error}</p>{/if}
  {#if cover && backups.length > 0}
    <p data-testid="cover-today">
      {#if cover.covered_by_name}
        {t("backups.coverToday")} <strong>{cover.covered_by_name}</strong>
      {:else}
        <span class="muted">{t("backups.nobodyToday")}</span>
      {/if}
    </p>
  {/if}
  <ol data-testid="backup-list">
    {#each backups as b (b.pid)}
      <li>
        <strong>{b.backup_name ?? b.backup_pid}</strong>
        {#if b.backup_title}<span class="muted">— {b.backup_title}</span>{/if}
        {#if b.starts_on || b.ends_on}
          <span class="muted">({b.starts_on ?? "…"} → {b.ends_on ?? "…"})</span>
        {/if}
        {#if b.note}<span class="muted">— {b.note}</span>{/if}
        <button type="button" onclick={() => void run(() => removeBackup(b.pid))}>
          {t("backups.remove")}
        </button>
      </li>
    {:else}
      <li class="muted">{t("backups.none")}</li>
    {/each}
  </ol>
  <form
    onsubmit={(event) => {
      event.preventDefault();
      void run(async () => {
        await addBackup(workerPid, {
          backup_pid: choice,
          ...(from ? { starts_on: from } : {}),
          ...(until ? { ends_on: until } : {}),
          ...(note.trim() ? { note: note.trim() } : {}),
        });
        choice = from = until = note = "";
      });
    }}
  >
    <label>
      {t("backups.add")}
      <select bind:value={choice} required data-testid="backup-choice">
        <option value="" disabled>{t("backups.choose")}</option>
        {#each candidates as w (w.pid)}<option value={w.pid}>{w.display_name}</option>{/each}
      </select>
    </label>
    <label>{t("backups.from")} <input type="date" bind:value={from} /></label>
    <label>{t("backups.until")} <input type="date" bind:value={until} /></label>
    <label>{t("backups.note")} <input bind:value={note} maxlength="500" /></label>
    <button type="submit" data-testid="backup-add">{t("backups.add")}</button>
  </form>
</section>
