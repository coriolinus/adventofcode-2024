use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use aoclib::geometry::{tile::DisplayWidth, Point};

#[derive(Debug, Copy, Clone, derive_more::FromStr, derive_more::Into)]
pub struct Char(char);

impl DisplayWidth for Char {
    const DISPLAY_WIDTH: usize = 1;
}

type Map = aoclib::geometry::map::Map<Char>;

pub fn part1(input: &Path) -> Result<(), Error> {
    let map = <Map as TryFrom<&Path>>::try_from(input)?;
    let mut antennae_by_frequency: HashMap<char, Vec<Point>> = Default::default();
    for (location, ch) in map.iter() {
        let ch = char::from(*ch);
        if ch.is_alphanumeric() {
            antennae_by_frequency.entry(ch).or_default().push(location);
        }
    }

    let mut antinodes = HashSet::new();
    for (_ch, points) in antennae_by_frequency.iter() {
        for a in points.iter().copied() {
            for b in points.iter().copied() {
                if a == b {
                    continue;
                }

                let diff = b - a;

                for antinode in [a - diff, b + diff] {
                    if map.in_bounds(antinode) {
                        antinodes.insert(antinode);
                    }
                }
            }
        }
    }

    println!("antinodes in map: {}", antinodes.len());
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    let map = <Map as TryFrom<&Path>>::try_from(input)?;
    let mut antennae_by_frequency: HashMap<char, Vec<Point>> = Default::default();
    for (location, ch) in map.iter() {
        let ch = char::from(*ch);
        if ch.is_alphanumeric() {
            antennae_by_frequency.entry(ch).or_default().push(location);
        }
    }

    let mut antinodes = HashSet::new();
    for (_ch, points) in antennae_by_frequency.iter() {
        for a in points.iter().copied() {
            for b in points.iter().copied() {
                if a == b {
                    continue;
                }

                let diff = b - a;

                let mut antinode = a;
                while map.in_bounds(antinode) {
                    antinodes.insert(antinode);
                    antinode -= diff;
                }
                antinode = b;
                while map.in_bounds(antinode) {
                    antinodes.insert(antinode);
                    antinode += diff;
                }
            }
        }
    }

    println!("antinodes in map: {}", antinodes.len());
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("no solution found")]
    NoSolution,
}
