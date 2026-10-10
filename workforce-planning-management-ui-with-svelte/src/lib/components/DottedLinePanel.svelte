<!--
  Dotted-line reporting: a secondary (usually functional or project) line
  alongside the solid-line manager. A person can have several dotted-line
  managers, and each can be anyone. It changes nothing in the org chart and
  grants no extra access. Ending one keeps it as history.
-->
<script lang="ts">
  import { t } from "#lib/i18n.svelte.js";
  import {
    addDottedManager,
    dottedLine,
    endDottedManager,
    listWorkers,
    type DottedLink,
  } from "#lib/api/wpm.js";

  let { workerPid }: { workerPid: string } = $props();

  let managers = $state<DottedLink[]>([]);
  let reports = $state<DottedLink[]>([]);
  let everyone = $state<Array<{ pid: string; display_name: string }>>([]);
  let error = $state<string | null>(null);
  let choice = $state("");
  let note = $state("");

  const message = (cause: unknown) => (cause instanceof Error ? cause.message : String(cause));
  const candidates = $derived(
    everyone.filter((w) => w.pid !== workerPid && !managers.some((m) => m.pid === w.pid)),
  );

  async function load() {
    try {
      const [line, workers] = await Promise.all([dottedLine(workerPid), listWorkers()]);
      managers = line.dotted_line_managers;
      reports = line.dotted_line_reports;
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

<section class="panel" data-testid="dotted-line">
  <h2>{t("comp.dottedLinePanel.dotted_line_reporting")}</h2>
  {#if error}<p class="error" data-testid="error">{error}</p>{/if}
  <h3>{t("comp.dottedLinePanel.dotted_line_managers")}</h3>
  <ul data-testid="dotted-managers">
    {#each managers as m (m.pid)}
      <li>
        {m.display_name}
        {#if m.note}<span class="muted">— {m.note}</span>{/if}
        {#if m.on_behalf}<span class="chip">{t("comp.dottedLinePanel.set_on_their_behalf")}</span>{/if}
        <button type="button" onclick={() => void run(() => endDottedManager(workerPid, m.pid))}>{t("comp.dottedLinePanel.end")}</button>
      </li>
    {:else}
      <li class="muted">{t("comp.dottedLinePanel.none")}</li>
    {/each}
  </ul>
  <form
    onsubmit={(event) => {
      event.preventDefault();
      void run(async () => {
        await addDottedManager(workerPid, choice, note.trim() || undefined);
        choice = "";
        note = "";
      });
    }}
  >
    <label>
      {t("comp.dottedLinePanel.add_a_dotted_line_manager")}
      <select bind:value={choice} required>
        <option value="" disabled>{t("comp.dottedLinePanel.anyone")}</option>
        {#each candidates as w (w.pid)}<option value={w.pid}>{w.display_name}</option>{/each}
      </select>
    </label>
    <label>{t("comp.dottedLinePanel.why")} <input bind:value={note} maxlength="500" placeholder={t("comp.dottedLinePanel.project_function")} /></label>
    <button type="submit">{t("comp.dottedLinePanel.add")}</button>
  </form>
  <h3>{t("comp.dottedLinePanel.dotted_line_reports")}</h3>
  <ul data-testid="dotted-reports">
    {#each reports as r (r.pid)}
      <li>{r.display_name}{#if r.note} <span class="muted">— {r.note}</span>{/if}</li>
    {:else}
      <li class="muted">{t("comp.dottedLinePanel.none")}</li>
    {/each}
  </ul>
</section>
