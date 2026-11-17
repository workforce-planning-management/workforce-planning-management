//! Currency codes and their minor units (WPM-T400, WPM-D151; WPM-R214, WPM-D143).
//!
//! No currency is built in. A stored amount is an integer count of its currency's minor unit, and the number of
//! decimal places of that unit (the exponent: 0 for JPY, 2 for EUR, 3 for KWD, 4 for CLF) is the ISO 4217 one in
//! the table below, never a fixed hundred and never the Unicode CLDR display digits (CLDR shows 0 places for 16
//! codes, HUF, IDR, COP, PKR and IQD among them, where ISO 4217 has 2, or 3 for IQD).
//!
//! The table has the same shape and entries as the sibling project-portfolio-management service's
//! `src/currency.rs` and the contact-relationship-management service's `src/rules/currency.rs`, so the family's
//! apps agree on every exponent; change them together. WPM uses the table as the list of codes the API accepts: an
//! expense claim, a salary, a plan budget, a contractor rate, a benefit plan, a benchmark or a requisition in a code
//! the table does not hold is refused with 422, so every new stored code has a known exponent. Codes stored before
//! WPM-T400 are not checked (see spec/worldwide.md).

/// ISO 4217 minor-unit exponents: the active codes, sorted, with the number of decimal places of
/// the minor unit. Funds and metals with no minor unit (`XAU`, `XDR`, `XXX`, ...) are left out, so
/// the API does not accept them. The UI's `src/lib/money.ts` holds the same table (a vitest test compares the two).
pub const MINOR_UNITS: &[(&str, u32)] = &[
    ("AED", 2),
    ("AFN", 2),
    ("ALL", 2),
    ("AMD", 2),
    ("ANG", 2),
    ("AOA", 2),
    ("ARS", 2),
    ("AUD", 2),
    ("AWG", 2),
    ("AZN", 2),
    ("BAM", 2),
    ("BBD", 2),
    ("BDT", 2),
    ("BGN", 2),
    ("BHD", 3),
    ("BIF", 0),
    ("BMD", 2),
    ("BND", 2),
    ("BOB", 2),
    ("BRL", 2),
    ("BSD", 2),
    ("BTN", 2),
    ("BWP", 2),
    ("BYN", 2),
    ("BZD", 2),
    ("CAD", 2),
    ("CDF", 2),
    ("CHF", 2),
    ("CLF", 4),
    ("CLP", 0),
    ("CNY", 2),
    ("COP", 2),
    ("CRC", 2),
    ("CUP", 2),
    ("CVE", 2),
    ("CZK", 2),
    ("DJF", 0),
    ("DKK", 2),
    ("DOP", 2),
    ("DZD", 2),
    ("EGP", 2),
    ("ERN", 2),
    ("ETB", 2),
    ("EUR", 2),
    ("FJD", 2),
    ("FKP", 2),
    ("GBP", 2),
    ("GEL", 2),
    ("GHS", 2),
    ("GIP", 2),
    ("GMD", 2),
    ("GNF", 0),
    ("GTQ", 2),
    ("GYD", 2),
    ("HKD", 2),
    ("HNL", 2),
    ("HTG", 2),
    ("HUF", 2),
    ("IDR", 2),
    ("ILS", 2),
    ("INR", 2),
    ("IQD", 3),
    ("IRR", 2),
    ("ISK", 0),
    ("JMD", 2),
    ("JOD", 3),
    ("JPY", 0),
    ("KES", 2),
    ("KGS", 2),
    ("KHR", 2),
    ("KMF", 0),
    ("KPW", 2),
    ("KRW", 0),
    ("KWD", 3),
    ("KYD", 2),
    ("KZT", 2),
    ("LAK", 2),
    ("LBP", 2),
    ("LKR", 2),
    ("LRD", 2),
    ("LSL", 2),
    ("LYD", 3),
    ("MAD", 2),
    ("MDL", 2),
    ("MGA", 2),
    ("MKD", 2),
    ("MMK", 2),
    ("MNT", 2),
    ("MOP", 2),
    ("MRU", 2),
    ("MUR", 2),
    ("MVR", 2),
    ("MWK", 2),
    ("MXN", 2),
    ("MYR", 2),
    ("MZN", 2),
    ("NAD", 2),
    ("NGN", 2),
    ("NIO", 2),
    ("NOK", 2),
    ("NPR", 2),
    ("NZD", 2),
    ("OMR", 3),
    ("PAB", 2),
    ("PEN", 2),
    ("PGK", 2),
    ("PHP", 2),
    ("PKR", 2),
    ("PLN", 2),
    ("PYG", 0),
    ("QAR", 2),
    ("RON", 2),
    ("RSD", 2),
    ("RUB", 2),
    ("RWF", 0),
    ("SAR", 2),
    ("SBD", 2),
    ("SCR", 2),
    ("SDG", 2),
    ("SEK", 2),
    ("SGD", 2),
    ("SHP", 2),
    ("SLE", 2),
    ("SOS", 2),
    ("SRD", 2),
    ("SSP", 2),
    ("STN", 2),
    ("SVC", 2),
    ("SYP", 2),
    ("SZL", 2),
    ("THB", 2),
    ("TJS", 2),
    ("TMT", 2),
    ("TND", 3),
    ("TOP", 2),
    ("TRY", 2),
    ("TTD", 2),
    ("TWD", 2),
    ("TZS", 2),
    ("UAH", 2),
    ("UGX", 0),
    ("USD", 2),
    ("UYI", 0),
    ("UYU", 2),
    ("UYW", 4),
    ("UZS", 2),
    ("VED", 2),
    ("VES", 2),
    ("VND", 0),
    ("VUV", 0),
    ("WST", 2),
    ("XAF", 0),
    ("XCD", 2),
    ("XCG", 2),
    ("XOF", 0),
    ("XPF", 0),
    ("YER", 2),
    ("ZAR", 2),
    ("ZMW", 2),
    ("ZWG", 2),
];

/// Decimal places of `code`'s minor unit, or `None` when the table does not hold it.
#[must_use]
pub fn exponent(code: &str) -> Option<u32> {
    MINOR_UNITS
        .binary_search_by(|(c, _)| (*c).cmp(code))
        .ok()
        .map(|i| MINOR_UNITS[i].1)
}

/// Whether `code` is three capital letters and a code the table holds.
#[must_use]
pub fn is_known(code: &str) -> bool {
    check(code).is_ok()
}

/// `code` when it is an ISO 4217 code in the table.
///
/// # Errors
///
/// The reason, when it is not three capital letters or not a code the table holds.
pub fn check(code: &str) -> Result<&str, String> {
    if code.len() != 3 || !code.bytes().all(|b| b.is_ascii_uppercase()) {
        return Err(
            "a currency is an ISO 4217 code: three capital letters, such as JPY, EUR or KWD"
                .to_string(),
        );
    }
    if exponent(code).is_none() {
        return Err(format!("{code} is not an ISO 4217 currency code"));
    }
    Ok(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_table_is_sorted_unique_and_well_formed() {
        for pair in MINOR_UNITS.windows(2) {
            assert!(pair[0].0 < pair[1].0, "{} before {}", pair[0].0, pair[1].0);
        }
        for (code, e) in MINOR_UNITS {
            assert_eq!(check(code), Ok(*code));
            assert!(matches!(e, 0 | 2 | 3 | 4), "{code}");
        }
    }

    #[test]
    fn the_exponents_that_are_not_two_are_exactly_the_iso_list() {
        let of = |e: u32| {
            MINOR_UNITS
                .iter()
                .filter(|(_, x)| *x == e)
                .map(|(c, _)| *c)
                .collect::<Vec<_>>()
        };
        assert_eq!(
            of(0),
            [
                "BIF", "CLP", "DJF", "GNF", "ISK", "JPY", "KMF", "KRW", "PYG", "RWF", "UGX", "UYI",
                "VND", "VUV", "XAF", "XOF", "XPF"
            ]
        );
        assert_eq!(of(3), ["BHD", "IQD", "JOD", "KWD", "LYD", "OMR", "TND"]);
        assert_eq!(of(4), ["CLF", "UYW"]);
    }

    #[test]
    fn the_codes_where_cldr_shows_fewer_digits_keep_their_iso_minor_unit() {
        for code in [
            "AFN", "ALL", "COP", "HUF", "IDR", "IRR", "KPW", "LAK", "LBP", "MGA", "MMK", "PKR",
            "SOS", "SYP", "YER",
        ] {
            assert_eq!(exponent(code), Some(2), "{code}");
        }
        assert_eq!(exponent("IQD"), Some(3));
    }

    #[test]
    fn exponent_looks_the_code_up_and_knows_no_other() {
        assert_eq!(exponent("JPY"), Some(0));
        assert_eq!(exponent("EUR"), Some(2));
        assert_eq!(exponent("KWD"), Some(3));
        assert_eq!(exponent("CLF"), Some(4));
        assert_eq!(exponent("ZZZ"), None);
        assert_eq!(exponent("XAU"), None, "a metal has no minor unit");
        assert_eq!(exponent("jpy"), None, "the table is upper case");
    }

    #[test]
    fn a_code_is_checked_for_shape_then_against_the_table() {
        for good in [
            "JPY", "EUR", "KWD", "CLF", "INR", "BRL", "NGN", "HUF", "IDR", "USD",
        ] {
            assert_eq!(check(good), Ok(good));
            assert!(is_known(good));
        }
        for bad in ["jpy", "JY", "JPYY", "J1Y", "", "£££", " EUR", "ÉUR"] {
            assert!(
                check(bad).unwrap_err().contains("three capital letters"),
                "{bad:?}"
            );
            assert!(!is_known(bad));
        }
        for unknown in ["ZZZ", "ABC", "XAU", "XXX"] {
            assert!(
                check(unknown).unwrap_err().contains("not an ISO 4217"),
                "{unknown:?}"
            );
            assert!(!is_known(unknown));
        }
    }
}
