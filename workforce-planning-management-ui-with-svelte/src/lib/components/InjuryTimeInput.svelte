<script lang="ts">
  import { t } from "#lib/i18n.svelte.js";
  import { resolveInjuryTime } from "#lib/injury-time.js";

  const {
    label,
    name,
    testid,
    zoneList,
    initialZone,
    required = false,
  }: {
    label: string;
    name: string;
    testid: string;
    zoneList: string;
    initialZone: string;
    required?: boolean;
  } = $props();

  const id = $props.id();
  let local = $state("");
  let zone = $state<string>();
  let chosen = $state("");
  let dateInput: HTMLInputElement;
  let zoneInput: HTMLInputElement;

  const shownZone = $derived(zone ?? initialZone);
  const resolution = $derived(resolveInjuryTime(local, shownZone));
  const instant = $derived.by(() => {
    if (resolution.kind === "empty") return "";
    if (resolution.kind === "unique") return resolution.candidate.instant;
    if (resolution.kind === "repeated") {
      return resolution.candidates.find(
        (candidate) => candidate.instant === chosen,
      )?.instant;
    }
    return undefined;
  });

  const zoneError = $derived(
    resolution.kind === "invalidZone" ? t("injuryRecords.invalidZone") : "",
  );
  const dateError = $derived(
    resolution.kind === "skipped"
      ? t("injuryRecords.skippedTime")
      : resolution.kind === "invalidTime"
        ? t("injuryRecords.invalidTime")
        : "",
  );

  $effect(() => {
    dateInput.setCustomValidity(dateError);
    zoneInput.setCustomValidity(zoneError);
  });
</script>

<fieldset>
  <legend>{label}</legend>
  <input
    aria-label={label}
    aria-invalid={dateError ? "true" : undefined}
    aria-describedby={dateError ? `${id}-error` : undefined}
    type="datetime-local"
    {required}
    bind:this={dateInput}
    bind:value={local}
    oninput={() => (chosen = "")}
    data-testid={testid}
  />
  <label>
    {t("injuryRecords.timeZone")}
    <input
      dir="ltr"
      list={zoneList}
      value={shownZone}
      autocomplete="off"
      spellcheck={false}
      aria-invalid={zoneError ? "true" : undefined}
      aria-describedby={zoneError ? `${id}-error` : undefined}
      bind:this={zoneInput}
      oninput={(event) => {
        zone = event.currentTarget.value;
        chosen = "";
      }}
      data-testid={`${testid}-zone`}
    />
  </label>
  {#if resolution.kind === "repeated"}
    <label>
      {t("injuryRecords.repeatedTime")}
      <select required bind:value={chosen} data-testid={`${testid}-offset`}>
        <option value="">{t("injuryRecords.chooseOffset")}</option>
        {#each resolution.candidates as candidate (candidate.instant)}
          <option dir="ltr" value={candidate.instant}
            >{candidate.offset}</option
          >
        {/each}
      </select>
    </label>
  {/if}
  <span id={`${id}-error`} role="status">{dateError || zoneError}</span>
  {#if instant !== undefined}
    <input type="hidden" {name} value={instant} />
  {/if}
</fieldset>

<style>
  fieldset {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    min-inline-size: 0;
    max-inline-size: 100%;
  }
  label {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  input,
  select {
    max-inline-size: 100%;
  }
  [role="status"]:empty {
    position: absolute;
  }
</style>
