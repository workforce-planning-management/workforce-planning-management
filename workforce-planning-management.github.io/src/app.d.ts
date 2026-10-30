// See https://svelte.dev/docs/kit/types#app
declare global {
  namespace App {
    interface PageData {
      /** The page title, also used in <title>. */
      title?: string;
      /** The role-brief locale of the page, when it has one. */
      locale?: import("#lib/site.js").BriefLocale;
      /** The same page in each brief locale (locale → site route), for the locale switcher. */
      alternates?: Partial<Record<import("#lib/site.js").BriefLocale, string>>;
    }
  }
}

export {};
