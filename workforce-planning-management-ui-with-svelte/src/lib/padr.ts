// The words for where someone stands against the PADR cycle (WPM-R172). `unknown` has its own words, because it is never "current".

import { t } from "./i18n.svelte.js";

export function standingWord(standing: string): string {
  if (standing === "current") return t("padr.current");
  if (standing === "overdue") return t("padr.overdue");
  return t("padr.unknown");
}
