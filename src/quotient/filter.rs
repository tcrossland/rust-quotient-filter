use crate::quotient::slot::{CONTINUATION, IntSlot, OCCUPIED, SHIFTED, Slot, SlotPair};

pub trait Filter<T> {
    fn new(size: usize) -> T;
    fn shift(&mut self, idx: usize);
    fn lookup_run(&self, q: usize) -> (usize, IntSlot);
    fn lookup(&self, q: usize, r: u32) -> Option<SlotPair>;
    fn put(&mut self, q: usize, r: u32);
    fn get(&self, q: usize) -> IntSlot;
}

pub struct QuotientFilter {
    // size: usize,
    pub slots: Vec<IntSlot>,
}

impl Filter<QuotientFilter> for QuotientFilter {
    fn new(size: usize) -> QuotientFilter {
        QuotientFilter {
            // size: size,
            slots: vec![0; size],
        }
    }

    fn shift(&mut self, idx: usize) {
        self.slots[idx + 1] = (self.slots[idx + 1] & OCCUPIED) | self.slots[idx].shift();
    }

    /// Returns the slot where quotient `q`'s run starts, or would start.
    /// `q` must be marked occupied.
    fn lookup_run(&self, q: usize) -> SlotPair {
        // Walk back to the start of the cluster.
        let mut b = q;
        while self.get(b).is_shifted() {
            b -= 1;
        }
        // Walk forward one run for each occupied quotient until reaching q.
        let mut s = b;
        while b != q {
            s += 1;
            while self.get(s).is_continuation() {
                s += 1;
            }
            b += 1;
            while !self.get(b).is_occupied() {
                b += 1;
            }
        }
        (s, self.get(s))
    }

    fn lookup(&self, q: usize, r: u32) -> Option<SlotPair> {
        if !self.get(q).is_occupied() {
            return None;
        }
        // Remainders within a run are kept in ascending order.
        let (mut i, mut s) = self.lookup_run(q);
        loop {
            if s.remainder() == r {
                return Some((i, s));
            }
            if s.remainder() > r {
                return None;
            }
            i += 1;
            s = self.get(i);
            if !s.is_continuation() {
                return None;
            }
        }
    }

    fn put(&mut self, q: usize, r: u32) {
        if self.get(q).is_empty() {
            self.slots[q] = r | OCCUPIED;
            return;
        }
        let was_occupied = self.get(q).is_occupied();
        self.slots[q] |= OCCUPIED;
        let (start, _) = self.lookup_run(q);

        // Find where r goes in q's run, keeping the run in ascending order.
        let mut i = start;
        if was_occupied {
            loop {
                let s = self.get(i);
                if s.remainder() == r {
                    return;
                }
                if s.remainder() > r {
                    break;
                }
                i += 1;
                if !self.get(i).is_continuation() {
                    break;
                }
            }
        }

        // Move the rest of the cluster one slot to the right to make room at i.
        let mut end = i;
        while !self.get(end).is_empty() {
            end += 1;
        }
        while end > i {
            end -= 1;
            self.shift(end);
        }

        let mut flags = 0;
        if i != q {
            flags |= SHIFTED;
        }
        if i != start {
            flags |= CONTINUATION;
        }
        self.slots[i] = (self.slots[i] & OCCUPIED) | r | flags;
        if was_occupied && i == start {
            // The run's previous first entry now follows r.
            self.slots[i + 1] |= CONTINUATION;
        }
    }

    fn get(&self, q: usize) -> IntSlot {
        self.slots[q]
    }
}

#[cfg(test)]
mod tests {
    use super::{Filter, QuotientFilter};
    use crate::quotient::slot::Slot;
    use std::collections::HashSet;

    const MAX_REMAINDER: u32 = (1 << 29) - 1;

    /// SplitMix64, so that a failing seed can be run again.
    fn next(state: &mut u64) -> u64 {
        *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = *state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    fn occupied_slots(filter: &QuotientFilter) -> usize {
        filter.slots.iter().filter(|s| !s.is_empty()).count()
    }

    /// Inserts the pairs in order, skipping any already stored, as `main` does.
    /// Returns the filter and the distinct pairs stored.
    fn filled(size: usize, pairs: &[(usize, u32)]) -> (QuotientFilter, HashSet<(usize, u32)>) {
        let mut filter = QuotientFilter::new(size);
        let mut stored = HashSet::new();
        for &(q, r) in pairs {
            let found = filter.lookup(q, r).is_some();
            assert_eq!(
                found,
                stored.contains(&(q, r)),
                "lookup({}, {:#x}) before put",
                q,
                r
            );
            if !found {
                filter.put(q, r);
                stored.insert((q, r));
            }
        }
        (filter, stored)
    }

    fn assert_matches_set(
        filter: &QuotientFilter,
        stored: &HashSet<(usize, u32)>,
        absent: &[(usize, u32)],
    ) {
        for &(q, r) in stored {
            assert!(
                filter.lookup(q, r).is_some(),
                "stored ({}, {:#x}) not found",
                q,
                r
            );
        }
        for &(q, r) in absent.iter().filter(|p| !stored.contains(p)) {
            assert!(
                filter.lookup(q, r).is_none(),
                "absent ({}, {:#x}) found",
                q,
                r
            );
        }
        assert_eq!(occupied_slots(filter), stored.len());
    }

    fn random_pairs(
        seed: u64,
        count: usize,
        quotients: usize,
        remainders: u32,
    ) -> Vec<(usize, u32)> {
        let mut state = seed;
        (0..count)
            .map(|_| {
                let x = next(&mut state);
                let q = (x % quotients as u64) as usize;
                let r = ((x >> 32) as u32) % remainders;
                (q, r)
            })
            .collect()
    }

    #[test]
    fn empty_filter_finds_nothing() {
        let filter = QuotientFilter::new(16);
        assert!(filter.lookup(0, 0).is_none());
        assert!(filter.lookup(3, MAX_REMAINDER).is_none());
    }

    #[test]
    fn one_run_inserted_in_descending_order() {
        let pairs: Vec<(usize, u32)> = (0..8).rev().map(|r| (4, r * 1000)).collect();
        let (filter, stored) = filled(32, &pairs);
        assert_matches_set(&filter, &stored, &[(4, 1), (4, 7001), (3, 0), (5, 0)]);
    }

    #[test]
    fn cluster_of_runs_inserted_out_of_order() {
        let pairs = [
            (2, 50),
            (1, 70),
            (3, 10),
            (1, 20),
            (2, 5),
            (3, 90),
            (1, 45),
            (0, 30),
        ];
        let (filter, stored) = filled(32, &pairs);
        assert_matches_set(
            &filter,
            &stored,
            &[(0, 31), (1, 46), (2, 6), (3, 11), (4, 10)],
        );
    }

    #[test]
    fn extreme_remainders() {
        let pairs = [(7, MAX_REMAINDER), (7, 0), (8, 0), (6, MAX_REMAINDER)];
        let (filter, stored) = filled(32, &pairs);
        assert_matches_set(&filter, &stored, &[(7, 1), (8, MAX_REMAINDER), (6, 0)]);
    }

    /// Wide remainders: nearly every pair is new, so this checks for misses and
    /// false matches between neighbouring runs as the filter fills up.
    #[test]
    fn random_pairs_match_a_set() {
        let quotients = 1024;
        for seed in 0..20 {
            let pairs = random_pairs(seed, 900, quotients, MAX_REMAINDER + 1);
            let (filter, stored) = filled(quotients + 1024, &pairs);
            let absent = random_pairs(seed + 1000, 2000, quotients, MAX_REMAINDER + 1);
            assert_matches_set(&filter, &stored, &absent);
        }
    }

    /// Narrow remainders: many repeats and near misses within the same run.
    #[test]
    fn random_pairs_with_few_remainders_match_a_set() {
        let quotients = 256;
        for seed in 0..20 {
            let pairs = random_pairs(seed, 400, quotients, 4);
            let (filter, stored) = filled(quotients + 1024, &pairs);
            let absent = random_pairs(seed + 1000, 1000, quotients, 4);
            assert_matches_set(&filter, &stored, &absent);
        }
    }
}
