use color_eyre::Result;
use std::{collections::HashMap, path::Path};

#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    derive_more::Deref,
    derive_more::DerefMut,
    derive_more::FromStr,
)]
struct Stone(u64);

impl Stone {
    fn n_digits(self) -> u32 {
        self.ilog10() + 1
    }

    /// Apply the first rule which matches
    ///
    /// Return the new stone which is split from this one, if applicable
    fn apply_rules(&mut self) -> Option<Self> {
        if **self == 0 {
            **self = 1;
            return None;
        }
        let digits = self.n_digits();
        if digits & 1 == 0 {
            let half_digits = digits / 2;
            let one_and_n_zeros = 10_u64.pow(half_digits);
            let right = **self % one_and_n_zeros;
            **self /= one_and_n_zeros;
            return Some(Stone(right));
        }
        **self *= 2024;
        None
    }
}

type StoneCounts = HashMap<Stone, u64>;

fn blink(out: impl IntoIterator<Item = (Stone, u64)>, into: &mut StoneCounts) {
    for (mut stone, count) in out.into_iter() {
        let next = stone.apply_rules();
        *into.entry(stone).or_default() += count;
        if let Some(next) = next {
            *into.entry(next).or_default() += count;
        }
    }
}

fn parse(input: &Path) -> Result<StoneCounts> {
    let mut out = StoneCounts::new();
    for stone in std::fs::read_to_string(input)?
        .split_whitespace()
        .map(|s| s.parse::<Stone>())
    {
        *out.entry(stone?).or_default() += 1;
    }
    Ok(out)
}

fn multiblink(mut stones: StoneCounts, n_blinks: u32) {
    let mut temp = StoneCounts::new();

    for _ in 0..n_blinks {
        blink(stones.drain(), &mut temp);
        std::mem::swap(&mut stones, &mut temp);
    }

    println!("after {n_blinks} blinks: {}", stones.values().sum::<u64>());
}

pub fn part1(input: &Path) -> Result<()> {
    let stones = parse(input)?;
    multiblink(stones, 25);
    Ok(())
}

pub fn part2(input: &Path) -> Result<()> {
    let stones = parse(input)?;
    multiblink(stones, 75);
    Ok(())
}

#[cfg(test)]
mod tests {
    mod part1 {
        use crate::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn short_example() {
            let input = [0, 1, 10, 99, 999]
                .into_iter()
                .map(|d| (Stone(d), 1))
                .collect::<StoneCounts>();
            let mut next = StoneCounts::new();
            blink(input, &mut next);

            next.retain(|_k, v| *v != 0);
            let expect = maplit::hashmap! {Stone(1) => 2, Stone(2024) => 1, Stone(0) => 1, Stone(9) => 2, Stone(2021976) => 1};
            assert_eq!(next, expect);
        }
    }
}
