// The site's own words (navigation, picker labels, wordmark, footer, search), as content: one catalogue per language in
// src/content/locales/. `en-001` is the source. Every catalogue has every key (tests/e2e/site.spec.ts checks), so a
// page never mixes languages by accident. Written by an AI assistant; not reviewed by a native speaker.
import ar from "../content/locales/ar-001.json" with { type: "json" };
import en from "../content/locales/en-001.json" with { type: "json" };
import bn from "../content/locales/bn-001.json" with { type: "json" };
import cy from "../content/locales/cy-001.json" with { type: "json" };
import da from "../content/locales/da-001.json" with { type: "json" };
import de from "../content/locales/de-001.json" with { type: "json" };
import es from "../content/locales/es-001.json" with { type: "json" };
import fr from "../content/locales/fr-001.json" with { type: "json" };
import hi from "../content/locales/hi-001.json" with { type: "json" };
import id from "../content/locales/id-001.json" with { type: "json" };
import it from "../content/locales/it-001.json" with { type: "json" };
import ja from "../content/locales/ja-001.json" with { type: "json" };
import ko from "../content/locales/ko-001.json" with { type: "json" };
import nl from "../content/locales/nl-001.json" with { type: "json" };
import ptBr from "../content/locales/pt-br.json" with { type: "json" };
import pt from "../content/locales/pt-001.json" with { type: "json" };
import ru from "../content/locales/ru-001.json" with { type: "json" };
import sv from "../content/locales/sv-001.json" with { type: "json" };
import sw from "../content/locales/sw-001.json" with { type: "json" };
import ur from "../content/locales/ur-001.json" with { type: "json" };
import vi from "../content/locales/vi-001.json" with { type: "json" };
import zh from "../content/locales/zh-001.json" with { type: "json" };
import zhTw from "../content/locales/zh-tw.json" with { type: "json" };

export type Ui = typeof en;

const CATALOGUES: Record<string, Ui> = { ar, bn, cy, da, de, en, es, fr, hi, id, it, ja, ko, nl, pt, "pt-br": ptBr, ru, sv, sw, ur, vi, zh, "zh-tw": zhTw };

/** The catalogue for a page's locale: its own where it has one (`zh-tw`), else by language, so `cy-gb` reads `cy-001`, and English where there is none yet. */
export function ui(locale: string | null | undefined): Ui {
  const key = locale?.toLowerCase() ?? "en";
  return CATALOGUES[key] ?? CATALOGUES[key.split("-")[0] ?? "en"] ?? en;
}

/** Every catalogue, for the parity test. */
export const catalogues = CATALOGUES;

/** A string with its `{name}` placeholders filled. */
export function fill(template: string, values: Record<string, string>): string {
  return template.replace(/\{(\w+)\}/g, (whole, name: string) => values[name] ?? whole);
}
