//! Priority-based fit: drop the least important items first when a row of
//! widths does not fit the available space.

/// Decides which items to keep so their widths (plus `gap` between kept
/// neighbours) fit in `available`. Items with `None` are never dropped. Among
/// droppable items the highest priority value is dropped first; ties drop the
/// rightmost first. Returns one keep flag per item, in input order.
pub fn fit_by_priority(
    widths: &[f32],
    drop_priority: &[Option<u8>],
    gap: f32,
    available: f32,
) -> Vec<bool> {
    debug_assert_eq!(widths.len(), drop_priority.len());

    let mut kept = vec![true; widths.len()];

    let total_width = |kept: &[bool]| -> f32 {
        let mut sum = 0.0;
        let mut count = 0;
        for (i, &k) in kept.iter().enumerate() {
            if k {
                sum += widths[i];
                count += 1;
            }
        }
        if count > 1 {
            sum += gap * (count - 1) as f32;
        }
        sum
    };

    while total_width(&kept) > available {
        // `max_by_key` returns the last of equally-maximum elements, which
        // gives the rightmost-first tie break for free.
        let victim = kept
            .iter()
            .enumerate()
            .filter(|&(i, &k)| k && drop_priority[i].is_some())
            .max_by_key(|&(i, _)| (drop_priority[i], i));
        match victim {
            Some((i, _)) => kept[i] = false,
            // No droppable item left; the caller clips what remains.
            None => break,
        }
    }

    kept
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_fit_keeps_all() {
        let widths = [5.0, 5.0, 5.0];
        let prio = [Some(1), Some(2), None];
        assert_eq!(
            fit_by_priority(&widths, &prio, 1.0, 100.0),
            vec![true, true, true]
        );
    }

    #[test]
    fn drops_highest_priority_first() {
        let widths = [10.0, 10.0, 10.0];
        let prio = [None, Some(1), Some(2)];
        assert_eq!(
            fit_by_priority(&widths, &prio, 1.0, 21.0),
            vec![true, true, false]
        );
    }

    #[test]
    fn ties_drop_rightmost_first() {
        let widths = [10.0, 10.0, 10.0];
        let prio = [Some(1), Some(1), Some(1)];
        // Fitting 15 requires dropping two of the three equal-priority items;
        // each drop should take the rightmost still-kept one.
        assert_eq!(
            fit_by_priority(&widths, &prio, 0.0, 15.0),
            vec![true, false, false]
        );
    }

    #[test]
    fn none_is_never_dropped() {
        let widths = [10.0, 10.0];
        let prio = [None, None];
        assert_eq!(fit_by_priority(&widths, &prio, 0.0, 0.0), vec![true, true]);
    }

    #[test]
    fn gap_counts_only_between_kept() {
        let widths = [10.0, 10.0, 10.0];
        let prio = [None, None, Some(1)];
        // Before the drop the gap is counted twice (3 kept items); after
        // dropping the last item it must be counted only once.
        assert_eq!(
            fit_by_priority(&widths, &prio, 5.0, 25.0),
            vec![true, true, false]
        );
    }

    #[test]
    fn empty_input() {
        assert_eq!(fit_by_priority(&[], &[], 1.0, 0.0), Vec::<bool>::new());
    }
}
