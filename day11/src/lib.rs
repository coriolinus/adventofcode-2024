use color_eyre::Result;
use std::path::Path;

#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
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

fn blink(out: impl IntoIterator<Item = Stone> + ExactSizeIterator, into: &mut Vec<Stone>) {
    // ensure `into` has capacity at least 1.5x `out.len()`
    let need_capacity = out.len() * 3 / 2 - into.capacity();
    into.reserve(need_capacity);

    for mut stone in out.into_iter() {
        let next = stone.apply_rules();
        into.push(stone);
        if let Some(next) = next {
            into.push(next);
        }
    }
}

pub fn part1(input: &Path) -> Result<()> {
    let mut stones = std::fs::read_to_string(input)?
        .split_whitespace()
        .map(|s| s.parse::<Stone>())
        .collect::<Result<Vec<_>, _>>()?;
    let mut temp = Vec::new();

    for _ in 0..25 {
        blink(stones.drain(..), &mut temp);
        std::mem::swap(&mut stones, &mut temp);
    }

    println!("after 25 blinks: {}", stones.len());

    Ok(())
}

pub fn part2(input: &Path) -> Result<()> {
    let mut stones = std::fs::read_to_string(input)?
        .split_whitespace()
        .map(|s| s.parse::<Stone>())
        .collect::<Result<Vec<_>, _>>()?;
    let mut temp = Vec::new();

    for _ in 0..75 {
        blink(stones.drain(..), &mut temp);
        std::mem::swap(&mut stones, &mut temp);
    }

    println!("after 75 blinks: {}", stones.len());

    Ok(())
}
