use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;
use uuid::Bytes;
use uuid::Uuid;

pub const HASH_SIZE: u8 = 56;
pub const QUOT_SIZE: u8 = 27;
const REM_SIZE: u8 = HASH_SIZE - QUOT_SIZE;

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
    fn hash(&self) -> (u32, u32);
}

impl QuotientRemainder for String {
    fn hash(&self) -> (u32, u32) {
        let u = Uuid::parse_str(&self).unwrap();
        u.hash()
    }
}

impl QuotientRemainder for Uuid {
    fn hash(&self) -> (u32, u32) {
        let h: u64 = hash((*self).as_bytes(), HASH_SIZE);
        let q = (h >> REM_SIZE) as u32;
        let r = (h ^ ((q as u64) << REM_SIZE)) as u32;
        // println!("{:056b}", h);
        // println!("{:027b}/{:029b}", q, r);
        (q, r)
    }
}
