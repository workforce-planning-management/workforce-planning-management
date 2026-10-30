//! Pure rules for ergonomic (DSE) workstation assessments (WPM-R32 /
//! WPM-D24): the default checklist, the completion gate, and the
//! open-issue count. Every item names **equipment or environment** —
//! there is deliberately no symptom in the vocabulary.

/// One item of the default checklist: a stable key a locale translates (`erg.items.<key>` in the
/// UI's catalogue) and the English text stored beside it for an API reader (WPM-T386).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DefaultItem {
    /// The stable key, lower-case letters and underscores; stored in `ergonomic_items.item_key`.
    pub key: &'static str,
    /// The English text, stored in `ergonomic_items.name`.
    pub text: &'static str,
}

/// The default workstation checklist. **One regime's list, an example:** its shape is that of the
/// UK Health and Safety (Display Screen Equipment) Regulations 1992 workstation assessment, the only
/// regime read for it; it is not claimed to be what any other jurisdiction requires. A deployer
/// elsewhere passes its own `items` when it opens an assessment. Each item has a key, so the UI shows
/// it in the reader's language; a key is never reused for a different item.
pub const DSE_ITEMS: &[DefaultItem] = &[
    DefaultItem {
        key: "screen",
        text: "Screen at a comfortable height and distance, free of flicker",
    },
    DefaultItem {
        key: "chair",
        text: "Chair adjustable, with lower-back support",
    },
    DefaultItem {
        key: "keyboard_mouse",
        text: "Keyboard and mouse positioned to keep wrists straight",
    },
    DefaultItem {
        key: "desk_space",
        text: "Enough desk space; documents holdable beside the screen",
    },
    DefaultItem {
        key: "lighting",
        text: "Lighting adequate; screen free of glare and reflections",
    },
    DefaultItem {
        key: "leg_room",
        text: "Leg room clear; feet supported",
    },
    DefaultItem {
        key: "breaks",
        text: "Work pattern allows breaks or changes of activity",
    },
    DefaultItem {
        key: "software",
        text: "Software and display legible and responsive",
    },
];

/// The key of a default item from its stored English text, for a row written before items had keys
/// (the migration backfills the same mapping). Custom items have none.
#[must_use]
pub fn key_for_text(text: &str) -> Option<&'static str> {
    DSE_ITEMS.iter().find(|i| i.text == text).map(|i| i.key)
}

/// The key an item is shown by: the stored one, else the default item whose English text it holds
/// word for word, else none (a custom item, shown as typed).
#[must_use]
pub fn display_key(stored: Option<&str>, name: &str) -> Option<String> {
    stored
        .map(ToString::to_string)
        .or_else(|| key_for_text(name).map(ToString::to_string))
}

/// Completing an assessment requires every item answered — recording
/// an obligation and not enforcing it would make the record worse
/// than useless (the WPM-D15 posture).
///
/// # Errors
///
/// A human-readable refusal counting the unanswered items.
pub fn may_complete(answers: &[Option<bool>]) -> Result<(), String> {
    let unanswered = answers.iter().filter(|a| a.is_none()).count();
    if unanswered == 0 {
        Ok(())
    } else {
        Err(format!(
            "{unanswered} item(s) unanswered — every item needs ok or issue"
        ))
    }
}

/// How many answered items flagged an issue (`ok == false`).
#[must_use]
pub fn open_issues(answers: &[Option<bool>]) -> usize {
    answers.iter().filter(|a| **a == Some(false)).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The default checklist is non-empty and names equipment, not
    /// symptoms — no item mentions the body's state (WPM-D24).
    #[test]
    fn default_checklist_is_about_the_workstation() {
        assert_eq!(DSE_ITEMS.len(), 8);
        for item in DSE_ITEMS {
            let lower = item.text.to_lowercase();
            for symptom in ["pain", "ache", "strain", "injury", "symptom"] {
                assert!(!lower.contains(symptom), "{item:?} must not name a symptom");
            }
        }
    }

    #[test]
    fn every_default_item_has_a_unique_key_of_plain_letters() {
        let mut keys: Vec<&str> = DSE_ITEMS.iter().map(|i| i.key).collect();
        for key in &keys {
            assert!(
                !key.is_empty() && key.chars().all(|c| c.is_ascii_lowercase() || c == '_'),
                "{key}"
            );
        }
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), DSE_ITEMS.len(), "keys are unique");
    }

    #[test]
    fn an_old_row_finds_its_key_by_its_text_and_a_custom_item_has_none() {
        assert_eq!(
            key_for_text("Chair adjustable, with lower-back support"),
            Some("chair")
        );
        assert_eq!(key_for_text("Standing desk set up"), None);
        assert_eq!(
            key_for_text("chair adjustable, with lower-back support"),
            None
        );
        assert_eq!(
            display_key(None, "Leg room clear; feet supported").as_deref(),
            Some("leg_room")
        );
        assert_eq!(
            display_key(Some("chair"), "anything").as_deref(),
            Some("chair")
        );
        assert_eq!(display_key(None, "Standing desk set up"), None);
    }

    /// The migration that backfills `item_key` holds the same pairs as this table, so an old row
    /// gets the key a new one would.
    #[test]
    fn the_backfill_migration_holds_the_same_pairs() {
        // compared without whitespace or a trailing comma, so rustfmt may wrap the pairs as it likes
        let squeeze = |s: &str| s.split_whitespace().collect::<String>().replace(",)", ")");
        let sql = squeeze(include_str!(
            "../../migration/src/m20261030_000111_ergonomic_item_keys.rs"
        ));
        for item in DSE_ITEMS {
            assert!(
                sql.contains(&squeeze(&format!("(\"{}\", \"{}\")", item.key, item.text))),
                "{item:?} is not in the backfill"
            );
        }
    }

    #[test]
    fn completion_requires_every_answer() {
        assert!(may_complete(&[Some(true), Some(false)]).is_ok());
        let err = may_complete(&[Some(true), None, None]).unwrap_err();
        assert!(err.contains('2'), "counts the unanswered");
        assert!(
            may_complete(&[]).is_ok(),
            "an empty checklist has nothing unanswered"
        );
    }

    #[test]
    fn open_issues_counts_flagged_only() {
        assert_eq!(
            open_issues(&[Some(true), Some(false), None, Some(false)]),
            2
        );
        assert_eq!(open_issues(&[Some(true), None]), 0);
    }
}
