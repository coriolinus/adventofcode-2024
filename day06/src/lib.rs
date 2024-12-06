use aoclib::geometry::{
    tile::{Bool, DisplayWidth},
    Direction, MapConversionErr, Point,
};
use std::path::Path;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, parse_display::FromStr, parse_display::Display,
)]
enum Tile {
    #[display(".")]
    Blank,
    #[display("#")]
    Obstruction,
    #[display("^")]
    Initial,
}

impl DisplayWidth for Tile {
    const DISPLAY_WIDTH: usize = 1;
}

type Map = aoclib::geometry::map::Map<Tile>;
type Visited = aoclib::geometry::map::Map<Bool>;

#[derive(parse_display::Display)]
#[display("G({position.x},{position.y};{orientation:?})")]
struct Guard {
    position: Point,
    orientation: Direction,
}

impl Guard {
    fn new(position: Point) -> Self {
        Self {
            position,
            orientation: Direction::Up,
        }
    }
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let map = <Map as TryFrom<&Path>>::try_from(input)?;
    let mut guard = None;
    for (position, tile) in map.iter() {
        if *tile == Tile::Initial {
            guard = Some(Guard::new(position));
            break;
        }
    }
    let mut guard = guard.ok_or(Error::GuardNotFound)?;
    let mut visited = Visited::new(map.width(), map.height());

    while map.in_bounds(guard.position) {
        visited[guard.position] = true.into();
        let forward = guard.position + guard.orientation;
        // eprintln!(
        //     "{guard} facing {}@({},{})",
        //     map[forward], forward.x, forward.y
        // );

        if map.in_bounds(forward) && map[forward] == Tile::Obstruction {
            guard.orientation = guard.orientation.turn_right();
        } else {
            guard.position = forward;
        }
    }

    let visited_count = visited
        .iter()
        .filter(|(_, tile)| bool::from(**tile))
        .count();
    println!("visited locations: {visited_count}");

    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    unimplemented!("input file: {:?}", input)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("map converstion")]
    MapConversion(#[from] MapConversionErr),
    #[error("initial guard position not found")]
    GuardNotFound,
    #[error("no solution found")]
    NoSolution,
}
