# Changelog

## Unreleased

## 0.3.0 - 2026-11-22

### Added
- Options for the number of quotient and remainder bits, the number of UUIDs and the seed (`--help` lists them).
- UUIDs come from a seeded generator, and each run logs its seed, so `--seed` repeats a run exactly.
- The run logs the number of false matches.
- The README explains the prototype, how to run it and what it showed.
- README diagrams: a worked example of runs and clusters, how a UUID becomes a quotient and a remainder in a slot, a Bloom filter's lookup beside the quotient filter's, and a flowchart of how a run handles each ID.

### Fixed
- `put` could file an entry in another quotient's run, where it could no longer be found.
- `lookup` could match against a neighbouring quotient's run.
- The slot statistics missed clusters that end at an empty slot, so the longest cluster was under-reported.

### Changed
- Moves to the 2024 edition, with Rust 1.86 or newer required.
- Upgrades log4rs to 1.4.
- Commits `Cargo.lock`.
- `Cargo.toml` lists the author without an email address, and the licence covers 2016-2026.

## 0.2.0 - 2020-10-14
