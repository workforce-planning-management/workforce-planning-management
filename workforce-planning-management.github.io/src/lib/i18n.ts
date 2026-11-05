// The site's own words (navigation, picker labels, wordmark, footer, search), as content: one catalogue per language in
// src/content/locales/. `en-001` is the source. Every catalogue has every key (tests/e2e/site.spec.ts checks), so a
// page never mixes languages by accident. Written by an AI assistant; not reviewed by a native speaker.
import ar from "../content/locales/ar-001.json" with { type: "json" };
import en from "../content/locales/en-001.json" with { type: "json" };
import cy from "../content/locales/cy-001.json" with { type: "json" };
import hi from "../content/locales/hi-001.json" with { type: "json" };
import zh from "../content/locales/zh-001.json" with { type: "json" };

export type Ui = typeof en;

const CATALOGUES: Record<string, Ui> = { ar, cy, en, hi, zh };

/** The catalogue for a page's locale: by language, so `cy-gb` reads `cy-001`, and English where there is none yet. */
export function ui(locale: string | null | undefined): Ui {
  return CATALOGUES[locale?.split("-")[0] ?? "en"] ?? en;
}

/** Every catalogue, for the parity test. */
export const catalogues = CATALOGUES;

/** A string with its `{name}` placeholders filled. */
export function fill(template: string, values: Record<string, string>): string {
  return template.replace(/\{(\w+)\}/g, (whole, name: string) => values[name] ?? whole);
}
