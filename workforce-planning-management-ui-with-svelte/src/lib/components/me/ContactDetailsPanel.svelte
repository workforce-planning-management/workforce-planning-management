<!--
  My contact details (WPM-R124): home address, telephone numbers and personal e-mail, written by the worker.
  Replaced as a whole; HR and payroll can read them, a manager cannot; the previous value is not kept.
  The address follows the chosen country's order and labels (WPM-D152, `#lib/address-formats`); for a country the
  table lacks it is one free-text block in the person's own order. A field that holds a value is never hidden.
-->
<script lang="ts">
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import {
    clearMyContactDetails,
    getMyContactDetails,
    getMyCountries,
    setMyContactDetails,
    type MyContactDetails,
  } from "#lib/api/wpm.js";
  import { i18n, t, tp } from "#lib/i18n.svelte.js";
  import { countryOptions } from "#lib/countries.js";
  import { addressLayout } from "#lib/address-formats.js";

  const FIELDS = [
    "address_line1",
    "address_line2",
    "city",
    "region",
    "postcode",
    "address_text",
    "country",
    "phone_mobile",
    "phone_home",
    "personal_email",
  ] as const;
  type Field = (typeof FIELDS)[number];
  const OTHER = ["phone_mobile", "phone_home", "personal_email"] as const;

  let form = $state<Record<Field, string>>(blank());
  const layout = $derived(addressLayout(form.country, form));
  // The organization's added codes (WPM-D152); a failed read leaves the ISO 3166-1 list alone.
  let added = $state<Array<{ code: string; name: string | null }>>([]);
  const countries = $derived(countryOptions(i18n.locale, added, form.country));
  let held = $state(false);
  // What the service stored beside each number (WPM-D152): shown for the saved value, not while typing.
  let stored = $state<MyContactDetails | null>(null);
  let saved = $state(false);
  let error = $state<string | null>(null);

  function blank(): Record<Field, string> {
    return Object.fromEntries(FIELDS.map((f) => [f, ""])) as Record<Field, string>;
  }
  const message = (cause: unknown) => (errorText(cause));

  function fill(details: MyContactDetails | null) {
    held = details !== null;
    stored = details;
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

  $effect(() => {
    let stale = false;
    void getMyCountries()
      .then((r) => {
        if (!stale) added = r.deployer_codes;
      })
      .catch(() => {});
    return () => {
      stale = true;
    };
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
    <label>
      {t("details.contact.country")}
      <select data-testid="contact-country" bind:value={form.country}>
        <option value="">{t("details.contact.countryNone")}</option>
        {#each countries as c (c.code)}
          <option value={c.code}>{c.name}</option>
        {/each}
      </select>
    </label>
    <fieldset class="cd-address" data-testid="contact-address" data-mode={layout.mode}>
      <legend>{t("details.address.title")}</legend>
      {#if layout.mode === "table" && layout.format}
        <p class="muted" data-testid="contact-address-order">{t("details.address.ordered")}</p>
      {:else if layout.mode === "block"}
        <p class="muted" data-testid="contact-address-order">{t("details.address.blockHint")}</p>
      {/if}
      {#if layout.block}
        <label>
          {t("details.address.text")}
          <textarea data-testid="contact-address_text" rows="5" autocomplete="off" bind:value={form.address_text}></textarea>
        </label>
      {/if}
      {#each layout.lines as line, i (i)}
        <div class="cd-line" data-testid="contact-line">
          {#each line as placed (placed.field)}
            <label>
              {t(placed.label)}
              <input data-testid="contact-{placed.field}" type="text" autocomplete="off" bind:value={form[placed.field]} />
            </label>
          {/each}
        </div>
      {/each}
    </fieldset>
    {#each OTHER as f (f)}
      <label>
        {t(`details.contact.${f}`)}
        <input
          data-testid="contact-{f}"
          type={f === "personal_email" ? "email" : "tel"}
          autocomplete="off"
          bind:value={form[f]}
        />
      </label>
      {#if (f === "phone_mobile" || f === "phone_home") && stored?.[f]}
        {@const digits = stored[`${f}_digits`]}
        {@const e164 = stored[`${f}_e164`]}
        <p class="muted cd-forms" data-testid="contact-{f}-forms">
          {#if digits}<span>{tp("details.contact.digits", { digits })}</span>{/if}
          <span>{e164 ? tp("details.contact.e164", { e164 }) : t("details.contact.noE164")}</span>
        </p>
      {/if}
    {/each}
    <button type="submit" data-testid="contact-save">{t("common.save")}</button>
    {#if held}
      <button type="button" data-testid="contact-clear" onclick={() => void clear()}>{t("details.contact.clear")}</button>
    {/if}
    {#if saved}<span class="muted" role="status" data-testid="contact-saved">{t("details.saved")}</span>{/if}
  </form>
</section>

<style>
  .cd-address {
    border: 0;
    margin: 0;
    padding: 0;
    min-inline-size: 0;
  }
  .cd-line {
    display: flex;
    flex-wrap: wrap;
    gap: 0 1em;
  }
  .cd-line > label {
    flex: 1 1 10em;
  }
  .cd-forms {
    display: flex;
    flex-wrap: wrap;
    gap: 0 1em;
    margin-block: 0 0.5em;
  }
  .cd-forms span {
    unicode-bidi: plaintext;
  }
  textarea {
    inline-size: 100%;
    font: inherit;
  }
</style>
