use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;
use uuid::Bytes;
use uuid::Uuid;

/// Default number of hash bits used as the quotient (the slot index).
pub const QUOT_SIZE: u8 = 27;
/// Default number of hash bits stored in each slot.
pub const REM_SIZE: u8 = 29;
/// A slot has room for at most this many remainder bits.
pub const MAX_REM_SIZE: u8 = 29;

fn hash(bytes: &Bytes, size: u8) -> u64 {
    let mut rdr = Cursor::new(bytes);
    let msb: u64 = rdr.read_u64::<BigEndian>().unwrap();
    let lsb: u64 = rdr.read_u64::<BigEndian>().unwrap();
    let hash: u64 = msb ^ lsb;
    if size >= 64 {
        hash
    } else {
        hash & ((1 << size) - 1)
    }
}

pub trait QuotientRemainder {
    /// Splits the hash into a quotient of `quot_size` bits and a remainder
    /// of `rem_size` bits.
    fn hash(&self, quot_size: u8, rem_size: u8) -> (u32, u32);
}

impl QuotientRemainder for String {
    fn hash(&self, quot_size: u8, rem_size: u8) -> (u32, u32) {
        let u = Uuid::parse_str(self).unwrap();
        u.hash(quot_size, rem_size)
    }
}

impl QuotientRemainder for Uuid {
    fn hash(&self, quot_size: u8, rem_size: u8) -> (u32, u32) {
        let h: u64 = hash(self.as_bytes(), quot_size + rem_size);
        let q = (h >> rem_size) as u32;
        let r = (h & ((1 << rem_size) - 1)) as u32;
        (q, r)
    }
}

#[cfg(test)]
mod tests {
    use super::{QUOT_SIZE, QuotientRemainder, REM_SIZE};
    use uuid::Uuid;

    #[test]
    fn splits_the_low_bits_of_the_xored_halves() {
        // msb ^ lsb = 0x0123_4567_89ab_cdef
        let u = Uuid::from_u128(0x0123_4567_89ab_cdef_0000_0000_0000_0000);
        let h: u64 = 0x0123_4567_89ab_cdef;
        assert_eq!(u.hash(8, 4), (((h >> 4) & 0xff) as u32, (h & 0xf) as u32));
        let h56 = h & ((1 << 56) - 1);
        assert_eq!(
            u.hash(QUOT_SIZE, REM_SIZE),
            ((h56 >> 29) as u32, (h56 & ((1 << 29) - 1)) as u32)
        );
    }
}
