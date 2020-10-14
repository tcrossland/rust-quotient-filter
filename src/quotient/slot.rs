pub const OCCUPIED: u32 = 1 << 31;
pub const CONTINUATION: u32 = 1 << 30;
pub const SHIFTED: u32 = 1 << 29;
pub const FLAGS: u32 = OCCUPIED | CONTINUATION | SHIFTED;
const NOT_EMPTY: u32 = OCCUPIED | CONTINUATION | SHIFTED;
const REMAINDER: u32 = (1 << 29) - 1;

pub trait Slot {
    fn is_run(&self) -> bool;
    fn is_cluster(&self) -> bool;
    fn is_occupied(&self) -> bool;
    fn is_continuation(&self) -> bool;
    fn is_shifted(&self) -> bool;
    fn is_empty(&self) -> bool;
    fn remainder(&self) -> u32;
    fn shift(&self) -> u32;
    fn fmt(&self) -> String;
    fn flags(&self) -> String;
}

impl Slot for u32 {
    fn is_run(&self) -> bool {
        !self.is_continuation() && !self.is_empty()
    }
    fn is_cluster(&self) -> bool {
        self.is_occupied() && !self.is_shifted() && !self.is_continuation()
    }
    fn is_occupied(&self) -> bool {
        self & OCCUPIED != 0
    }
    fn is_continuation(&self) -> bool {
        self & CONTINUATION != 0
    }
    fn is_shifted(&self) -> bool {
        self & SHIFTED != 0
    }
    fn is_empty(&self) -> bool {
        self & NOT_EMPTY == 0
    }
    fn remainder(&self) -> u32 {
        self & REMAINDER
    }
    fn shift(&self) -> u32 {
        (self & (OCCUPIED - 1)) | SHIFTED
    }
    fn flags(&self) -> String {
        let o: char = if self.is_occupied() { 'O' } else { '-' };
        let c: char = if self.is_continuation() { 'C' } else { '-' };
        let s: char = if self.is_shifted() { 'S' } else { '-' };
        format!("{}{}{}", o, c, s)
    }
    fn fmt(&self) -> String {
        let o: char = if self.is_occupied() { 'O' } else { '-' };
        let c: char = if self.is_continuation() { 'C' } else { '-' };
        let s: char = if self.is_shifted() { 'S' } else { '-' };
        format!("{}{}{}:0x{:08x}", o, c, s, self.remainder())
    }
}

pub type IntSlot = u32;

pub type SlotPair = (usize, IntSlot);

pub type Run = Vec<SlotPair>;

pub type Cluster = Vec<Run>;
