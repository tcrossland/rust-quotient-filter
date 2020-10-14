#[macro_use]
extern crate log;
extern crate byteorder;
extern crate log4rs;
extern crate uuid;

mod quotient;

use quotient::filter::{Filter, QuotientFilter};
use quotient::hash::{QuotientRemainder, QUOT_SIZE};
use quotient::slot::Slot;
use quotient::stats::{Stats, StatsCollector};
use uuid::Uuid;

fn main() {
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();
    let size: usize = (1 << QUOT_SIZE) + 1000;
    info!("Starting Quotient Filter example ({} slots)...", size);
    process(size, 100_000_000);
}

fn insert_uuid(filter: &mut QuotientFilter, uuid: Uuid) {
    let qr = &uuid.hash();
    let (idx, val) = (qr.0 as usize, qr.1);
    match filter.lookup(idx, val) {
        None => filter.put(idx, val),
        Some(s) => warn!(
            "Found value {} in slot 0x{:08x} {}",
            uuid.to_hyphenated_ref(),
            s.0,
            s.1.fmt()
        ),
    }
}

fn process(slots: usize, items: usize) {
    let filter: &mut QuotientFilter = &mut QuotientFilter::new(slots);

    let first_uuid = Uuid::new_v4();

    insert_uuid(filter, first_uuid);

    for i in 0..items + 1 {
        let u: Uuid = Uuid::new_v4();
        insert_uuid(filter, u);
        if i > 0 && i % 10_000_000 == 0 {
            info!("Processed {} UUIDs", i);
        }
    }

    info!(
        "Attempting insertion of duplicate value {}, warning is expected",
        first_uuid.to_hyphenated_ref()
    );
    insert_uuid(filter, first_uuid);

    info!("Collecting statistics (may take a few seconds)...");
    let s = filter
        .slots
        .iter()
        .enumerate()
        .fold(Stats::new(), |acc, (i, slot)| acc.collect(i, slot));

    s.print();
}
