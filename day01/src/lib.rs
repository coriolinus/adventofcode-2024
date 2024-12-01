use aoclib::parse;
use std::path::Path;

struct Ns {
    left: u32,
    right: u32,
}

impl std::str::FromStr for Ns {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split_whitespace();
        let left = split
            .next()
            .ok_or(Error::InvalidInput)?
            .parse()
            .map_err(|_| Error::InvalidInput)?;
        let right = split
            .next()
            .ok_or(Error::InvalidInput)?
            .parse()
            .map_err(|_| Error::InvalidInput)?;
        Ok(Self { left, right })
    }
}

#[derive(Default)]
struct Lists {
    left: Vec<u32>,
    right: Vec<u32>,
}

impl FromIterator<Ns> for Lists {
    fn from_iter<T: IntoIterator<Item = Ns>>(iter: T) -> Self {
        let mut lists = Lists::default();
        for Ns { left, right } in iter {
            lists.left.push(left);
            lists.right.push(right);
        }
        lists
    }
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let mut lists = parse::<Ns>(input)?.collect::<Lists>();
    lists.left.sort_unstable();
    lists.right.sort_unstable();
    let dist_sum = lists
        .left
        .iter()
        .copied()
        .zip(lists.right.iter().copied())
        .map(|(left, right)| left.abs_diff(right))
        .sum::<u32>();
    println!("dist sum: {dist_sum}");
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    let mut lists = parse::<Ns>(input)?.collect::<Lists>();
    lists.left.sort_unstable();
    lists.right.sort_unstable();
    let mut rights = lists.right.iter().copied().peekable();
    let mut similarity = 0;

    for value in lists.left {
        let mut matches = 0;

        while let Some(right) = rights.peek().copied() {
            if right <= value {
                rights.next();
            }
            if right == value {
                matches += 1;
            }
            if right > value {
                break;
            }
        }

        similarity += value * matches;
    }

    println!("similarity score: {similarity}");
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("invalid input")]
    InvalidInput,
    #[error("no solution found")]
    NoSolution,
}
