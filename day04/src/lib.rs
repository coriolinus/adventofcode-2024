use aoclib::geometry::{tile::DisplayWidth, MapConversionErr, Point};
use std::path::Path;

type WordSearch = aoclib::geometry::map::Map<Char>;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    derive_more::Constructor,
    derive_more::From,
    derive_more::Into,
    derive_more::Deref,
    derive_more::DerefMut,
    derive_more::Display,
    derive_more::FromStr,
)]
struct Char(char);

impl DisplayWidth for Char {
    const DISPLAY_WIDTH: usize = 1;
}

const XMAS: &[u8] = b"XMAS";

fn directions() -> impl Iterator<Item = (i32, i32)> {
    (-1..=1)
        .flat_map(|dx| (-1..=1).map(move |dy| (dx, dy)))
        .filter(|(dx, dy)| !(*dx == 0 && *dy == 0))
}

fn is_xmas(grid: &WordSearch, origin: Point, dx: i32, dy: i32) -> bool {
    let mut idx = 0;
    for point in grid.project(origin, dx, dy).take(XMAS.len()) {
        if (*grid[point] as u8) != XMAS[idx] {
            return false;
        }
        idx += 1;
    }

    idx == XMAS.len()
}

fn is_x_mas(grid: &WordSearch, origin: Point) -> bool {
    let is_mas = |dx, dy| {
        let pm = origin + (dx, dy);
        let ps = origin + (-dx, -dy);
        *grid[origin] == 'A'
            && grid.in_bounds(pm)
            && *grid[pm] == 'M'
            && grid.in_bounds(ps)
            && *grid[ps] == 'S'
    };

    let diags = [(-1, -1), (-1, 1), (1, -1), (1, 1)];

    let mas_count = diags
        .into_iter()
        .filter(|(dx, dy)| is_mas(*dx, *dy))
        .count();

    mas_count > 1
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let search_grid = <WordSearch as TryFrom<&Path>>::try_from(input)?;

    let mut count = 0;

    for (origin, _tile) in search_grid.iter() {
        for (dx, dy) in directions() {
            if is_xmas(&search_grid, origin, dx, dy) {
                count += 1;
            }
        }
    }

    println!("xmas count: {count}");
    Ok(())
}

// not right: 15
pub fn part2(input: &Path) -> Result<(), Error> {
    let search_grid = <WordSearch as TryFrom<&Path>>::try_from(input)?;

    let count = search_grid
        .iter()
        .filter(|(origin, _tile)| is_x_mas(&search_grid, *origin))
        .count();

    println!("x-mas count: {count}");
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("invalid input")]
    MapConversion(#[from] MapConversionErr),
    #[error("no solution found")]
    NoSolution,
}
