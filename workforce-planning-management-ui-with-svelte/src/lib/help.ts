// Where help goes (WPM-R243, WPM-D191). These are the project's own channels, as CONTRIBUTING.md and SECURITY.md name them: the
// repository's issue tracker for bugs and questions, and the security policy (which says to email, not to open a public issue).
// A deployment that runs WPM for its own staff would point these at its own service desk; nothing here is invented.

import { t, type StringKey } from "./i18n.svelte";

/** The repository, as `package.json` names it. */
export const REPOSITORY_URL =
  "https://github.com/workforce-planning-management/workforce-planning-management";

/** Bugs and questions. */
export const ISSUES_URL = `${REPOSITORY_URL}/issues`;

/** How to report a vulnerability (by email, never a public issue). */
export const SECURITY_URL = `${REPOSITORY_URL}/blob/main/SECURITY.md`;

/** The documentation website. */
export const DOCS_URL = "https://workforce-planning-management.github.io/";

/**
 * What to try next for a failure, chosen by which of the catalogue's failure sentences `message` is (see `errorKey` in
 * `write.ts`): a refusal or a missing record points to a manager or HR, a rejected entry to checking it, anything else to
 * trying again and reporting it if it persists.
 */
export function nextStepKey(message: string): StringKey {
  const is = (key: StringKey) => message === t(key);
  if (
    is("errors.forbidden") ||
    is("errors.notFound") ||
    is("common.notAllowed")
  )
    return "errors.next.access";
  if (is("errors.invalid") || is("errors.conflict")) return "errors.next.input";
  return "errors.next.retry";
}
