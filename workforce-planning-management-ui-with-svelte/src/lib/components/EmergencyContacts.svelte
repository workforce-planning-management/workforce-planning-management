<!--
  Emergency contacts: people to reach if something happens to the worker.
  Third-party personal data — only the worker and HR can see or change
  them, so the panel renders nothing for anyone else (the service refuses
  the read with 403).
-->
<script lang="ts">
  import {
    addEmergencyContact,
    listEmergencyContacts,
    removeEmergencyContact,
  } from "#lib/api/wpm.js";
  import { ApiError } from "#lib/api/client.js";
  import type { EmergencyContact } from "#lib/api/types.js";
  import { t } from "#lib/i18n.svelte.js";

  let { workerPid }: { workerPid: string } = $props();

  let contacts = $state<EmergencyContact[]>([]);
  let allowed = $state(true);
  let error = $state<string | null>(null);
  let name = $state("");
  let relationship = $state("");
  let phone = $state("");
  let altPhone = $state("");
  let email = $state("");
  let note = $state("");

  const message = (cause: unknown) => (cause instanceof Error ? cause.message : String(cause));

  async function load() {
    try {
      contacts = await listEmergencyContacts(workerPid);
      allowed = true;
    } catch (cause) {
      // Not the worker or HR: not an error, just not theirs to see.
      if (cause instanceof ApiError && (cause.status === 403 || cause.status === 401)) {
        allowed = false;
      } else {
        error = message(cause);
      }
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

{#if allowed}
  <section class="panel" data-testid="emergency-contacts">
    <h2>{t("contacts.title")}</h2>
    <p class="muted">{t("contacts.hint")}</p>
    {#if error}<p class="error" data-testid="error">{error}</p>{/if}
    <ol data-testid="contact-list">
      {#each contacts as c (c.pid)}
        <li>
          <strong>{c.name}</strong>
          <span class="muted">— {c.relationship}</span>
          · <a href={`tel:${c.phone.replace(/[^+\d]/g, "")}`}>{c.phone}</a>
          {#if c.alt_phone}· <a href={`tel:${c.alt_phone.replace(/[^+\d]/g, "")}`}>{c.alt_phone}</a>{/if}
          {#if c.email}· <a href={`mailto:${c.email}`}>{c.email}</a>{/if}
          {#if c.note}<span class="muted">— {c.note}</span>{/if}
          <button type="button" onclick={() => void run(() => removeEmergencyContact(c.pid))}>
            {t("contacts.remove")}
          </button>
        </li>
      {:else}
        <li class="muted">{t("contacts.none")}</li>
      {/each}
    </ol>
    <form
      onsubmit={(event) => {
        event.preventDefault();
        void run(async () => {
          await addEmergencyContact(workerPid, {
            name: name.trim(),
            relationship: relationship.trim(),
            phone: phone.trim(),
            ...(altPhone.trim() ? { alt_phone: altPhone.trim() } : {}),
            ...(email.trim() ? { email: email.trim() } : {}),
            ...(note.trim() ? { note: note.trim() } : {}),
          });
          name = relationship = phone = altPhone = email = note = "";
        });
      }}
    >
      <label>{t("common.name")} <input bind:value={name} required maxlength="200" data-testid="contact-name" /></label>
      <label>{t("contacts.relationship")} <input bind:value={relationship} required maxlength="200" data-testid="contact-relationship" /></label>
      <label>{t("contacts.phone")} <input type="tel" bind:value={phone} required data-testid="contact-phone" /></label>
      <label>{t("contacts.altPhone")} <input type="tel" bind:value={altPhone} /></label>
      <label>{t("contacts.email")} <input type="email" bind:value={email} /></label>
      <label>{t("contacts.note")} <input bind:value={note} maxlength="500" /></label>
      <button type="submit" data-testid="contact-add">{t("contacts.add")}</button>
    </form>
  </section>
{/if}
