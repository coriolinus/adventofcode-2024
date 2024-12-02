use aoclib::parse;
use itertools::Itertools;
use std::{path::Path, str::FromStr};

struct Report {
    levels: Vec<i32>,
}

impl FromStr for Report {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let levels = s
            .split_whitespace()
            .map(str::parse)
            .collect::<Result<_, _>>()?;
        Ok(Self { levels })
    }
}

fn diffs_are_safe<Diffs>(diffs: Diffs) -> bool
where
    Diffs: IntoIterator<Item = i32>,
{
    let mut gt_0 = true;
    let mut lt_0 = true;
    let mut in_range = true;

    for item in diffs.into_iter() {
        gt_0 &= item > 0;
        lt_0 &= item < 0;
        in_range &= (1..=3).contains(&item.abs());
    }

    (gt_0 || lt_0) && in_range
}

impl Report {
    fn is_safe(&self) -> bool {
        diffs_are_safe(self.levels.windows(2).map(|w| w[1] - w[0]))
    }

    fn is_safe_with_problem_compensator(&self) -> bool {
        if self.is_safe() {
            return true;
        }

        for skip_idx in 0..self.levels.len() {
            let skip_iter = self
                .levels
                .iter()
                .copied()
                .enumerate()
                .filter_map(|(idx, d)| (idx != skip_idx).then_some(d))
                .tuple_windows()
                .map(|(a, b)| b - a);
            if diffs_are_safe(skip_iter) {
                return true;
            }
        }

        false
    }
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let safes = parse::<Report>(input)?.filter(Report::is_safe).count();
    println!("safe reports: {safes}");
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    let safes = parse::<Report>(input)?
        .filter(Report::is_safe_with_problem_compensator)
        .count();
    println!("safe reports with problem compensator: {safes}");
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("invalid input")]
    InvalidInput(#[from] std::num::ParseIntError),
    #[error("no solution found")]
    NoSolution,
}

#[cfg(test)]
mod tests {
    mod part2 {
        use crate::*;
        use rstest::rstest;

        #[rstest]
        #[case([7, 6, 4, 2, 1], true)]
        #[case([1, 2, 7, 8, 9], false)]
        #[case([9, 7, 6, 2, 1], false)]
        #[case([1, 3, 2, 4, 5], true)]
        #[case([8, 6, 4, 4, 1], true)]
        #[case([1, 3, 6, 7, 9], true)]
        fn part2_test<const N: usize>(#[case] inputs: [i32; N], #[case] expect: bool) {
            let report = Report {
                levels: inputs.into(),
            };
            assert_eq!(report.is_safe_with_problem_compensator(), expect);
        }
    }
}
