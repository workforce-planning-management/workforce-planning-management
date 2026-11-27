<!--
  Grade a seat (WPM-R255): the band and pay point a position is graded at, a reference to a published pay scale and never
  an amount. The service checks the scale, the band and that a pay point is a step of the band; a refusal is described in
  the reader's language. Clearing makes the grade unknown again.
-->
<script lang="ts">
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { clearPositionGrade, setPositionGrade, type Position } from "#lib/api/wpm.js";
  import { t } from "#lib/i18n.svelte.js";

  let { positions, onchange }: { positions: Position[]; onchange: () => void } = $props();

  const open = $derived(positions.filter((p) => p.state !== "abolished"));
  let seat = $state("");
  let scale = $state("national-2026-27");
  let band = $state("");
  let payPoint = $state<number | null>(null);
  let error = $state<string | null>(null);
  $effect(() => {
    if (!seat || !open.some((p) => p.pid === seat)) seat = open[0]?.pid ?? "";
  });

  async function save(event: SubmitEvent) {
    event.preventDefault();
    error = null;
    try {
      await setPositionGrade(seat, {
        scale: scale.trim(),
        band: band.trim(),
        ...(payPoint ? { pay_point: payPoint } : {}),
      });
      onchange();
    } catch (cause) {
      error = errorText(cause);
    }
  }

  async function clear() {
    error = null;
    try {
      await clearPositionGrade(seat);
      onchange();
    } catch (cause) {
      error = errorText(cause);
    }
  }
</script>

{#if open.length > 0}
  <section>
    <h2>{t("pages.positions.grade_a_seat")}</h2>
    {#if error}<ErrorNote testid="grade-error" message={error} />{/if}
    <form onsubmit={save} data-testid="grade-form">
      <label>
        {t("pages.positions.seat")}
        <select bind:value={seat} data-testid="grade-seat">
          {#each open as p (p.pid)}<option value={p.pid}>{p.cost_centre} · {p.number ?? p.title}</option>{/each}
        </select>
      </label>
      <label>{t("pages.positions.scale")} <input bind:value={scale} required data-testid="grade-scale" /></label>
      <label>{t("pages.positions.band")} <input bind:value={band} required maxlength="8" data-testid="grade-band" /></label>
      <label>{t("pages.positions.pay_point")} <input type="number" min="1" bind:value={payPoint} data-testid="grade-pay-point" /></label>
      <button type="submit" data-testid="grade-save">{t("pages.positions.set_grade")}</button>
      <button type="button" onclick={() => void clear()} data-testid="grade-clear">{t("pages.positions.clear_grade")}</button>
    </form>
  </section>
{/if}
