//! An opt-in dictionary of English given-name classes (Michael / Mike / Mick).
//!
//! A class is a set of names the caller considers interchangeable. The built-in
//! table is deliberately small and English-only; extend it with
//! [`NicknameTable::with_class`] for the names your workforce uses.

use crate::normalizer::fold;

const ENGLISH: &[&[&str]] = &[
    &["alexander", "alex", "sandy", "xander"],
    &["andrew", "andy", "drew"],
    &["anthony", "tony"],
    &["benjamin", "ben", "benny"],
    &[
        "catherine",
        "katherine",
        "kathryn",
        "cathy",
        "kathy",
        "kate",
        "katie",
    ],
    &["charles", "charlie", "chuck"],
    &["christopher", "chris", "kit"],
    &["daniel", "dan", "danny"],
    &["david", "dave", "davie"],
    &["edward", "ed", "eddie", "ted", "ned"],
    &["elizabeth", "liz", "lizzie", "beth", "betty", "eliza"],
    &["james", "jim", "jimmy", "jamie"],
    &["jennifer", "jen", "jenny"],
    &["john", "jon", "johnny", "jack"],
    &["jonathan", "jon", "jonny"],
    &["joseph", "joe", "joey"],
    &["margaret", "maggie", "meg", "peggy", "marge"],
    &["matthew", "matt", "matty"],
    &["michael", "mike", "mick", "mickey"],
    &["nicholas", "nick", "nicky"],
    &["patricia", "pat", "patty", "trish"],
    &["richard", "rick", "ricky", "dick", "rich"],
    &["robert", "rob", "robbie", "bob", "bobby", "bert"],
    &["samuel", "sam", "sammy"],
    &["stephen", "steven", "steve"],
    &["susan", "sue", "suzy", "susie"],
    &["thomas", "tom", "tommy"],
    &["timothy", "tim", "timmy"],
    &["victoria", "vicky", "vicki", "tori"],
    &["william", "will", "bill", "billy", "liam"],
];

/// Classes of given names that count as the same name.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NicknameTable {
    classes: Vec<Vec<String>>,
}

impl NicknameTable {
    /// No classes: only identical names are nicknames of each other.
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// The built-in English classes.
    #[must_use]
    pub fn english() -> Self {
        Self {
            classes: ENGLISH
                .iter()
                .map(|c| c.iter().map(std::string::ToString::to_string).collect())
                .collect(),
        }
    }

    /// Add a class. Names are folded, so `"Siân"` and `"sian"` are the same entry.
    #[must_use]
    pub fn with_class<I, S>(mut self, names: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let class: Vec<String> = names.into_iter().filter_map(|n| fold(n.as_ref())).collect();
        if class.len() > 1 {
            self.classes.push(class);
        }
        self
    }

    /// Whether two *folded* given names are in a shared class and are not equal.
    #[must_use]
    pub fn related(&self, a: &str, b: &str) -> bool {
        a != b
            && self
                .classes
                .iter()
                .any(|c| c.iter().any(|n| n == a) && c.iter().any(|n| n == b))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_classes_relate_both_ways() {
        let t = NicknameTable::english();
        assert!(t.related("michael", "mike"));
        assert!(t.related("mike", "michael"));
        assert!(t.related("liz", "elizabeth"));
        assert!(!t.related("mike", "mike"));
        assert!(!t.related("mike", "bob"));
    }

    #[test]
    fn empty_table_relates_nothing() {
        assert!(!NicknameTable::empty().related("michael", "mike"));
    }

    #[test]
    fn custom_classes_are_folded() {
        let t = NicknameTable::empty().with_class(["Siân", "Shan"]);
        assert!(t.related("sian", "shan"));
    }

    #[test]
    fn one_name_is_not_a_class() {
        assert_eq!(
            NicknameTable::empty().with_class(["Solo"]),
            NicknameTable::empty()
        );
    }
}
