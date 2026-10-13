// The word for a result that may be `unknown`. In the vocabulary `unknown` already reads "not enough
// to compare" (a pay comparison); for capacity, oversight and cutover it means the answer cannot be
// said, which is a different thing and keeps its own word.

import { t, tv } from "./i18n.svelte";

export function stateLabel(state: string): string {
  return state === "unknown" ? t("common.unknown") : tv(state);
}
