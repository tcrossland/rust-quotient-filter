# Quotient Filter Prototype
A [Quotient Filter](https://en.wikipedia.org/wiki/Quotient_filter) prototype written in [Rust](https://www.rust-lang.org/en-US/documentation.html).
A quotient filter answers the same question as the better-known [Bloom filter](https://en.wikipedia.org/wiki/Bloom_filter), whether an item is definitely not in a set or probably is, in comparable space; unlike a Bloom filter, it can be resized, or merged into a larger filter, without rehashing the original keys, so it can grow with its set, and items can be deleted. This prototype implements insertion and lookup only.
I built it in 2016 as an experiment in idempotency for an append-only, reactive Kafka architecture: checking each message ID against the filter before processing it.
Running it tries to insert 100 million pseudo-random UUIDs, generated from a seed it logs, into the filter. It logs a warning for every false match (a new UUID the filter reports as already present), then the number of false matches and slot statistics.
At the default size it held 100 million IDs in about 540 MB, around 5.4 bytes each. Of 20 seeded runs (seeds 1 to 20), 17 had no false matches and 3 had one: 3 in all, against about 1.4 expected (about 0.07 per run).

## Usage instructions
1. [Install Rust](https://www.rust-lang.org/en-US/install.html) 1.86 or newer, which includes cargo
2. Clone this repository
3. `cargo run --release` (the program runs significantly slower unless [compiler optimizations](https://doc.rust-lang.org/book/ch14-01-release-profiles.html) are enabled)
4. `cargo run --release -- --help` lists the options: the number of quotient and remainder bits, the number of UUIDs and the seed. Each run logs its seed, and `--seed` repeats a run exactly.

## Findings
The filter splits each UUID's hash into a quotient, which picks the slot, and a remainder, which is stored in it. More remainder bits mean fewer false matches; more quotient bits mean more slots. A quotient's remainders sit in adjacent slots, called a run. A cluster starts with a run in its own slot and takes in the runs pushed along behind it; a lookup may have to scan a cluster from its start.

With the defaults (27-bit quotients, 29-bit remainders) and seed 1, a run inserted 100,000,000 UUIDs into 134,218,728 slots (2^27, plus 1,000 spare at the end because the filter doesn't wrap around), about 75% full, with no false matches. About 0.07 false matches are expected at this size, so this shows false matches are rare rather than measuring their rate. The run took 10 seconds and peaked at about 540 MB of memory (measured with `/usr/bin/time -l`; Rust 1.100.0, Apple M2, November 2026). Inserting the IDs took about 6.6 seconds of that (from the log's timestamps), about 66 ns per ID on one thread, including generating each UUID and looking it up first; collecting the slot statistics took the rest.

By the end of that run, the chance of a false match on a lookup is about 1.4 × 10⁻⁹ (the load, 0.745 IDs per quotient, divided by 2^29). A Bloom filter with that rate needs about 42 bits (1.44 × log2(1/p) bits per item for a false match rate p, the standard Bloom filter sizing), or 5.3 bytes, per ID, against 5.4 bytes here, so the case for a quotient filter rests on merging, resizing, deletion and locality (a lookup reads one stretch of adjacent slots, where a Bloom filter at this rate reads up to 29 scattered bits, about 2 on average for a new ID) rather than on size.

With seed 1 the longest cluster held 221 runs, and the longest run 11 slots. Across seeds 1 to 20 the longest cluster ranged from 143 to 229 runs and the longest run from 9 to 11 slots, so seed 1's longest cluster is near the top of that range. A lookup may scan a whole cluster, so lookups can slow down as the filter fills.

With 16-bit quotients and 8-bit remainders, 50,000 UUIDs (seed 1) gave 76 false matches, against about 74.5 expected.
For n UUIDs, q quotient bits and r remainder bits, the expected number of false matches is about n² / (2 × 2^q × 2^r). `cargo run --release -- --quotient-bits 16 --remainder-bits 8 --items 50000 --seed 1` repeats this run.

Earlier runs used a filter whose `put` could file an entry in another quotient's run, where it could no longer be found, and whose `lookup` could match against a neighbouring quotient's run. A 100-million run of that filter also reported no false matches, since the program looks up only new IDs, apart from one deliberate duplicate, and so never looked for an entry filed in the wrong run; only the unit tests in `src/quotient/filter.rs` (`cargo test`), which check every lookup against a `HashSet`, found both bugs. The figures above are from the fixed filter.

The filter has a fixed number of slots, while a topic's set of message IDs keeps growing, and the expected number of false matches grows with the square of the number of IDs. A consumer would need to bound the set, for example with one filter per time window, merged or dropped as windows age. Quotient filters can be merged and resized, but this prototype implements neither.

For checking message IDs before processing them, I found it worked well at this scale: 100 million IDs in about 540 MB, at about 66 ns each. These figures assume random IDs: the hash XORs the two halves of each UUID (`src/quotient/hash.rs`), which is uniform when either half is random, as in version 4 and version 7 UUIDs, but not for version 1 UUIDs, whose second half is the same for every ID from one generator, so nearby IDs would share remainders and collide far more often.
