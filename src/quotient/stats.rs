use quotient::slot::{Cluster, IntSlot, Run, Slot, FLAGS};
use std::collections::btree_map::BTreeMap;

pub trait StatsCollector {
    fn new() -> Stats;
    fn collect(self, usize, &IntSlot) -> Stats;
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
    fn new() -> Stats {
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
            if slot.is_cluster() {
                self.clusters += 1;
                self.runs += 1;
                if self.current_cluster.len() > self.longest_cluster.len() {
                    self.longest_cluster = self.current_cluster.to_vec();
                }
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
