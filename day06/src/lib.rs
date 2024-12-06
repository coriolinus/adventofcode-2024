use aoclib::geometry::{tile::DisplayWidth, Direction, MapConversionErr, Point};
use rayon::{iter::ParallelBridge, prelude::ParallelIterator};
use std::{
    ops::{Index, IndexMut},
    path::Path,
};

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
struct VisitRecorder([bool; 4]);

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
        self[direction] = true;
    }

    fn is_set(&self, direction: Direction) -> bool {
        self[direction]
    }

    fn is_visited(&self) -> bool {
        Direction::iter().any(|direction| self.is_set(direction))
    }
}

impl Index<Direction> for VisitRecorder {
    type Output = bool;

    fn index(&self, index: Direction) -> &Self::Output {
        &self.0[Self::direction_idx(index)]
    }
}

impl IndexMut<Direction> for VisitRecorder {
    fn index_mut(&mut self, index: Direction) -> &mut Self::Output {
        &mut self.0[Self::direction_idx(index)]
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

// too low: 413
// too low: 416
pub fn part2(input: &Path) -> Result<(), Error> {
    // plan: keep track of previously visited, not just boolean, but a stack of directions
    // at each position, project what would happen if we turned right, right now
    // if we encounter a place where we'd be proceeding along the current dirction before hitting
    // an obstacle or leaving the map, then the current forward could become the location of an obstacle
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
