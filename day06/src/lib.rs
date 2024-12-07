use aoclib::geometry::{tile::DisplayWidth, Direction, MapConversionErr, Point};
use rayon::{iter::ParallelBridge, prelude::ParallelIterator};
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct VisitRecorder(u8);

impl VisitRecorder {
    fn direction_idx(index: Direction) -> usize {
        match index {
            Direction::Right => 0,
            Direction::Left => 1,
            Direction::Up => 2,
            Direction::Down => 3,
        }
    }

    fn set(&mut self, direction: Direction) {
        self.0 |= 1 << Self::direction_idx(direction);
    }

    fn is_set(&self, direction: Direction) -> bool {
        self.0 & (1 << Self::direction_idx(direction)) != 0
    }

    fn is_visited(&self) -> bool {
        self.0 != 0
    }
}

type Map = aoclib::geometry::map::Map<Tile>;
type Visited = aoclib::geometry::map::Map<VisitRecorder>;

#[derive(Debug, Clone, parse_display::Display)]
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
        visited[guard.position].set(guard.orientation);
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

    let visited_count = visited.iter().filter(|(_, tile)| tile.is_visited()).count();
    println!("visited locations: {visited_count}");

    Ok(())
}

fn produces_infinite_loop_with_additional_obstacle(
    mut guard: Guard,
    map: &Map,
    additional_obstacle: Point,
) -> bool {
    if map[additional_obstacle] != Tile::Blank {
        return false;
    }

    let mut visited = Visited::new(map.width(), map.height());

    while map.in_bounds(guard.position) {
        if visited[guard.position].is_set(guard.orientation) {
            return true;
        }
        visited[guard.position].set(guard.orientation);
        let forward = guard.position + guard.orientation;

        if forward == additional_obstacle
            || (map.in_bounds(forward) && map[forward] == Tile::Obstruction)
        {
            guard.orientation = guard.orientation.turn_right();
        } else {
            guard.position = forward;
        }
    }

    false
}

pub fn part2(input: &Path) -> Result<(), Error> {
    let map = <Map as TryFrom<&Path>>::try_from(input)?;
    let mut guard = None;
    for (position, tile) in map.iter() {
        if *tile == Tile::Initial {
            guard = Some(Guard::new(position));
            break;
        }
    }
    let guard = guard.ok_or(Error::GuardNotFound)?;

    let new_obstacles = map
        .iter()
        .map(|(position, _)| position)
        .par_bridge()
        .filter(|additional_obstacle| {
            produces_infinite_loop_with_additional_obstacle(
                guard.clone(),
                &map,
                *additional_obstacle,
            )
        })
        .count();
    println!("potential new obstacles: {new_obstacles}");

    Ok(())
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
