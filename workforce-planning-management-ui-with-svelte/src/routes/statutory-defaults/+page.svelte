<!--
  Statutory defaults (`/statutory-defaults`, WPM-T403): a read-only view of `GET /api/statutory-defaults`. For each
  `WPM_*` setting whose default is one regime's value, or a starting point that is no regime's: the value in force,
  whether this deployment set it, the default, and whose the default is (a jurisdiction code, what it rests on, and
  whether that source was read). Configuration, not personal data: anyone signed in reads it. Nothing here changes a
  setting; the deployer does that in the service's configuration. What a default rests on is the service's English text.
-->
<script lang="ts">
  import { formatDate, formatNumber, isCivilDate } from "#lib/locale-format.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import DataTable from "#lib/components/DataTable.svelte";
  import { getStatutoryDefaults } from "#lib/api/wpm.js";
  import type { StatutoryDefault } from "#lib/api/wpm.js";
  import { t, tf, tp } from "#lib/i18n.svelte.js";
  import type {
    DataGridColumn,
    DataGridRow,
  } from "@lilydesignsystem/svelte-data-grid";

  let settings = $state<StatutoryDefault[] | null>(null);
  let error = $state<string | null>(null);

  $effect(() => {
    let stale = false;
    void (async () => {
      try {
        const found = await getStatutoryDefaults();
        if (!stale) {
          settings = found.settings;
          error = null;
        }
      } catch (cause) {
        if (!stale) error = errorText(cause);
      }
    })();
    return () => {
      stale = true;
    };
  });

  type Tier = { from_workers: number; every_years: number; first_report_on: string };
  const isTier = (v: unknown): v is Tier =>
    typeof v === "object" && v !== null && "from_workers" in v && "every_years" in v && "first_report_on" in v;

  /** The value in force as text in the page's language; an unreadable value is "—", never a guess. */
  function shown(value: unknown): string {
    if (value === null || value === undefined) return "—";
    if (typeof value === "boolean") return value ? t("statutoryDefaults.on") : t("statutoryDefaults.off");
    if (typeof value === "number") return formatNumber(value);
    if (typeof value === "string") return isCivilDate(value) ? formatDate(value) : value;
    if (Array.isArray(value)) {
      if (value.length === 0) return t("statutoryDefaults.none");
      return value
        .map((item) =>
          isTier(item)
            ? tf("statutoryDefaults.tier", {
                workers: item.from_workers,
                years: item.every_years,
                date: formatDate(item.first_report_on),
              })
            : typeof item === "string"
              ? item
              : "—",
        )
        .join("; ");
    }
    return "—";
  }

  const rows = $derived(
    (settings ?? []).map((s) => ({
      setting: s.setting,
      area: tp(`statutoryDefaults.areas.${s.area}`) ?? s.area,
      value: shown(s.value),
      set: s.set ? t("statutoryDefaults.set_here") : t("statutoryDefaults.default_used"),
      default: s.default,
      regime: s.default_from.regime ?? t("statutoryDefaults.no_regime"),
      status: tp(`statutoryDefaults.statuses.${s.default_from.status}`) ?? s.default_from.status,
      reference: s.default_from.reference,
    })),
  );

  const columns = $derived<DataGridColumn[]>([
    { id: "setting", header: t("statutoryDefaults.setting"), sortable: true, rowHeader: true },
    { id: "area", header: t("statutoryDefaults.area"), sortable: true },
    { id: "value", header: t("statutoryDefaults.value") },
    // Whose the default is comes before the long texts, so it is on screen without scrolling the table.
    { id: "regime", header: t("statutoryDefaults.regime"), sortable: true },
    { id: "status", header: t("statutoryDefaults.status"), sortable: true },
    { id: "set", header: t("statutoryDefaults.set"), sortable: true },
    { id: "default", header: t("statutoryDefaults.default") },
    { id: "reference", header: t("statutoryDefaults.reference") },
  ]);
</script>

<svelte:head>
  <title>{tf("pages.statutoryDefaults.page_title", { statutoryDefaults: t("nav.statutoryDefaults") })}</title>
</svelte:head>

<h1>{t("nav.statutoryDefaults")}</h1>
<p class="muted">{t("statutoryDefaults.hint")}</p>
{#if error}<ErrorNote testid="error" prefixed message={error} />{/if}

{#if settings}
  <p class="muted">{t("statutoryDefaults.reference_note")}</p>
  <DataTable
    testid="defaults-table"
    label={t("nav.statutoryDefaults")}
    columns={columns}
    rows={rows as unknown as DataGridRow[]}
    rowId={(row) => String(row.setting)}
    rowAttrs={(row) => ({ "data-testid": `default-${row.setting}` })}
  />
{:else if !error}
  <p>{t("common.loading")}</p>
{/if}
