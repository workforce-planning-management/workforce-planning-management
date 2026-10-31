//! Migration: a telephone number kept **three ways** (WPM-D152 question 2, WPM-T382): as typed (unchanged), as ASCII
//! digits for search and export, and in E.164 where it can be derived. Adds the derived columns to
//! `worker_contact_details` (the mobile and home numbers) and `emergency_contacts` (the number and the alternate), and
//! backfills existing rows.
//!
//! The backfill is SQL, so this crate stays independent of the service. It follows `rules::phone` in the service: the
//! digits of any decimal digit set the service accepts become ASCII; a number written with `+` is its own E.164; a
//! worker's number written without `+` is read in the address's country, else the country of `work_jurisdiction`, when
//! that country is in [`CALLING_CODES`] (its trunk prefix dropped; the North American plan takes ten digits, or eleven
//! starting `1`); a number starting `00`, a country not in the table, or a result over 15 digits gets no E.164 form. An
//! emergency contact's number is not read in the worker's country (the contact may live elsewhere): only `+` gives it
//! one. The tables here are copies of the service's, frozen with this migration; a service test checks they are equal
//! and that this SQL and the rule agree on the same rows.

use sea_orm_migration::prelude::*;

/// The zero of each Unicode decimal digit set the service reads (a copy of `rules::contact_details::DIGIT_ZEROS`).
pub const DIGIT_ZEROS: [u32; 31] = [
    0x0660, 0x06f0, 0x07c0, 0x0966, 0x09e6, 0x0a66, 0x0ae6, 0x0b66, 0x0be6, 0x0c66, 0x0ce6, 0x0d66,
    0x0de6, 0x0e50, 0x0ed0, 0x0f20, 0x1040, 0x1090, 0x17e0, 0x1810, 0x1946, 0x19d0, 0x1b50, 0x1bb0,
    0x1c40, 0x1c50, 0xa620, 0xa9d0, 0xa9f0, 0xaa50, 0xabf0,
];

/// `(country, calling code, trunk prefix)`: a copy of the service's `src/rules/data/calling-codes.tsv` (unverified; see
/// that file for its sources).
pub const CALLING_CODES: &[(&str, &str, Option<&str>)] = &[
    ("AD", "376", None),
    ("AE", "971", Some("0")),
    ("AR", "54", Some("0")),
    ("AT", "43", Some("0")),
    ("AU", "61", Some("0")),
    ("BD", "880", Some("0")),
    ("BE", "32", Some("0")),
    ("BG", "359", Some("0")),
    ("BH", "973", None),
    ("BR", "55", Some("0")),
    ("BY", "375", Some("8")),
    ("BZ", "501", None),
    ("CA", "1", None),
    ("CH", "41", Some("0")),
    ("CN", "86", Some("0")),
    ("CR", "506", None),
    ("CY", "357", None),
    ("CZ", "420", None),
    ("DE", "49", Some("0")),
    ("DK", "45", None),
    ("DZ", "213", Some("0")),
    ("EE", "372", None),
    ("EG", "20", Some("0")),
    ("ES", "34", None),
    ("ET", "251", Some("0")),
    ("FI", "358", Some("0")),
    ("FR", "33", Some("0")),
    ("GB", "44", Some("0")),
    ("GH", "233", Some("0")),
    ("GR", "30", None),
    ("GT", "502", None),
    ("HK", "852", None),
    ("HN", "504", None),
    ("HR", "385", Some("0")),
    ("HU", "36", Some("06")),
    ("ID", "62", Some("0")),
    ("IE", "353", Some("0")),
    ("IL", "972", Some("0")),
    ("IN", "91", Some("0")),
    ("IR", "98", Some("0")),
    ("IS", "354", None),
    ("IT", "39", None),
    ("JO", "962", Some("0")),
    ("JP", "81", Some("0")),
    ("KE", "254", Some("0")),
    ("KR", "82", Some("0")),
    ("KW", "965", None),
    ("KZ", "7", Some("8")),
    ("LB", "961", Some("0")),
    ("LK", "94", Some("0")),
    ("LU", "352", None),
    ("LV", "371", None),
    ("MA", "212", Some("0")),
    ("MC", "377", None),
    ("MO", "853", None),
    ("MT", "356", None),
    ("MX", "52", None),
    ("MY", "60", Some("0")),
    ("NG", "234", Some("0")),
    ("NI", "505", None),
    ("NL", "31", Some("0")),
    ("NO", "47", None),
    ("NZ", "64", Some("0")),
    ("OM", "968", None),
    ("PA", "507", None),
    ("PE", "51", Some("0")),
    ("PH", "63", Some("0")),
    ("PK", "92", Some("0")),
    ("PL", "48", None),
    ("PT", "351", None),
    ("QA", "974", None),
    ("RO", "40", Some("0")),
    ("RS", "381", Some("0")),
    ("RU", "7", Some("8")),
    ("SA", "966", Some("0")),
    ("SE", "46", Some("0")),
    ("SG", "65", None),
    ("SI", "386", Some("0")),
    ("SK", "421", Some("0")),
    ("SM", "378", None),
    ("SV", "503", None),
    ("TH", "66", Some("0")),
    ("TN", "216", Some("0")),
    ("TR", "90", Some("0")),
    ("TW", "886", Some("0")),
    ("TZ", "255", Some("0")),
    ("UA", "380", Some("0")),
    ("UG", "256", Some("0")),
    ("US", "1", None),
    ("VN", "84", Some("0")),
    ("ZA", "27", Some("0")),
];

/// `translate()`'s two arguments: every digit the service reads (full width, then each set), and its ASCII digit.
fn digit_map() -> (String, String) {
    let mut from = String::new();
    let mut to = String::new();
    for zero in std::iter::once(0xff10).chain(DIGIT_ZEROS) {
        for d in 0..10 {
            from.extend(char::from_u32(zero + d));
            to.extend(char::from_digit(d, 10));
        }
    }
    (from, to)
}

/// The ASCII digits of `column`, or NULL.
fn digits(column: &str) -> String {
    let (from, to) = digit_map();
    format!("NULLIF(regexp_replace(translate({column}, '{from}', '{to}'), '[^0-9]', '', 'g'), '')")
}

/// The E.164 form of `column` (whose digits are in `digits`), with the plan's code and trunk in `p.code` and `p.trunk`
/// (NULL for no known plan).
fn e164(column: &str, digits: &str) -> String {
    format!(
        "CASE
           WHEN {digits} IS NULL THEN NULL
           WHEN ltrim(translate({column}, '＋　', '+ ')) LIKE '+%'
                THEN CASE WHEN {digits} !~ '^0' AND length({digits}) <= 15 THEN '+' || {digits} END
           WHEN p.code IS NULL OR {digits} LIKE '00%' THEN NULL
           WHEN p.code = '1' THEN CASE WHEN {digits} ~ '^[2-9][0-9]{{9}}$' THEN '+1' || {digits}
                                       WHEN {digits} ~ '^1[2-9][0-9]{{9}}$' THEN '+' || {digits} END
           ELSE (SELECT CASE WHEN length(n) >= 4 AND length(p.code) + length(n) <= 15 THEN '+' || p.code || n END
                   FROM (SELECT CASE WHEN p.trunk IS NOT NULL AND left({digits}, length(p.trunk)) = p.trunk
                                     THEN substr({digits}, length(p.trunk) + 1) ELSE {digits} END AS n) national)
         END"
    )
}

/// The statements that fill the derived columns from what is stored. Public so a service test can run them over rows
/// of its own and compare them with the rule; running them again changes nothing.
#[must_use]
pub fn backfill() -> Vec<String> {
    let plans = CALLING_CODES
        .iter()
        .map(|(country, code, trunk)| {
            format!(
                "('{country}', '{code}', {})",
                trunk.map_or_else(|| "NULL".to_string(), |t| format!("'{t}'"))
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    vec![
        format!(
            "UPDATE worker_contact_details SET phone_mobile_digits = {}, phone_home_digits = {}",
            digits("phone_mobile"),
            digits("phone_home")
        ),
        format!(
            "UPDATE emergency_contacts SET phone_digits = {}, alt_phone_digits = {}",
            digits("phone"),
            digits("alt_phone")
        ),
        format!(
            "WITH plans(country, code, trunk) AS (VALUES {plans}),
                  src AS (
                      SELECT d.id,
                             COALESCE(d.country, CASE WHEN split_part(w.work_jurisdiction, '-', 1) ~ '^[A-Z]{{2}}$'
                                                      THEN split_part(w.work_jurisdiction, '-', 1) END) AS country
                        FROM worker_contact_details d LEFT JOIN workers w ON w.pid = d.worker_pid)
             UPDATE worker_contact_details d
                SET phone_mobile_e164 = {mobile}, phone_home_e164 = {home}
               FROM src LEFT JOIN plans p ON p.country = src.country
              WHERE src.id = d.id",
            mobile = e164("d.phone_mobile", "d.phone_mobile_digits"),
            home = e164("d.phone_home", "d.phone_home_digits"),
        ),
        format!(
            "UPDATE emergency_contacts c SET phone_e164 = {phone}, alt_phone_e164 = {alt}
               FROM (SELECT NULL::text AS code, NULL::text AS trunk) p",
            phone = e164("c.phone", "c.phone_digits"),
            alt = e164("c.alt_phone", "c.alt_phone_digits"),
        ),
    ]
}

/// The migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Add the derived columns and fill them.
    ///
    /// # Errors
    ///
    /// Propagates any DDL or SQL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        for sql in [
            "ALTER TABLE worker_contact_details
                 ADD COLUMN IF NOT EXISTS phone_mobile_digits VARCHAR(15) NULL,
                 ADD COLUMN IF NOT EXISTS phone_mobile_e164 VARCHAR(16) NULL,
                 ADD COLUMN IF NOT EXISTS phone_home_digits VARCHAR(15) NULL,
                 ADD COLUMN IF NOT EXISTS phone_home_e164 VARCHAR(16) NULL",
            "ALTER TABLE emergency_contacts
                 ADD COLUMN IF NOT EXISTS phone_digits VARCHAR(15) NULL,
                 ADD COLUMN IF NOT EXISTS phone_e164 VARCHAR(16) NULL,
                 ADD COLUMN IF NOT EXISTS alt_phone_digits VARCHAR(15) NULL,
                 ADD COLUMN IF NOT EXISTS alt_phone_e164 VARCHAR(16) NULL",
        ] {
            db.execute_unprepared(sql).await?;
        }
        for sql in backfill() {
            db.execute_unprepared(&sql).await?;
        }
        Ok(())
    }

    /// Drop the derived columns (what was typed is untouched).
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        for sql in [
            "ALTER TABLE worker_contact_details
                 DROP COLUMN IF EXISTS phone_mobile_digits, DROP COLUMN IF EXISTS phone_mobile_e164,
                 DROP COLUMN IF EXISTS phone_home_digits, DROP COLUMN IF EXISTS phone_home_e164",
            "ALTER TABLE emergency_contacts
                 DROP COLUMN IF EXISTS phone_digits, DROP COLUMN IF EXISTS phone_e164,
                 DROP COLUMN IF EXISTS alt_phone_digits, DROP COLUMN IF EXISTS alt_phone_e164",
        ] {
            db.execute_unprepared(sql).await?;
        }
        Ok(())
    }
}
