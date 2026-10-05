<!--
  Announcements (`/announcements`): the full feed, with the editors' tools
  (post, retire, scheduled and expired posts). Posts are plain text.
-->
<script lang="ts">
  import AnnouncementsFeed from "#lib/components/AnnouncementsFeed.svelte";
  import { t } from "#lib/i18n.svelte.js";
</script>

<svelte:head><title>{t("nav.announcements")} — WPM</title></svelte:head>

<h1>{t("nav.announcements")}</h1>
<AnnouncementsFeed manage />
