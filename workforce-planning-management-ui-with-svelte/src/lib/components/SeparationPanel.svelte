<!--
  A leaver's separation record (WPM-R222): the last day and the reason from the leaver record, and the final pay date and pay summary
  reference HR adds. The reason affects a person's eligibility, so anyone but HR is refused and the panel is simply not there.
-->
<script lang="ts">
  import { ApiError } from "#lib/api/client.js";
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { getSeparationRecord, saveSeparationRecord, type SeparationRecord } from "#lib/api/wpm.js";
  import { formatDate } from "#lib/locale-format.js";
  import { t } from "#lib/i18n.svelte.js";

  let { movementPid }: { movementPid: string } = $props();

  let view = $state<SeparationRecord | null>(null);
  let error = $state<string | null>(null);
  let finalPay = $state("");
  let reference = $state("");

  function reasonWord(code: string): string {
    const words: Record<string, () => string> = {
      resignation: () => t("movements.reasonResignation"),
      redundancy: () => t("movements.reasonRedundancy"),
      retirement: () => t("movements.reasonRetirement"),
      end_of_contract: () => t("movements.reasonEndOfContract"),
      dismissal: () => t("movements.reasonDismissal"),
      other: () => t("movements.reasonOther"),
    };
    return words[code]?.() ?? code;
  }

  function take(found: SeparationRecord) {
    view = found;
    finalPay = found.final_pay_on ?? "";
    reference = found.pay_summary_ref ?? "";
  }

  const load = guardedLoad(
    () => getSeparationRecord(movementPid),
    take,
    (cause) => {
      if (!(cause instanceof ApiError && (cause.status === 403 || cause.status === 404))) error = errorText(cause);
    },
  );

  $effect(() => {
    void movementPid;
    void load();
  });

  async function save(event: SubmitEvent) {
    event.preventDefault();
    error = null;
    try {
      take(
        await saveSeparationRecord(movementPid, {
          final_pay_on: finalPay || null,
          pay_summary_ref: reference.trim() || null,
        }),
      );
    } catch (cause) {
      error = errorText(cause);
    }
  }
</script>

{#if error}<ErrorNote testid="separation-error" message={error} />{/if}
{#if view}
  <section class="panel" data-testid="separation-panel">
    <h2>{t("separation.title")}</h2>
    <p class="muted">{t("separation.hint")}</p>
    <p>{t("separation.lastDay")}: {formatDate(view.last_day)}</p>
    <p data-testid="separation-reason">{t("separation.reason")}: {view.reason_code ? reasonWord(view.reason_code) : "—"}</p>
    <form onsubmit={save}>
      <label>
        {t("separation.finalPay")}
        <input type="date" bind:value={finalPay} data-testid="separation-final-pay" />
      </label>
      <label>
        {t("separation.payRef")}
        <input bind:value={reference} maxlength="200" data-testid="separation-ref" />
      </label>
      <p class="muted">{t("separation.payRefHint")}</p>
      <button type="submit" data-testid="separation-save">{t("common.save")}</button>
    </form>
    {#if view.missing.length > 0}
      <p data-testid="separation-missing">
        {t("separation.missing")}:
        {view.missing
          .map((gap) => (gap === "final_pay_on" ? t("separation.finalPay") : t("separation.payRef")))
          .join(" · ")}
      </p>
    {:else}
      <p class="muted" data-testid="separation-complete">{t("separation.complete")}</p>
    {/if}
  </section>
{/if}

<style>
  form label {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    margin-bottom: 0.75rem;
    max-width: 100%;
  }
  form input {
    max-width: 100%;
    box-sizing: border-box;
  }
</style>
