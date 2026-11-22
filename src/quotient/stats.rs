use crate::quotient::slot::{Cluster, FLAGS, IntSlot, Run, Slot};
use log::{info, warn};
use std::collections::btree_map::BTreeMap;

pub trait StatsCollector {
    fn new() -> Self;
    fn collect(self, pos: usize, slot: &IntSlot) -> Stats;
    fn print(&self);
}

pub struct Stats {
    flags: BTreeMap<IntSlot, usize>,
    clusters: usize,
    runs: usize,
    current_run: Run,
    longest_run: Run,
    current_cluster: Cluster,
    longest_cluster: Cluster,
}

impl StatsCollector for Stats {
    fn new() -> Self {
        Stats {
            flags: BTreeMap::new(),
            clusters: 0,
            runs: 0,
            current_run: Vec::new(),
            longest_run: Vec::new(),
            current_cluster: Vec::new(),
            longest_cluster: Vec::new(),
        }
    }
    fn collect(mut self, pos: usize, slot: &IntSlot) -> Stats {
        *self.flags.entry(slot & FLAGS).or_insert(0) += 1;
        if slot.is_continuation() {
            self.current_run.push((pos, *slot));
        } else {
            if !self.current_run.is_empty() {
                self.current_cluster.push(self.current_run.to_vec());
                if self.current_run.len() > self.longest_run.len() {
                    self.longest_run = self.current_run.to_vec();
                }
            }
            // A cluster ends at the next cluster's start or at an empty slot.
            if (slot.is_cluster() || slot.is_empty())
                && self.current_cluster.len() > self.longest_cluster.len()
            {
                self.longest_cluster = self.current_cluster.to_vec();
            }
            if slot.is_cluster() {
                self.clusters += 1;
                self.runs += 1;
                self.current_cluster.clear();
                self.current_run.clear();
                self.current_run.push((pos, *slot));
            } else if slot.is_run() {
                self.runs += 1;
                self.current_run.clear();
                self.current_run.push((pos, *slot));
            } else if slot.is_empty() {
                self.current_run.clear();
                self.current_cluster.clear();
            } else {
                warn!("unexpected slot at {}: {}", pos, slot.fmt());
            }
        }
        self
    }
    fn print(&self) {
        info!("=========================== STATISTICS ===========================");
        info!("Slot Statistics (O=Occupied, C=Continuation, S=Shifted):");
        for (k, v) in &self.flags {
            info!(" {} {:>9}", k.flags(), v);
        }
        info!("Total Clusters: {}", self.clusters);
        info!("Total Runs: {}", self.runs);
        info!("Longest cluster: {} runs", self.longest_cluster.len());
        for (i, r) in self.longest_cluster.iter().enumerate() {
            for s in r.iter() {
                info!(" {:>3} 0x{:08x}:{}", i + 1, s.0, s.1.fmt());
            }
        }
        info!("Longest run: {} slots", self.longest_run.len());
        for s in self.longest_run.iter() {
            info!("     0x{:08x}:{}", s.0, s.1.fmt());
        }
        info!("==================================================================");
    }
}

#[cfg(test)]
mod tests {
    use super::{Stats, StatsCollector};
    use crate::quotient::filter::{Filter, QuotientFilter};

    fn stats(filter: &QuotientFilter) -> Stats {
        filter
            .slots
            .iter()
            .enumerate()
            .fold(Stats::new(), |acc, (i, slot)| acc.collect(i, slot))
    }

    #[test]
    fn counts_clusters_that_end_at_an_empty_slot() {
        let mut filter = QuotientFilter::new(32);
        // Quotients 1, 2 and 3 form one cluster in slots 1 to 4; slot 5 is empty.
        for (q, r) in [(1, 10), (1, 20), (2, 30), (3, 40), (10, 50)] {
            filter.put(q, r);
        }
        let s = stats(&filter);
        assert_eq!(s.clusters, 2);
        assert_eq!(s.runs, 4);
        assert_eq!(s.longest_cluster.len(), 3);
        assert_eq!(s.longest_run.len(), 2);
    }
}
