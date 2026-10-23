<!--
  My contact details (WPM-R124): home address, telephone numbers and personal e-mail, written by the worker.
  Replaced as a whole; HR and payroll can read them, a manager cannot; the previous value is not kept.
-->
<script lang="ts">
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import {
    clearMyContactDetails,
    getMyContactDetails,
    setMyContactDetails,
    type MyContactDetails,
  } from "#lib/api/wpm.js";
  import { t } from "#lib/i18n.svelte.js";

  const FIELDS = [
    "address_line1",
    "address_line2",
    "city",
    "region",
    "postcode",
    "country",
    "phone_mobile",
    "phone_home",
    "personal_email",
  ] as const;
  type Field = (typeof FIELDS)[number];

  let form = $state<Record<Field, string>>(blank());
  let held = $state(false);
  let saved = $state(false);
  let error = $state<string | null>(null);

  function blank(): Record<Field, string> {
    return Object.fromEntries(FIELDS.map((f) => [f, ""])) as Record<Field, string>;
  }
  const message = (cause: unknown) => (errorText(cause));

  function fill(details: MyContactDetails | null) {
    held = details !== null;
    form = blank();
    if (details) for (const f of FIELDS) form[f] = details[f] ?? "";
  }

  $effect(() => {
    void (async () => {
      try {
        fill((await getMyContactDetails()).contact_details);
      } catch (cause) {
        error = message(cause);
      }
    })();
  });

  async function save() {
    error = null;
    saved = false;
    try {
      fill((await setMyContactDetails({ ...form })).contact_details);
      saved = true;
    } catch (cause) {
      error = message(cause);
    }
  }

  async function clear() {
    error = null;
    saved = false;
    try {
      await clearMyContactDetails();
      fill(null);
    } catch (cause) {
      error = message(cause);
    }
  }
</script>

<section class="panel" data-testid="my-contact-details">
  <h2>{t("details.contact.title")}</h2>
  <p class="muted">{t("details.contact.hint")}</p>
  {#if error}<ErrorNote testid="contact-error" message={error} />{/if}
  <form
    onsubmit={(event) => {
      event.preventDefault();
      void save();
    }}
  >
    {#each FIELDS as f (f)}
      <label>
        {t(`details.contact.${f}`)}
        <input
          data-testid="contact-{f}"
          type={f === "personal_email" ? "email" : f.startsWith("phone") ? "tel" : "text"}
          autocomplete="off"
          bind:value={form[f]}
        />
      </label>
    {/each}
    <button type="submit" data-testid="contact-save">{t("common.save")}</button>
    {#if held}
      <button type="button" data-testid="contact-clear" onclick={() => void clear()}>{t("details.contact.clear")}</button>
    {/if}
    {#if saved}<span class="muted" role="status" data-testid="contact-saved">{t("details.saved")}</span>{/if}
  </form>
</section>
