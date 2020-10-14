use quotient::slot::{IntSlot, Slot, SlotPair, CONTINUATION, OCCUPIED, SHIFTED};

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

    fn lookup_run(&self, q: usize) -> SlotPair {
        let mut s: IntSlot = self.get(q);
        let mut i: usize = q;
        let mut runs: i32 = 0;
        if s.is_empty() {
            (q, s)
        } else {
            let mut run_stack: Vec<usize> = vec![];
            while s.is_shifted() {
                if !s.is_continuation() {
                    run_stack.push(i);
                }
                s = self.get(i);
                i -= 1;
                if s.is_occupied() {
                    runs += 1;
                }
            }
            while runs > 0 && !run_stack.is_empty() {
                i = run_stack.pop().unwrap();
                runs -= 1;
            }
            while runs > 0 {
                i += 1;
                s = self.get(i);
                if !s.is_continuation() {
                    runs -= 1;
                }
            }
            s = self.get(i);
            (i, s)
        }
    }

    fn lookup(&self, q: usize, r: u32) -> Option<SlotPair> {
        let (run, mut s) = self.lookup_run(q);
        if s.is_empty() {
            None
        } else {
            let (mut i, mut watermark) = (run, 0);
            while !s.is_empty() && s.remainder() < r && watermark == 0 {
                i += 1;
                s = self.get(i);
                if !s.is_continuation() {
                    watermark = i
                }
            }
            if watermark == 0 && s.remainder() == r {
                Some((i, s))
            } else {
                None
            }
        }
    }

    fn put(&mut self, q: usize, r: u32) {
        let (run, mut s) = self.lookup_run(q);
        let (mut i, mut watermark) = (run, run);
        if !s.is_empty() {
            while !s.is_empty() {
                i += 1;
                s = self.get(i);
                if !s.is_continuation() {
                    watermark = i
                }
            }
            while i > watermark {
                i -= 1;
                self.shift(i);
            }
            while i > run && self.get(i - 1).remainder() > r {
                i -= 1;
                self.shift(i);
                if i == run {
                    self.slots[i + 1] |= CONTINUATION;
                }
            }
        }
        if i == q {
            self.slots[i] = r | OCCUPIED;
        } else if i > run {
            self.slots[i] = (self.slots[i] & OCCUPIED) | r | SHIFTED | CONTINUATION;
            self.slots[q] |= OCCUPIED;
        } else {
            self.slots[i] = (self.slots[i] & OCCUPIED) | r | SHIFTED;
            self.slots[q] |= OCCUPIED;
        }
    }

    fn get(&self, q: usize) -> IntSlot {
        self.slots[q]
    }
}
