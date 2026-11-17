//! The two date and time types a database row holds (WPM-T528).
//!
//! The service's dates and times are `jiff` types: [`jiff::Timestamp`] for an instant, [`jiff::civil::Date`] for a calendar
//! day. `sqlx` cannot decode into them directly (neither crate depends on the other), so a row holds these two newtypes,
//! which implement `sqlx`'s traits through `jiff-sqlx` and give the `jiff` value back by [`Deref`], [`From`] or `.get()`.
//! `sqlx.toml` maps PostgreSQL's `timestamptz` and `date` to them, so the query macros produce them and expect them as
//! parameters; `Into` turns a `jiff` value into one. They serialise exactly as the `jiff` value does: an instant as
//! RFC 3339 in UTC with a `Z` (`2026-11-17T09:30:00Z`), a day as `2026-11-17`.

use std::{fmt, ops::Deref};

use jiff_sqlx::ToSqlx as _;
use serde::{Deserialize, Serialize};
use sqlx::{
    Decode, Encode, Postgres, Type,
    encode::IsNull,
    error::BoxDynError,
    postgres::{PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueRef},
};

macro_rules! sql_newtype {
    ($(#[$doc:meta])* $name:ident, $jiff:ty, $wrapper:ty) => {
        $(#[$doc])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub $jiff);

        impl $name {
            /// The `jiff` value.
            #[must_use]
            pub const fn get(self) -> $jiff {
                self.0
            }
        }

        impl Deref for $name {
            type Target = $jiff;
            fn deref(&self) -> &$jiff {
                &self.0
            }
        }

        impl From<$jiff> for $name {
            fn from(value: $jiff) -> Self {
                Self(value)
            }
        }

        impl From<$name> for $jiff {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl PartialEq<$jiff> for $name {
            fn eq(&self, other: &$jiff) -> bool {
                self.0 == *other
            }
        }

        impl PartialEq<$name> for $jiff {
            fn eq(&self, other: &$name) -> bool {
                *self == other.0
            }
        }

        impl PartialOrd<$jiff> for $name {
            fn partial_cmp(&self, other: &$jiff) -> Option<std::cmp::Ordering> {
                self.0.partial_cmp(other)
            }
        }

        impl PartialOrd<$name> for $jiff {
            fn partial_cmp(&self, other: &$name) -> Option<std::cmp::Ordering> {
                self.partial_cmp(&other.0)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Display::fmt(&self.0, f)
            }
        }

        impl Type<Postgres> for $name {
            fn type_info() -> PgTypeInfo {
                <$wrapper as Type<Postgres>>::type_info()
            }
            fn compatible(ty: &PgTypeInfo) -> bool {
                <$wrapper as Type<Postgres>>::compatible(ty)
            }
        }

        impl PgHasArrayType for $name {
            fn array_type_info() -> PgTypeInfo {
                <$wrapper as PgHasArrayType>::array_type_info()
            }
        }

        impl Encode<'_, Postgres> for $name {
            fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
                Encode::<Postgres>::encode(self.0.to_sqlx(), buf)
            }
        }

        impl<'r> Decode<'r, Postgres> for $name {
            fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
                Ok(Self(<$wrapper as Decode<'r, Postgres>>::decode(value)?.to_jiff()))
            }
        }
    };
}

sql_newtype!(
    /// A `timestamptz` column: an instant.
    SqlTimestamp,
    jiff::Timestamp,
    jiff_sqlx::Timestamp
);

sql_newtype!(
    /// A `date` column: a calendar day.
    SqlDate,
    jiff::civil::Date,
    jiff_sqlx::Date
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn they_serialise_as_the_jiff_value_does() {
        let at: jiff::Timestamp = "2026-11-17T09:30:00.25+00:00".parse().expect("an instant");
        assert_eq!(
            serde_json::to_string(&SqlTimestamp(at)).expect("json"),
            "\"2026-11-17T09:30:00.25Z\""
        );
        let day = jiff::civil::date(2026, 11, 17);
        assert_eq!(
            serde_json::to_string(&SqlDate(day)).expect("json"),
            "\"2026-11-17\""
        );
        let back: SqlDate = serde_json::from_str("\"2026-11-17\"").expect("a day");
        assert_eq!(back, day);
    }

    #[test]
    fn they_compare_with_the_jiff_value_both_ways() {
        let day = jiff::civil::date(2026, 11, 17);
        let later = jiff::civil::date(2026, 11, 18);
        assert!(SqlDate(day) < later);
        assert!(later > SqlDate(day));
        assert_eq!(SqlDate(day).year(), 2026);
    }
}
