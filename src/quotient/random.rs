use uuid::{Builder, Uuid, Variant, Version};

/// SplitMix64: a small, fast generator. The same seed gives the same
/// sequence, so a run can be repeated.
pub fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// A version 4 UUID made from the generator's next 128 bits.
pub fn random_uuid(state: &mut u64) -> Uuid {
    let bits = (u128::from(splitmix64(state)) << 64) | u128::from(splitmix64(state));
    Builder::from_u128(bits)
        .set_variant(Variant::RFC4122)
        .set_version(Version::Random)
        .build()
}

#[cfg(test)]
mod tests {
    use super::{random_uuid, splitmix64};
    use uuid::{Variant, Version};

    #[test]
    fn splitmix64_matches_reference_values() {
        // First outputs for seed 0 from the reference implementation.
        let mut state = 0;
        assert_eq!(splitmix64(&mut state), 0xe220_a839_7b1d_cdaf);
        assert_eq!(splitmix64(&mut state), 0x6e78_9e6a_a1b9_65f4);
    }

    #[test]
    fn same_seed_gives_same_uuids() {
        let (mut a, mut b) = (42, 42);
        for _ in 0..3 {
            assert_eq!(random_uuid(&mut a), random_uuid(&mut b));
        }
    }

    #[test]
    fn uuids_are_version_4() {
        let mut state = 7;
        let u = random_uuid(&mut state);
        assert_eq!(u.get_version(), Some(Version::Random));
        assert_eq!(u.get_variant(), Some(Variant::RFC4122));
    }
}
