//! An absent optional text, group or list is an empty one (RCA-P-3).
//!
//! `Option::unwrap_or_default` is disallowed (RCA-ENF-2) because it turns an unknown *figure* into a zero. A group that has no entry
//! in a lookup, an optional free-text field that was not sent, or a list that does not exist yet are known to be empty: there is
//! nothing unknown in them. This names that case, so a reader can tell it from a figure that was defaulted.
//!
//! Never use it for a number, a count, an amount or a date: those are `None` (unknown) until the specification declares a fallback.

/// `or_empty`: the value, or an empty one of the same type when it is absent.
pub trait AbsentIsEmpty<T> {
    /// The value, or `T::default()` (an empty collection or text) when it is absent.
    fn or_empty(self) -> T;
}

impl<T: Default> AbsentIsEmpty<T> for Option<T> {
    fn or_empty(mut self) -> T {
        std::mem::take(self.get_or_insert_with(T::default))
    }
}

#[cfg(test)]
mod tests {
    use super::AbsentIsEmpty;

    #[test]
    fn an_absent_collection_or_text_is_empty_and_a_present_one_is_kept() {
        assert_eq!(None::<Vec<u8>>.or_empty(), Vec::<u8>::new());
        assert_eq!(Some(vec![1_u8]).or_empty(), vec![1_u8]);
        assert_eq!(None::<String>.or_empty(), "");
        assert_eq!(Some("a".to_string()).or_empty(), "a");
    }
}
