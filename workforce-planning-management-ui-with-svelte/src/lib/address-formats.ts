// Address order and labels per country (WPM-D152 question 1, answer (c); WPM-T382). The structured fields stay; this
// table says, for a country it holds, which fields an address uses, in which order and on which lines, and what the
// locality, the region and the postal code are called there. For a country the table lacks, the panel offers one
// free-text block (`address_text`) in the person's own order instead. With no country chosen, the neutral order and
// labels are used. The API stores the fields as typed and does not read this table.
//
// The shape is Google's libaddressinput address data (its `fmt` string, `%A` being the two address lines, and its
// `locality_name_type`, `state_name_type` and `zip_name_type`); Unicode CLDR has no address formats. Every entry says
// where it came from and its status. **None has been read against its source**: they were written from the AI
// assistant's recollection of that data on 2026-10-31, so each is `unverified` until someone reads it.

import type { StringKey } from "./i18n.svelte.js";

export type AddressField =
  "address_line1" | "address_line2" | "city" | "region" | "postcode";

/** What the locality is called (libaddressinput `locality_name_type`). */
export type LocalityKind = "city" | "post_town";
/** What the first-level area is called (libaddressinput `state_name_type`). */
export type RegionKind = "state" | "province" | "prefecture" | "do_si";
/** What the postal code is called (libaddressinput `zip_name_type`). */
export type PostalKind = "postal" | "zip" | "pin";

export interface AddressFormat {
  /** ISO 3166-1 alpha-2. */
  country: string;
  /** The lines of the address, each the fields on it in order, from the libaddressinput `fmt`. */
  lines: AddressField[][];
  locality: LocalityKind;
  /** Absent where the format has no first-level area (`%S`). */
  region?: RegionKind;
  postal: PostalKind;
  /** Where the entry came from. */
  source: string;
  /** `read` once someone has checked it against the source; `unverified` until then. */
  status: "read" | "unverified";
}

const A: AddressField[][] = [["address_line1"], ["address_line2"]];
const source = (country: string) =>
  `Google libaddressinput address data, chromium-i18n.appspot.com/ssl-address/data/${country} (fmt and name types), ` +
  "as recalled by the AI assistant on 2026-10-31; not read";

const ENTRIES: Omit<AddressFormat, "source" | "status">[] = [
  // %N%n%O%n%A%n%Z %C
  {
    country: "DE",
    lines: [...A, ["postcode", "city"]],
    locality: "city",
    postal: "postal",
  },
  // %O%n%N%n%A%n%Z %C %X
  {
    country: "FR",
    lines: [...A, ["postcode", "city"]],
    locality: "city",
    postal: "postal",
  },
  // %N%n%O%n%A%n%C%n%Z
  {
    country: "GB",
    lines: [...A, ["city"], ["postcode"]],
    locality: "post_town",
    postal: "postal",
  },
  // %N%n%O%n%A%n%C, %S %Z
  {
    country: "US",
    lines: [...A, ["city", "region", "postcode"]],
    locality: "city",
    region: "state",
    postal: "zip",
  },
  // %N%n%O%n%A%n%T%n%F%n%L%n%C %Z%n%S
  {
    country: "IN",
    lines: [...A, ["city", "postcode"], ["region"]],
    locality: "city",
    region: "state",
    postal: "pin",
  },
  // %O%n%N%n%A%n%D%n%C-%S%n%Z
  {
    country: "BR",
    lines: [...A, ["city", "region"], ["postcode"]],
    locality: "city",
    region: "state",
    postal: "postal",
  },
  // %N%n%O%n%A%n%C %Z
  {
    country: "SA",
    lines: [...A, ["city", "postcode"]],
    locality: "city",
    postal: "postal",
  },
  // 〒%Z%n%S%n%C%n%A%n%O%n%N: large to small
  {
    country: "JP",
    lines: [["postcode"], ["region"], ["city"], ...A],
    locality: "city",
    region: "prefecture",
    postal: "postal",
  },
  // %Z%n%S%C%D%n%A%n%O%n%N: large to small
  {
    country: "CN",
    lines: [["postcode"], ["region", "city"], ...A],
    locality: "city",
    region: "province",
    postal: "postal",
  },
  // %S %C%D%n%A%n%O%n%N%n%Z: large to small, the postal code last
  {
    country: "KR",
    lines: [["region", "city"], ...A, ["postcode"]],
    locality: "city",
    region: "do_si",
    postal: "postal",
  },
];

/** The table, a modest set of countries of several orders: small to large, large to small, the postal code first or last. */
export const ADDRESS_FORMATS: readonly AddressFormat[] = ENTRIES.map(
  (entry) => ({
    ...entry,
    source: source(entry.country),
    status: "unverified",
  }),
);

/** The neutral order with no country chosen: the fields as they always were, with the neutral labels. */
export const NEUTRAL_LINES: AddressField[][] = [
  ...A,
  ["city"],
  ["region"],
  ["postcode"],
];

export const ADDRESS_FIELDS: readonly AddressField[] = [
  "address_line1",
  "address_line2",
  "city",
  "region",
  "postcode",
];

/** The table's entry for `country` (any case), or `undefined`. */
export function addressFormat(
  country: string | null | undefined,
): AddressFormat | undefined {
  const code = (country ?? "").trim().toUpperCase();
  return ADDRESS_FORMATS.find((f) => f.country === code);
}

/** One field to show: what it is and the catalogue key of its label. */
export interface PlacedField {
  field: AddressField;
  label: StringKey;
}

export interface AddressLayout {
  /** `neutral`: no country; `table`: the country's own order; `block`: a country the table lacks, so the free-text block. */
  mode: "neutral" | "table" | "block";
  format: AddressFormat | undefined;
  /** The structured fields, line by line. */
  lines: PlacedField[][];
  /** Whether to show the free-text block: always for `block`, and wherever it already holds text, so nothing is hidden. */
  block: boolean;
}

const neutralLabel = (field: AddressField) =>
  `details.contact.${field}` as StringKey;

function labelFor(
  field: AddressField,
  format: AddressFormat | undefined,
): StringKey {
  if (format && field === "city")
    return `details.address.locality.${format.locality}` as StringKey;
  if (format?.region && field === "region")
    return `details.address.region.${format.region}` as StringKey;
  if (format && field === "postcode")
    return `details.address.postal.${format.postal}` as StringKey;
  return neutralLabel(field);
}

/**
 * How to lay out the address for `country`, given what is held. A field the country's format does not use, or a
 * structured field under the free-text block, is still shown (at the end, with the neutral label) when it holds a
 * value, so nothing a person or an earlier form wrote is hidden.
 */
export function addressLayout(
  country: string | null | undefined,
  held: Partial<Record<AddressField | "address_text", string | null>> = {},
): AddressLayout {
  const has = (field: AddressField | "address_text") =>
    (held[field] ?? "").trim() !== "";
  const blank = (country ?? "").trim() === "";
  const format = blank ? undefined : addressFormat(country);
  const mode: AddressLayout["mode"] = blank
    ? "neutral"
    : format
      ? "table"
      : "block";
  const base: AddressField[][] =
    mode === "neutral" ? NEUTRAL_LINES : (format?.lines ?? []);
  const lines: PlacedField[][] = base.map((line) =>
    line.map((field) => ({ field, label: labelFor(field, format) })),
  );
  const placed = new Set(base.flat());
  for (const field of ADDRESS_FIELDS) {
    if (!placed.has(field) && has(field))
      lines.push([{ field, label: neutralLabel(field) }]);
  }
  return {
    mode,
    format,
    lines,
    block: mode === "block" || has("address_text"),
  };
}
