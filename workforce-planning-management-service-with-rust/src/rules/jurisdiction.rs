//! **Jurisdiction profiles** (WPM-R214, WPM-D143, WPM-D144, WPM-T407): the parameter registry, the chain of
//! jurisdiction codes that applies to a place of work, and the one pure resolver. DB-free and clock-free: the rows and
//! the date are supplied.
//!
//! A *profile row* is a deployer's dated, sourced statement of one parameter's value for one jurisdiction code, held
//! by an organization. Rows are append-only: a change is a new row with a later `effective_from`. A parameter has
//! **no default**: when no row applies, the answer is [`Resolution::Unknown`] (for a limit, "no limit configured"),
//! never a number.
//!
//! The chain comes from the place the work is done (the worker's `work_jurisdiction`), never from nationality,
//! citizenship or residence, and it is data the organization keeps ([`Link`]), because membership of a bloc or a
//! federation changes and WPM does not assert it. Each registry entry names one [`Combine`] rule for how the layers'
//! values meet.
//!
//! Where the design (spec/worldwide.md, "Jurisdiction profile design") left a detail open, the choice made here is
//! written down in the spec under "What the registry chose (WPM-T407)".

use std::collections::BTreeSet;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// The most links a chain may have; a longer one is refused as malformed (a cycle is refused before that).
pub const MAX_CHAIN: usize = 8;

/// A code's layer, from the widest to the narrowest.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Layer {
    /// A region of several countries (a bloc with shared law), or a deployer-registered code.
    Supranational,
    /// A country (ISO 3166-1 alpha-2), or a deployer-added code ISO 3166-1 lacks.
    National,
    /// A part of a country (ISO 3166-2): a state, province, canton or nation within a state.
    Subnational,
}

impl Layer {
    /// The stored name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Supranational => "supranational",
            Self::National => "national",
            Self::Subnational => "subnational",
        }
    }

    /// Parse a stored name.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "supranational" => Some(Self::Supranational),
            "national" => Some(Self::National),
            "subnational" => Some(Self::Subnational),
            _ => None,
        }
    }
}

/// Whether `code` has the shape of a jurisdiction code: 2 or 3 capital letters or digits, optionally `-` and 1 to 3
/// more (the shape `holidays.region` already uses).
#[must_use]
pub fn valid_code(code: &str) -> bool {
    let part = |text: &str, min: usize, max: usize| {
        (min..=max).contains(&text.len())
            && text
                .bytes()
                .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
    };
    match code.split_once('-') {
        Some((head, tail)) => part(head, 2, 3) && part(tail, 1, 3),
        None => part(code, 2, 3),
    }
}

/// The layer a code's shape implies when the organization has not recorded it: a code with a subdivision part is
/// subnational, any other national. A supranational code is never inferred; it must be recorded.
#[must_use]
pub fn inferred_layer(code: &str) -> Layer {
    if code.contains('-') {
        Layer::Subnational
    } else {
        Layer::National
    }
}

/// Whether `layer` may be recorded for `code`: a code with a subdivision part is only subnational; a code without
/// one is national or supranational.
#[must_use]
pub fn layer_fits(code: &str, layer: Layer) -> bool {
    code.contains('-') == (layer == Layer::Subnational)
}

/// One code an organization keeps, with its layer and its parent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Link {
    /// The jurisdiction code.
    pub code: String,
    /// Its layer.
    pub layer: Layer,
    /// The next wider code, if any.
    pub parent: Option<String>,
}

/// Why a chain could not be built.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChainError {
    /// The start or a parent does not have a code's shape.
    BadCode(String),
    /// A code appears twice on the way up.
    Cycle(String),
    /// A parent is narrower than its child (a national code under a subnational one).
    LayerOrder(String),
    /// More than [`MAX_CHAIN`] links.
    TooLong,
}

/// One step of a chain: a code and its layer, from the place of work outward.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Step {
    /// The code.
    pub code: String,
    /// Its layer.
    pub layer: Layer,
}

/// The chain that applies to work done in `start`: `start` first, then each parent the organization recorded. A code
/// the organization has not recorded is a chain of itself alone, at the layer its shape implies; its parent is not
/// guessed from its prefix.
///
/// # Errors
///
/// A malformed code, a cycle, a parent narrower than its child, or a chain longer than [`MAX_CHAIN`].
pub fn chain(start: &str, links: &[Link]) -> Result<Vec<Step>, ChainError> {
    let mut steps: Vec<Step> = Vec::new();
    let mut next = Some(start.to_string());
    while let Some(code) = next {
        if !valid_code(&code) {
            return Err(ChainError::BadCode(code));
        }
        if steps.iter().any(|step| step.code == code) {
            return Err(ChainError::Cycle(code));
        }
        if steps.len() == MAX_CHAIN {
            return Err(ChainError::TooLong);
        }
        let link = links.iter().find(|link| link.code == code);
        let layer = link.map_or_else(|| inferred_layer(&code), |link| link.layer);
        if steps.last().is_some_and(|child| layer > child.layer) {
            return Err(ChainError::LayerOrder(code));
        }
        next = link.and_then(|link| link.parent.clone());
        steps.push(Step { code, layer });
    }
    Ok(steps)
}

/// How the values of several layers meet.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Combine {
    /// The deepest layer that has a value wins.
    MostSpecific,
    /// For a minimum (a rest floor): the highest value of all layers, the most protective.
    StrictestMin,
    /// For a maximum (a ceiling, a window, a deadline): the lowest value of all layers, the most protective.
    StrictestMax,
    /// All layers' values together.
    Union,
    /// Each layer gives a floor, a ceiling or both; the result is their intersection, and an empty one is a
    /// [`Resolution::Conflict`], never resolved silently.
    Bounded,
}

/// The shape of a parameter's value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Shape {
    /// A whole number in the entry's unit, within `min..=max`.
    Integer {
        /// The least value allowed.
        min: i64,
        /// The greatest value allowed.
        max: i64,
    },
    /// A set of ISO weekdays, 1 (Monday) to 7 (Sunday); may be empty (no weekend), at most six.
    Weekdays,
    /// One ISO weekday, 1 to 7.
    Weekday,
    /// A set of short names (each 1 to 64 characters, at most 64 names).
    Names,
    /// `{"min": n?, "max": n?}` in the entry's unit, each within `0..=max`, at least one given, `min <= max`.
    Range {
        /// The greatest bound allowed.
        max: i64,
    },
}

/// One registry entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Parameter {
    /// The key, `area.name`.
    pub key: &'static str,
    /// The value's shape.
    pub shape: Shape,
    /// The unit of a number (`minutes`, `weeks`, `calendar_days`, `months`, `percent`), or `""`.
    pub unit: &'static str,
    /// How layers combine.
    pub combine: Combine,
    /// What the value means.
    pub meaning: &'static str,
}

/// The parameter registry, sorted by key. A key outside it is refused.
pub const REGISTRY: &[Parameter] = &[
    Parameter {
        key: "calendar.week_start",
        shape: Shape::Weekday,
        unit: "",
        combine: Combine::MostSpecific,
        meaning: "the first day of the working week",
    },
    Parameter {
        key: "holidays.weekend_days",
        shape: Shape::Weekdays,
        unit: "",
        combine: Combine::MostSpecific,
        meaning: "the weekly rest days a holiday's weekend policy and business days follow",
    },
    Parameter {
        key: "working_time.max_average_weekly_minutes",
        shape: Shape::Integer {
            min: 1,
            max: 10_080,
        },
        unit: "minutes",
        combine: Combine::StrictestMax,
        meaning: "the ceiling on average recorded minutes a week over the reference window",
    },
    Parameter {
        key: "working_time.min_daily_rest_minutes",
        shape: Shape::Integer { min: 1, max: 1_440 },
        unit: "minutes",
        combine: Combine::StrictestMin,
        meaning: "the least rest between the end of one shift and the start of the next",
    },
    Parameter {
        key: "working_time.reference_weeks",
        shape: Shape::Integer { min: 1, max: 52 },
        unit: "weeks",
        combine: Combine::StrictestMax,
        meaning: "the trailing window, in weeks of seven calendar days, the weekly average is taken over",
    },
];

/// The registry entry for `key`.
#[must_use]
pub fn parameter(key: &str) -> Option<&'static Parameter> {
    REGISTRY.iter().find(|entry| entry.key == key)
}

/// A parameter's value, in the shape its entry fixes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    /// A whole number, or one weekday.
    Integer(i64),
    /// A set of weekdays or of names, sorted and unique.
    Set(Vec<serde_json::Value>),
    /// A floor and a ceiling.
    Range {
        /// The floor.
        #[serde(skip_serializing_if = "Option::is_none")]
        min: Option<i64>,
        /// The ceiling.
        #[serde(skip_serializing_if = "Option::is_none")]
        max: Option<i64>,
    },
}

impl Value {
    /// The whole number, if this is one.
    #[must_use]
    pub const fn as_integer(&self) -> Option<i64> {
        match self {
            Self::Integer(n) => Some(*n),
            _ => None,
        }
    }

    /// The weekdays (1 to 7), if this is a set of them.
    #[must_use]
    pub fn as_weekdays(&self) -> Option<Vec<u8>> {
        match self {
            Self::Set(items) => items
                .iter()
                .map(|item| item.as_u64().and_then(|n| u8::try_from(n).ok()))
                .collect(),
            _ => None,
        }
    }
}

/// Check a stored or submitted JSON value against `key`'s entry and return it in its canonical form (sets sorted and
/// unique).
///
/// # Errors
///
/// An unknown key, or a value of the wrong shape or out of range, with a message naming the key.
pub fn validate(key: &str, raw: &serde_json::Value) -> Result<Value, String> {
    let entry = parameter(key).ok_or_else(|| format!("{key} is not in the parameter registry"))?;
    validate_entry(entry, raw)
}

/// [`validate`] against a given entry.
///
/// # Errors
///
/// A value of the wrong shape or out of range.
pub fn validate_entry(entry: &Parameter, raw: &serde_json::Value) -> Result<Value, String> {
    let key = entry.key;
    let weekday = |item: &serde_json::Value| item.as_i64().filter(|n| (1..=7).contains(n));
    match entry.shape {
        Shape::Integer { min, max } => raw
            .as_i64()
            .filter(|n| (min..=max).contains(n))
            .map(Value::Integer)
            .ok_or_else(|| format!("{key} must be a whole number from {min} to {max}")),
        Shape::Weekday => weekday(raw)
            .map(Value::Integer)
            .ok_or_else(|| format!("{key} must be an ISO weekday, 1 (Monday) to 7 (Sunday)")),
        Shape::Weekdays => {
            let items = raw
                .as_array()
                .ok_or_else(|| format!("{key} must be a list of ISO weekdays"))?;
            let days: Option<BTreeSet<i64>> = items.iter().map(weekday).collect();
            let days = days.ok_or_else(|| format!("{key} must list ISO weekdays, 1 to 7"))?;
            if days.len() > 6 || days.len() != items.len() {
                return Err(format!("{key} must list at most six different weekdays"));
            }
            Ok(Value::Set(
                days.into_iter().map(serde_json::Value::from).collect(),
            ))
        }
        Shape::Names => {
            let items = raw
                .as_array()
                .ok_or_else(|| format!("{key} must be a list of names"))?;
            let names: Option<BTreeSet<String>> = items
                .iter()
                .map(|item| {
                    item.as_str()
                        .map(str::trim)
                        .filter(|name| (1..=64).contains(&name.chars().count()))
                        .map(str::to_string)
                })
                .collect();
            let names =
                names.ok_or_else(|| format!("{key} must list names of 1 to 64 characters"))?;
            if names.len() > 64 {
                return Err(format!("{key} may list at most 64 names"));
            }
            Ok(Value::Set(
                names.into_iter().map(serde_json::Value::from).collect(),
            ))
        }
        Shape::Range { max } => {
            let bound = |name: &str| -> Result<Option<i64>, String> {
                match raw.get(name) {
                    None | Some(serde_json::Value::Null) => Ok(None),
                    Some(value) => value
                        .as_i64()
                        .filter(|n| (0..=max).contains(n))
                        .map(Some)
                        .ok_or_else(|| {
                            format!("{key}.{name} must be a whole number from 0 to {max}")
                        }),
                }
            };
            if !raw.is_object() {
                return Err(format!("{key} must be {{\"min\": n, \"max\": n}}"));
            }
            let (min, max_bound) = (bound("min")?, bound("max")?);
            match (min, max_bound) {
                (None, None) => Err(format!("{key} needs a min, a max or both")),
                (Some(lo), Some(hi)) if lo > hi => Err(format!("{key}.min must not exceed max")),
                _ => Ok(Value::Range {
                    min,
                    max: max_bound,
                }),
            }
        }
    }
}

/// How far a row's value has been checked.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    /// A worked case in a regime's shape, not a claim about its law.
    Example,
    /// Modelled on a source, or taken from part of it, not checked against the whole text.
    Unverified,
    /// The source was read whole and the value checked against it on `checked_on`.
    Read,
}

impl Status {
    /// The stored name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Example => "example",
            Self::Unverified => "unverified",
            Self::Read => "read",
        }
    }

    /// Parse a stored name.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "example" => Some(Self::Example),
            "unverified" => Some(Self::Unverified),
            "read" => Some(Self::Read),
            _ => None,
        }
    }
}

/// One profile row, as the resolver needs it. `seq` orders rows recorded for the same date (the later wins).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    /// The jurisdiction code.
    pub code: String,
    /// The registry key.
    pub key: String,
    /// The value, already validated.
    pub value: Value,
    /// The first day it is in force.
    pub effective_from: NaiveDate,
    /// The last day it is in force, if it ends.
    pub effective_to: Option<NaiveDate>,
    /// What it rests on.
    pub source: String,
    /// How far it was checked.
    pub status: Status,
    /// Recording order.
    pub seq: i64,
}

/// Where a resolved value came from.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Origin {
    /// The code whose row gave it.
    pub code: String,
    /// That code's layer.
    pub layer: Layer,
    /// That row's status.
    pub status: Status,
    /// That row's source.
    pub source: String,
    /// That row's first day in force.
    pub effective_from: NaiveDate,
}

/// The resolver's answer.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Resolution {
    /// A value, with every row that contributed to it and the least-checked status among them.
    Resolved {
        /// The value.
        value: Value,
        /// The least-checked status among `from` (`read` only when every origin was read).
        status: Status,
        /// Where it came from, the most specific first.
        from: Vec<Origin>,
    },
    /// No row applies: for a limit, "no limit configured". Never a number.
    Unknown {
        /// Why: `no_work_jurisdiction` or `not_configured`.
        reason: &'static str,
    },
    /// A [`Combine::Bounded`] parameter whose layers' bounds do not meet. Shown to HR, never resolved silently.
    Conflict {
        /// The rows whose bounds clash.
        from: Vec<Origin>,
    },
}

impl Resolution {
    /// The whole number, if resolved to one.
    #[must_use]
    pub fn integer(&self) -> Option<i64> {
        match self {
            Self::Resolved { value, .. } => value.as_integer(),
            _ => None,
        }
    }
}

/// The row in force on `as_of` for `code` and `key`: the latest `effective_from` not after `as_of` whose
/// `effective_to` (inclusive) has not passed, the later-recorded on a tie.
fn in_force<'a>(rows: &'a [Row], code: &str, key: &str, as_of: NaiveDate) -> Option<&'a Row> {
    rows.iter()
        .filter(|row| row.code == code && row.key == key)
        .filter(|row| row.effective_from <= as_of && row.effective_to.is_none_or(|to| as_of <= to))
        .max_by_key(|row| (row.effective_from, row.seq))
}

fn origin(step: &Step, row: &Row) -> Origin {
    Origin {
        code: step.code.clone(),
        layer: step.layer,
        status: row.status,
        source: row.source.clone(),
        effective_from: row.effective_from,
    }
}

/// Resolve `key` over `chain` (from [`chain`]; `None` when the work's place is not recorded) as of `as_of`.
///
/// # Errors
///
/// A key outside the registry. A row whose value no longer fits the registry is skipped, as if absent.
pub fn resolve(
    key: &str,
    chain: Option<&[Step]>,
    rows: &[Row],
    as_of: NaiveDate,
) -> Result<Resolution, String> {
    let entry = parameter(key).ok_or_else(|| format!("{key} is not in the parameter registry"))?;
    Ok(resolve_entry(entry, chain, rows, as_of))
}

/// [`resolve`] for a given entry.
#[must_use]
#[allow(clippy::too_many_lines)] // one arm per combination rule
pub fn resolve_entry(
    entry: &Parameter,
    chain: Option<&[Step]>,
    rows: &[Row],
    as_of: NaiveDate,
) -> Resolution {
    let key = entry.key;
    let Some(chain) = chain else {
        return Resolution::Unknown {
            reason: "no_work_jurisdiction",
        };
    };
    let found: Vec<(&Step, &Row)> = chain
        .iter()
        .filter_map(|step| in_force(rows, &step.code, key, as_of).map(|row| (step, row)))
        .filter(|(_, row)| {
            serde_json::to_value(&row.value)
                .ok()
                .and_then(|raw| validate_entry(entry, &raw).ok())
                .is_some()
        })
        .collect();
    if found.is_empty() {
        return Resolution::Unknown {
            reason: "not_configured",
        };
    }
    let resolved = |value: Value, picked: Vec<(&Step, &Row)>| {
        let status = picked
            .iter()
            .map(|(_, row)| row.status)
            .min()
            .unwrap_or(Status::Example);
        Resolution::Resolved {
            value,
            status,
            from: picked
                .into_iter()
                .map(|(step, row)| origin(step, row))
                .collect(),
        }
    };
    // `found` is the most specific first, so `min_by_key`/`max_by_key` ties keep the more specific on a strict pick.
    match entry.combine {
        Combine::MostSpecific => resolved(found[0].1.value.clone(), vec![found[0]]),
        Combine::StrictestMax | Combine::StrictestMin => {
            let mut best = found[0];
            for pair in &found[1..] {
                let (Some(now), Some(new)) = (best.1.value.as_integer(), pair.1.value.as_integer())
                else {
                    continue;
                };
                let stricter = if entry.combine == Combine::StrictestMax {
                    new < now
                } else {
                    new > now
                };
                if stricter {
                    best = *pair;
                }
            }
            resolved(best.1.value.clone(), vec![best])
        }
        Combine::Union => {
            let mut all: Vec<serde_json::Value> = Vec::new();
            for (_, row) in &found {
                if let Value::Set(items) = &row.value {
                    for item in items {
                        if !all.contains(item) {
                            all.push(item.clone());
                        }
                    }
                }
            }
            all.sort_by_key(ToString::to_string);
            resolved(Value::Set(all), found)
        }
        Combine::Bounded => {
            let floor = found
                .iter()
                .filter_map(|pair| range_min(&pair.1.value).map(|n| (n, *pair)))
                .fold(None::<(i64, (&Step, &Row))>, |acc, item| match acc {
                    Some(prev) if prev.0 >= item.0 => Some(prev),
                    _ => Some(item),
                });
            let ceiling = found
                .iter()
                .filter_map(|pair| range_max(&pair.1.value).map(|n| (n, *pair)))
                .fold(None::<(i64, (&Step, &Row))>, |acc, item| match acc {
                    Some(prev) if prev.0 <= item.0 => Some(prev),
                    _ => Some(item),
                });
            let picked: Vec<(&Step, &Row)> = floor
                .iter()
                .chain(ceiling.iter())
                .map(|(_, pair)| *pair)
                .fold(Vec::new(), |mut acc, pair| {
                    if !acc.iter().any(|(step, _)| step.code == pair.0.code) {
                        acc.push(pair);
                    }
                    acc
                });
            match (floor, ceiling) {
                (Some((lo, _)), Some((hi, _))) if lo > hi => Resolution::Conflict {
                    from: picked
                        .into_iter()
                        .map(|(step, row)| origin(step, row))
                        .collect(),
                },
                _ => resolved(
                    Value::Range {
                        min: floor.map(|(n, _)| n),
                        max: ceiling.map(|(n, _)| n),
                    },
                    picked,
                ),
            }
        }
    }
}

const fn range_min(value: &Value) -> Option<i64> {
    match value {
        Value::Range { min, .. } => *min,
        _ => None,
    }
}

const fn range_max(value: &Value) -> Option<i64> {
    match value {
        Value::Range { max, .. } => *max,
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // The codes are ISO 3166 user-assigned ones (`QA`–`QZ`, `XA`–`XZ`), so no test names a real country.

    fn day(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn link(code: &str, layer: Layer, parent: Option<&str>) -> Link {
        Link {
            code: code.into(),
            layer,
            parent: parent.map(Into::into),
        }
    }

    #[allow(clippy::needless_pass_by_value)] // the tests pass `json!` literals
    fn row(code: &str, key: &str, value: serde_json::Value, from: NaiveDate, seq: i64) -> Row {
        Row {
            code: code.into(),
            key: key.into(),
            value: validate_entry(entry_for(key), &value).unwrap(),
            effective_from: from,
            effective_to: None,
            source: format!("synthetic {code}"),
            status: Status::Example,
            seq,
        }
    }

    const NAMES: Parameter = Parameter {
        key: "test.names",
        shape: Shape::Names,
        unit: "",
        combine: Combine::Union,
        meaning: "a union parameter for tests",
    };
    const RANGE: Parameter = Parameter {
        key: "test.range",
        shape: Shape::Range { max: 36_500 },
        unit: "calendar_days",
        combine: Combine::Bounded,
        meaning: "a bounded parameter for tests",
    };

    fn entry_for(key: &str) -> &'static Parameter {
        match key {
            "test.names" => &NAMES,
            "test.range" => &RANGE,
            _ => parameter(key).unwrap(),
        }
    }

    /// A federal shape: a bloc, a country in it, a state in that.
    fn federal() -> Vec<Link> {
        vec![
            link("QZ", Layer::Supranational, None),
            link("XA", Layer::National, Some("QZ")),
            link("XA-01", Layer::Subnational, Some("XA")),
        ]
    }

    #[test]
    fn the_registry_is_sorted_unique_and_every_key_is_area_dot_name() {
        let keys: Vec<&str> = REGISTRY.iter().map(|entry| entry.key).collect();
        let mut sorted = keys.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(keys, sorted);
        assert!(keys.iter().all(|key| {
            key.split_once('.')
                .is_some_and(|(a, b)| !a.is_empty() && !b.is_empty())
        }));
        assert!(parameter("working_time.nothing").is_none());
    }

    #[test]
    fn codes_have_the_holiday_shape_and_a_layer_that_fits() {
        for good in ["XA", "QZ", "XAB", "XA-01", "XA-B", "X1-ABC"] {
            assert!(valid_code(good), "{good}");
        }
        for bad in [
            "", "X", "xa", "XABC", "XA-", "XA-ABCD", "XA-01-2", "X A", "ＸＡ",
        ] {
            assert!(!valid_code(bad), "{bad}");
        }
        assert!(layer_fits("XA-01", Layer::Subnational));
        assert!(!layer_fits("XA-01", Layer::National));
        assert!(layer_fits("QZ", Layer::Supranational) && layer_fits("XA", Layer::National));
        assert!(!layer_fits("XA", Layer::Subnational));
        assert_eq!(inferred_layer("XA"), Layer::National);
        assert_eq!(inferred_layer("XA-01"), Layer::Subnational);
        assert_eq!(Layer::parse("national"), Some(Layer::National));
        assert_eq!(Status::parse(Status::Read.as_str()), Some(Status::Read));
    }

    #[test]
    fn the_chain_runs_from_the_place_of_work_outward_and_guesses_no_parent() {
        let codes = |start: &str| -> Vec<String> {
            chain(start, &federal())
                .unwrap()
                .into_iter()
                .map(|step| step.code)
                .collect()
        };
        assert_eq!(codes("XA-01"), ["XA-01", "XA", "QZ"]);
        assert_eq!(codes("XA"), ["XA", "QZ"]);
        // Not recorded: alone, and its prefix is not taken as its parent.
        assert_eq!(codes("XA-02"), ["XA-02"]);
        assert_eq!(
            chain("XA-02", &federal()).unwrap()[0].layer,
            Layer::Subnational
        );
        assert_eq!(codes("XB"), ["XB"]);
    }

    #[test]
    fn a_malformed_chain_is_refused() {
        let cycle = vec![
            link("XA", Layer::National, Some("XB")),
            link("XB", Layer::National, Some("XA")),
        ];
        assert_eq!(chain("XA", &cycle), Err(ChainError::Cycle("XA".into())));
        let upside = vec![
            link("XA", Layer::National, Some("XA-01")),
            link("XA-01", Layer::Subnational, None),
        ];
        assert_eq!(
            chain("XA", &upside),
            Err(ChainError::LayerOrder("XA-01".into()))
        );
        assert_eq!(chain("xa", &[]), Err(ChainError::BadCode("xa".into())));
        let bad_parent = vec![link("XA", Layer::National, Some("nope"))];
        assert_eq!(
            chain("XA", &bad_parent),
            Err(ChainError::BadCode("nope".into()))
        );
        let long: Vec<Link> = (0..10)
            .map(|i| {
                link(
                    &format!("Q{i}"),
                    Layer::National,
                    Some(&format!("Q{}", i + 1)),
                )
            })
            .collect();
        assert_eq!(chain("Q0", &long), Err(ChainError::TooLong));
    }

    #[test]
    fn values_are_checked_against_their_entry() {
        let weeks = "working_time.reference_weeks";
        assert_eq!(validate(weeks, &json!(17)), Ok(Value::Integer(17)));
        for bad in [json!(0), json!(53), json!("17"), json!(1.5), json!(null)] {
            assert!(validate(weeks, &bad).is_err(), "{bad}");
        }
        let weekend = "holidays.weekend_days";
        assert_eq!(
            validate(weekend, &json!([6, 5])),
            Ok(Value::Set(vec![json!(5), json!(6)]))
        );
        assert_eq!(
            validate(weekend, &json!([])),
            Ok(Value::Set(vec![])),
            "no weekend at all is a shape"
        );
        for bad in [
            json!([0]),
            json!([8]),
            json!([5, 5]),
            json!([1, 2, 3, 4, 5, 6, 7]),
            json!(5),
        ] {
            assert!(validate(weekend, &bad).is_err(), "{bad}");
        }
        assert!(validate("calendar.week_start", &json!(7)).is_ok());
        assert!(validate("calendar.week_start", &json!(0)).is_err());
        assert!(
            validate("no.such", &json!(1))
                .unwrap_err()
                .contains("registry")
        );
        assert_eq!(
            validate_entry(&NAMES, &json!([" b", "a", "b"])),
            Ok(Value::Set(vec![json!("a"), json!("b")]))
        );
        assert!(validate_entry(&NAMES, &json!([""])).is_err());
        assert_eq!(
            validate_entry(&RANGE, &json!({"min": 30})),
            Ok(Value::Range {
                min: Some(30),
                max: None
            })
        );
        for bad in [
            json!({}),
            json!({"min": 9, "max": 3}),
            json!({"max": -1}),
            json!(5),
        ] {
            assert!(validate_entry(&RANGE, &bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn nothing_configured_is_unknown_never_a_number() {
        let steps = chain("XA-01", &federal()).unwrap();
        let key = "working_time.max_average_weekly_minutes";
        assert_eq!(
            resolve(key, Some(&steps), &[], day(2026, 7, 1)),
            Ok(Resolution::Unknown {
                reason: "not_configured"
            })
        );
        assert_eq!(
            resolve(
                key,
                None,
                &[row("XA", key, json!(2880), day(2020, 1, 1), 1)],
                day(2026, 7, 1)
            ),
            Ok(Resolution::Unknown {
                reason: "no_work_jurisdiction"
            })
        );
        // A row for a code outside the chain does not apply.
        let other = [row("XB", key, json!(2880), day(2020, 1, 1), 1)];
        assert_eq!(
            resolve(key, Some(&steps), &other, day(2026, 7, 1))
                .unwrap()
                .integer(),
            None
        );
        assert!(resolve("no.such", Some(&steps), &[], day(2026, 7, 1)).is_err());
    }

    #[test]
    fn rows_are_dated_and_the_latest_in_force_wins() {
        let key = "working_time.reference_weeks";
        let steps = chain("XA", &federal()).unwrap();
        let mut ended = row("XA", key, json!(26), day(2018, 1, 1), 1);
        ended.effective_to = Some(day(2019, 12, 31));
        let rows = [
            ended,
            row("XA", key, json!(17), day(2020, 1, 1), 2),
            row("XA", key, json!(12), day(2027, 1, 1), 3),
            row("XA", key, json!(16), day(2020, 1, 1), 4),
        ];
        let at = |date| resolve(key, Some(&steps), &rows, date).unwrap().integer();
        assert_eq!(at(day(2017, 12, 31)), None, "before any row");
        assert_eq!(at(day(2019, 12, 31)), Some(26), "the last day is inclusive");
        assert_eq!(
            at(day(2020, 1, 1)),
            Some(16),
            "same date: the later-recorded row"
        );
        assert_eq!(
            at(day(2027, 1, 1)),
            Some(12),
            "a later row takes over on its date"
        );
    }

    #[test]
    fn most_specific_takes_the_deepest_layer_with_a_value() {
        let key = "holidays.weekend_days";
        let rows = [
            row("QZ", key, json!([6, 7]), day(2020, 1, 1), 1),
            row("XA-01", key, json!([5]), day(2020, 1, 1), 2),
        ];
        let at = |start: &str| {
            resolve(
                key,
                Some(&chain(start, &federal()).unwrap()),
                &rows,
                day(2026, 1, 1),
            )
            .unwrap()
        };
        match at("XA-01") {
            Resolution::Resolved { value, from, .. } => {
                assert_eq!(value, Value::Set(vec![json!(5)]));
                assert_eq!(from[0].code, "XA-01");
                assert_eq!(from[0].layer, Layer::Subnational);
            }
            other => panic!("{other:?}"),
        }
        match at("XA") {
            Resolution::Resolved { value, from, .. } => {
                assert_eq!(value.as_weekdays(), Some(vec![6, 7]));
                assert_eq!(
                    from[0].code, "QZ",
                    "the state's value does not reach the rest of the country"
                );
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn strictest_takes_the_lowest_ceiling_and_the_highest_floor() {
        let ceiling = "working_time.max_average_weekly_minutes";
        let floor = "working_time.min_daily_rest_minutes";
        let rows = [
            row("QZ", ceiling, json!(2880), day(2020, 1, 1), 1),
            row("XA", ceiling, json!(2400), day(2020, 1, 1), 2),
            row("XA-01", ceiling, json!(2700), day(2020, 1, 1), 3),
            row("QZ", floor, json!(660), day(2020, 1, 1), 4),
            row("XA-01", floor, json!(600), day(2020, 1, 1), 5),
        ];
        let steps = chain("XA-01", &federal()).unwrap();
        let ceiling_now = resolve(ceiling, Some(&steps), &rows, day(2026, 1, 1)).unwrap();
        assert_eq!(ceiling_now.integer(), Some(2400));
        let Resolution::Resolved { from, .. } = &ceiling_now else {
            panic!()
        };
        assert_eq!(from.len(), 1);
        assert_eq!(from[0].code, "XA", "the value names where it came from");
        assert_eq!(
            resolve(floor, Some(&steps), &rows, day(2026, 1, 1))
                .unwrap()
                .integer(),
            Some(660)
        );
        // A tie keeps the more specific layer as the origin.
        let tie = [
            row("QZ", ceiling, json!(2400), day(2020, 1, 1), 1),
            row("XA", ceiling, json!(2400), day(2020, 1, 1), 2),
        ];
        let Resolution::Resolved { from, .. } =
            resolve(ceiling, Some(&steps), &tie, day(2026, 1, 1)).unwrap()
        else {
            panic!()
        };
        assert_eq!(from[0].code, "XA");
    }

    #[test]
    fn union_joins_every_layer() {
        let rows = [
            row("QZ", "test.names", json!(["b", "a"]), day(2020, 1, 1), 1),
            row("XA-01", "test.names", json!(["c", "a"]), day(2020, 1, 1), 2),
        ];
        let steps = chain("XA-01", &federal()).unwrap();
        let Resolution::Resolved { value, from, .. } =
            resolve_entry(&NAMES, Some(&steps), &rows, day(2026, 1, 1))
        else {
            panic!()
        };
        assert_eq!(value, Value::Set(vec![json!("a"), json!("b"), json!("c")]));
        assert_eq!(
            from.iter().map(|o| o.code.as_str()).collect::<Vec<_>>(),
            ["XA-01", "QZ"]
        );
    }

    #[test]
    fn bounded_intersects_and_an_empty_intersection_is_a_conflict() {
        let steps = chain("XA-01", &federal()).unwrap();
        let ok = [
            row("QZ", "test.range", json!({"max": 2190}), day(2020, 1, 1), 1),
            row("XA", "test.range", json!({"min": 365}), day(2020, 1, 1), 2),
            row(
                "XA-01",
                "test.range",
                json!({"min": 180, "max": 3650}),
                day(2020, 1, 1),
                3,
            ),
        ];
        let Resolution::Resolved { value, from, .. } =
            resolve_entry(&RANGE, Some(&steps), &ok, day(2026, 1, 1))
        else {
            panic!()
        };
        assert_eq!(
            value,
            Value::Range {
                min: Some(365),
                max: Some(2190)
            }
        );
        assert_eq!(
            from.iter().map(|o| o.code.as_str()).collect::<Vec<_>>(),
            ["XA", "QZ"]
        );
        let clash = [
            row("QZ", "test.range", json!({"max": 100}), day(2020, 1, 1), 1),
            row("XA", "test.range", json!({"min": 365}), day(2020, 1, 1), 2),
        ];
        let Resolution::Conflict { from } =
            resolve_entry(&RANGE, Some(&steps), &clash, day(2026, 1, 1))
        else {
            panic!("an empty intersection is refused, not resolved")
        };
        assert_eq!(from.len(), 2);
    }

    #[test]
    fn the_status_is_the_least_checked_origin() {
        let steps = chain("XA-01", &federal()).unwrap();
        let mut read = row("QZ", "test.names", json!(["a"]), day(2020, 1, 1), 1);
        read.status = Status::Read;
        let mut unverified = row("XA", "test.names", json!(["b"]), day(2020, 1, 1), 2);
        unverified.status = Status::Unverified;
        let status = |rows: &[Row]| match resolve_entry(&NAMES, Some(&steps), rows, day(2026, 1, 1))
        {
            Resolution::Resolved { status, .. } => status,
            other => panic!("{other:?}"),
        };
        assert_eq!(status(std::slice::from_ref(&read)), Status::Read);
        assert_eq!(
            status(&[read.clone(), unverified.clone()]),
            Status::Unverified
        );
        assert_eq!(
            status(&[
                read,
                unverified,
                row("XA-01", "test.names", json!(["c"]), day(2020, 1, 1), 3)
            ]),
            Status::Example
        );
    }

    /// Two profiles of a different shape (D143's equity test): a single-country one with no bloc, and the federal one,
    /// resolve the same three working-time keys each to its own values.
    #[test]
    fn two_profiles_of_a_different_shape_resolve_independently() {
        let mut links = federal();
        links.push(link("XB", Layer::National, None));
        let rows = [
            row(
                "XB",
                "working_time.reference_weeks",
                json!(17),
                day(2020, 1, 1),
                1,
            ),
            row(
                "XB",
                "working_time.max_average_weekly_minutes",
                json!(2880),
                day(2020, 1, 1),
                2,
            ),
            row(
                "XB",
                "working_time.min_daily_rest_minutes",
                json!(660),
                day(2020, 1, 1),
                3,
            ),
            row(
                "XA",
                "working_time.reference_weeks",
                json!(4),
                day(2020, 1, 1),
                4,
            ),
            row(
                "XA-01",
                "working_time.max_average_weekly_minutes",
                json!(2880),
                day(2020, 1, 1),
                5,
            ),
            row(
                "QZ",
                "working_time.max_average_weekly_minutes",
                json!(2700),
                day(2020, 1, 1),
                6,
            ),
        ];
        let get = |start: &str, key: &str| {
            resolve(
                key,
                Some(&chain(start, &links).unwrap()),
                &rows,
                day(2026, 1, 1),
            )
            .unwrap()
            .integer()
        };
        assert_eq!(get("XB", "working_time.reference_weeks"), Some(17));
        assert_eq!(
            get("XB", "working_time.max_average_weekly_minutes"),
            Some(2880)
        );
        assert_eq!(get("XA-01", "working_time.reference_weeks"), Some(4));
        assert_eq!(
            get("XA-01", "working_time.max_average_weekly_minutes"),
            Some(2700)
        );
        assert_eq!(
            get("XA-01", "working_time.min_daily_rest_minutes"),
            None,
            "no rest floor configured there"
        );
    }

    /// The spec's registry table (spec/worldwide.md, "The parameter registry") lists exactly the code's keys, with the
    /// same combination rule. Skipped when the spec is not beside the crate (a packaged copy).
    #[test]
    fn the_spec_lists_the_registry() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../spec/worldwide.md");
        let Ok(spec) = std::fs::read_to_string(path) else {
            return;
        };
        let table = spec
            .split("### The parameter registry")
            .nth(1)
            .and_then(|rest| rest.split("\n### ").next())
            .expect("the registry section");
        let listed: Vec<(String, String)> = table
            .lines()
            .filter(|line| line.starts_with("| `"))
            .map(|line| {
                let cells: Vec<&str> = line.split('|').map(str::trim).collect();
                let unquote = |cell: &str| cell.split('`').nth(1).unwrap_or_default().to_string();
                (unquote(cells[1]), unquote(cells[3]))
            })
            .collect();
        let code: Vec<(String, String)> = REGISTRY
            .iter()
            .map(|entry| {
                let combine = serde_json::to_value(entry.combine).unwrap();
                (entry.key.to_string(), combine.as_str().unwrap().to_string())
            })
            .collect();
        assert_eq!(listed, code);
    }
}
