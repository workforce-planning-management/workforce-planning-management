//! Small-cell suppression for counts shown as analytics (WPM-D35). DB-free.
//!
//! A count under the **floor** is hidden: a group of one or two says who they are. The rule is written once, here, so an endpoint does
//! not copy it. A zero reveals nobody and is shown. A floor of 0 is read as 1.
//!
//! **A hidden cell must not be recoverable by subtraction.** If a table shows its cells and a total, and exactly one cell is hidden, the
//! reader gets it by taking the shown cells from the total. [`with_total`] therefore hides the smallest remaining cell as well
//! (*complementary suppression*), or the total itself when there is nothing else to hide. This stops the exact figure being derived;
//! it does not stop a reader narrowing it to a range, and it does not defend against two tables that overlap (that needs a person's
//! judgement about which tables to publish together).

/// Whether a count may be shown at the floor. Zero is shown; any other count under the floor is not.
#[must_use]
pub fn visible(count: usize, floor: usize) -> bool {
    count == 0 || count >= floor.max(1)
}

/// A table of counts after suppression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suppressed<K> {
    /// Each cell with its count, or `None` where hidden, in the order given.
    pub cells: Vec<(K, Option<usize>)>,
    /// The total, or `None` where it would let a hidden cell be worked out, or is itself under the floor.
    pub total: Option<usize>,
}

/// Hide the cells under the floor, then protect the total from subtraction.
#[must_use]
pub fn with_total<K: Clone>(cells: &[(K, usize)], floor: usize) -> Suppressed<K> {
    let mut shown: Vec<bool> = cells.iter().map(|(_, n)| visible(*n, floor)).collect();
    let total: usize = cells.iter().map(|(_, n)| *n).sum();
    let hidden = shown.iter().filter(|s| !**s).count();
    let mut show_total = visible(total, floor);
    if hidden == 1 && show_total {
        // Hide the smallest other cell that is not zero; with none, the total itself would give the hidden cell away.
        let next = cells
            .iter()
            .enumerate()
            .filter(|(i, (_, n))| shown[*i] && *n > 0)
            .min_by_key(|(_, (_, n))| *n)
            .map(|(i, _)| i);
        match next {
            Some(i) => shown[i] = false,
            None => show_total = false,
        }
    }
    Suppressed {
        cells: cells
            .iter()
            .zip(&shown)
            .map(|((k, n), s)| (k.clone(), s.then_some(*n)))
            .collect(),
        total: show_total.then_some(total),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cells(counts: &[usize]) -> Vec<(usize, usize)> {
        counts.iter().copied().enumerate().collect()
    }

    fn hidden(s: &Suppressed<usize>) -> usize {
        s.cells.iter().filter(|(_, n)| n.is_none()).count()
    }

    #[test]
    fn a_count_under_the_floor_is_hidden_and_zero_is_not() {
        assert!(!visible(1, 10) && !visible(9, 10));
        assert!(visible(10, 10) && visible(11, 10));
        assert!(visible(0, 10), "a zero names nobody");
        assert!(visible(1, 0), "a floor of 0 is read as 1");
        assert!(visible(0, 0));
    }

    #[test]
    fn a_table_with_nothing_hidden_shows_everything() {
        let s = with_total(&cells(&[12, 30, 10]), 10);
        assert_eq!(
            s.cells.iter().map(|(_, n)| *n).collect::<Vec<_>>(),
            [Some(12), Some(30), Some(10)]
        );
        assert_eq!(s.total, Some(52));
    }

    #[test]
    fn two_hidden_cells_need_no_more_hiding() {
        let s = with_total(&cells(&[3, 4, 40]), 10);
        assert_eq!(
            s.cells.iter().map(|(_, n)| *n).collect::<Vec<_>>(),
            [None, None, Some(40)]
        );
        assert_eq!(
            s.total,
            Some(47),
            "two unknowns cannot be separated by subtraction"
        );
    }

    #[test]
    fn one_hidden_cell_costs_the_smallest_other_cell_too() {
        let s = with_total(&cells(&[3, 40, 12, 25]), 10);
        assert_eq!(
            s.cells.iter().map(|(_, n)| *n).collect::<Vec<_>>(),
            [None, Some(40), None, Some(25)],
            "12 is the smallest other"
        );
        assert_eq!(s.total, Some(80));
    }

    #[test]
    fn a_zero_is_never_the_cell_that_gets_hidden_as_cover() {
        let s = with_total(&cells(&[3, 0, 40]), 10);
        assert_eq!(
            s.cells.iter().map(|(_, n)| *n).collect::<Vec<_>>(),
            [None, Some(0), None],
            "40 is hidden, not the zero"
        );
    }

    #[test]
    fn a_lone_small_cell_hides_the_total_as_well() {
        let s = with_total(&cells(&[3]), 10);
        assert_eq!((s.cells[0].1, s.total), (None, None));
        let only_zeros_beside = with_total(&cells(&[0, 3]), 10);
        assert_eq!(
            only_zeros_beside.total, None,
            "the total is the hidden cell"
        );
    }

    #[test]
    fn a_total_under_the_floor_is_hidden() {
        let s = with_total(&cells(&[0, 0]), 10);
        assert_eq!(s.total, Some(0), "an empty table says so");
        let t = with_total(&cells(&[4, 4]), 10);
        assert_eq!(t.total, None, "eight is under the floor of ten");
    }

    /// For every table of up to five cells with counts 0 to 6 and floors 1 to 4: a shown count is a real one over the floor or zero,
    /// and a shown total never leaves exactly one hidden cell to be found by subtraction.
    #[test]
    fn no_hidden_cell_can_be_found_by_subtraction() {
        fn all(len: usize, max: usize) -> Vec<Vec<usize>> {
            if len == 0 {
                return vec![vec![]];
            }
            let shorter = all(len - 1, max);
            (0..=max)
                .flat_map(|n| shorter.iter().map(move |t| [vec![n], t.clone()].concat()))
                .collect()
        }
        let mut checked = 0;
        for len in 1..=5 {
            for table in all(len, 6) {
                for floor in 1..=4 {
                    let s = with_total(&cells(&table), floor);
                    for ((_, shown), (_, real)) in s.cells.iter().zip(cells(&table)) {
                        if let Some(n) = shown {
                            assert_eq!(*n, real, "a shown count is the real one");
                            assert!(visible(*n, floor), "{table:?} floor {floor}");
                        }
                    }
                    if s.total.is_some() {
                        assert_ne!(
                            hidden(&s),
                            1,
                            "{table:?} floor {floor}: one hidden cell and a total"
                        );
                    }
                    // Every cell under the floor and not zero is hidden.
                    for ((_, shown), (_, real)) in s.cells.iter().zip(cells(&table)) {
                        if real != 0 && real < floor {
                            assert!(
                                shown.is_none(),
                                "{table:?} floor {floor}: a small cell leaked"
                            );
                        }
                    }
                    checked += 1;
                }
            }
        }
        assert!(checked > 50_000, "{checked} tables checked");
    }

    /// The rule files that still hold their own copy of the floor comparison. **Only ever shrink this list**: move a file to
    /// [`visible`] or [`with_total`] and delete its name. A new file with its own copy fails the test below.
    const OWN_COPY: &[&str] = &[
        "appraisal.rs",
        "bargaining.rs",
        "bargaining_pack.rs",
        "derived.rs",
        "engagement.rs",
        "equality_monitoring.rs",
        "equipment.rs",
        "funding.rs",
        "health_requirements.rs",
        "human_capital.rs",
        "org_shape.rs",
        "pay_gap.rs",
        "pay_information.rs",
        "pay_range.rs",
        "policy.rs",
        "programmes.rs",
        "resignation.rs",
    ];

    fn has_own_copy(source: &str) -> bool {
        source.lines().any(|line| {
            let code = line.split("//").next().unwrap_or("");
            ["< floor", ">= floor", "floor.max(1)", ">= GROUP_FLOOR"]
                .iter()
                .any(|p| code.contains(p))
        })
    }

    /// A new rule that compares a count with the floor by hand must use `suppression` instead (WPM-D35).
    #[test]
    fn no_new_rule_file_writes_its_own_floor_comparison() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/rules");
        let mut found: Vec<String> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "rs"))
            .filter(|p| p.file_name().is_some_and(|n| n != "suppression.rs"))
            .filter(|p| has_own_copy(&std::fs::read_to_string(p).unwrap()))
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        found.sort();
        let new: Vec<_> = found
            .iter()
            .filter(|f| !OWN_COPY.contains(&f.as_str()))
            .collect();
        assert!(
            new.is_empty(),
            "use rules::suppression instead of a hand-written floor comparison in: {new:?}"
        );
        let stale: Vec<_> = OWN_COPY
            .iter()
            .filter(|f| !found.iter().any(|x| x == *f))
            .collect();
        assert!(
            stale.is_empty(),
            "these no longer hold their own copy; remove them from OWN_COPY: {stale:?}"
        );
    }
}
