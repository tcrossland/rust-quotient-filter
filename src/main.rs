mod quotient;

use crate::quotient::filter::{Filter, QuotientFilter};
use crate::quotient::hash::{MAX_REM_SIZE, QUOT_SIZE, QuotientRemainder, REM_SIZE};
use crate::quotient::random::random_uuid;
use crate::quotient::slot::Slot;
use crate::quotient::stats::{Stats, StatsCollector};
use log::{info, warn};
use std::env;
use std::process;
use uuid::Uuid;

const USAGE: &str =
    "Usage: quotient [--quotient-bits N] [--remainder-bits N] [--items N] [--seed N]

  --quotient-bits N   hash bits used as the slot index, 1 to 32 (default 27)
  --remainder-bits N  hash bits stored in each slot, 1 to 29 (default 29)
  --items N           random UUIDs to insert, 1 to 2^quotient-bits (default 100000000)
  --seed N            seed for the UUID generator (default: chosen at random and logged)";

/// Spare slots after the last quotient, since the filter doesn't wrap around.
const SPARE_SLOTS: usize = 1000;

#[derive(Debug, PartialEq)]
struct Options {
    quotient_bits: u8,
    remainder_bits: u8,
    items: usize,
    seed: Option<u64>,
}

fn parse_options(args: &[String]) -> Result<Options, String> {
    let mut options = Options {
        quotient_bits: QUOT_SIZE,
        remainder_bits: REM_SIZE,
        items: 100_000_000,
        seed: None,
    };
    let mut rest = args.iter();
    while let Some(flag) = rest.next() {
        let value = rest.next().ok_or(format!("{} needs a value", flag))?;
        let invalid = |_| format!("{} {}: not a valid number", flag, value);
        match flag.as_str() {
            "--quotient-bits" => options.quotient_bits = value.parse().map_err(invalid)?,
            "--remainder-bits" => options.remainder_bits = value.parse().map_err(invalid)?,
            "--items" => options.items = value.parse().map_err(invalid)?,
            "--seed" => options.seed = Some(value.parse().map_err(invalid)?),
            _ => return Err(format!("unknown option {}", flag)),
        }
    }
    if !(1..=32).contains(&options.quotient_bits) {
        return Err("--quotient-bits must be 1 to 32".to_string());
    }
    if !(1..=MAX_REM_SIZE).contains(&options.remainder_bits) {
        return Err(format!("--remainder-bits must be 1 to {}", MAX_REM_SIZE));
    }
    if options.items < 1 || options.items > 1 << options.quotient_bits {
        return Err("--items must be 1 to 2^quotient-bits".to_string());
    }
    Ok(options)
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("{}", USAGE);
        return;
    }
    let options = parse_options(&args).unwrap_or_else(|message| {
        eprintln!("{}\n\n{}", message, USAGE);
        process::exit(2);
    });
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();
    // Without --seed, take one seed from the system's random source.
    let seed = options
        .seed
        .unwrap_or_else(|| Uuid::new_v4().as_u128() as u64);
    run(&options, seed);
}

/// Inserts the UUID unless the filter reports it as present, in which case
/// it logs the match. Returns whether there was a match.
fn insert_uuid(filter: &mut QuotientFilter, options: &Options, uuid: Uuid) -> bool {
    let (q, r) = uuid.hash(options.quotient_bits, options.remainder_bits);
    let idx = q as usize;
    match filter.lookup(idx, r) {
        None => {
            filter.put(idx, r);
            false
        }
        Some(s) => {
            warn!(
                "Found value {} in slot 0x{:08x} {}",
                uuid.to_hyphenated_ref(),
                s.0,
                s.1.fmt()
            );
            true
        }
    }
}

fn run(options: &Options, seed: u64) {
    let slots = (1 << options.quotient_bits) + SPARE_SLOTS;
    info!(
        "Starting Quotient Filter example ({} slots, {}-bit quotients, {}-bit remainders, {} UUIDs, seed {})...",
        slots, options.quotient_bits, options.remainder_bits, options.items, seed
    );
    let filter = &mut QuotientFilter::new(slots);
    let mut state = seed;

    let first_uuid = random_uuid(&mut state);
    insert_uuid(filter, options, first_uuid);

    let mut false_matches = 0;
    for n in 2..=options.items {
        if insert_uuid(filter, options, random_uuid(&mut state)) {
            false_matches += 1;
        }
        if n % 10_000_000 == 0 {
            info!("Processed {} UUIDs", n);
        }
    }
    info!("False matches: {}", false_matches);

    info!(
        "Attempting insertion of duplicate value {}, warning is expected",
        first_uuid.to_hyphenated_ref()
    );
    insert_uuid(filter, options, first_uuid);

    info!("Collecting statistics (may take a few seconds)...");
    let s = filter
        .slots
        .iter()
        .enumerate()
        .fold(Stats::new(), |acc, (i, slot)| acc.collect(i, slot));

    s.print();
}

#[cfg(test)]
mod tests {
    use super::{Options, parse_options};

    fn args(s: &str) -> Vec<String> {
        s.split_whitespace().map(String::from).collect()
    }

    #[test]
    fn defaults_match_the_original_run() {
        assert_eq!(
            parse_options(&[]),
            Ok(Options {
                quotient_bits: 27,
                remainder_bits: 29,
                items: 100_000_000,
                seed: None
            })
        );
    }

    #[test]
    fn reads_every_option() {
        assert_eq!(
            parse_options(&args(
                "--quotient-bits 16 --remainder-bits 8 --items 50000 --seed 7"
            )),
            Ok(Options {
                quotient_bits: 16,
                remainder_bits: 8,
                items: 50_000,
                seed: Some(7)
            })
        );
    }

    #[test]
    fn rejects_bad_input() {
        for bad in [
            "--quotient-bits",
            "--quotient-bits x",
            "--quotient-bits 0",
            "--quotient-bits 33",
            "--remainder-bits 30",
            "--items 0",
            "--quotient-bits 4 --items 17",
            "--colour red",
        ] {
            assert!(parse_options(&args(bad)).is_err(), "{}", bad);
        }
    }
}
