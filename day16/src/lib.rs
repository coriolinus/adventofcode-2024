use aoclib::geometry::{tile::DisplayWidth, Direction, Point};
use color_eyre::{
    eyre::{Context as _, ContextCompat as _},
    Result,
};
use priority_queue::PriorityQueue;
use std::{cmp::Reverse, path::Path};

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, parse_display::Display, parse_display::FromStr,
)]
enum Tile {
    #[default]
    #[display(".")]
    Empty,
    #[display("#")]
    Wall,
    #[display("S")]
    Start,
    #[display("E")]
    End,
}

impl DisplayWidth for Tile {
    const DISPLAY_WIDTH: usize = 1;
}

type ReindeerMaze = aoclib::geometry::map::Map<Tile>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Reindeer {
    position: Point,
    orientation: Direction,
}

impl Reindeer {
    fn fwd(self) -> Self {
        Reindeer {
            position: self.position + self.orientation,
            orientation: self.orientation,
        }
    }

    fn turn_left(self) -> Self {
        Reindeer {
            position: self.position,
            orientation: self.orientation.turn_left(),
        }
    }

    fn turn_right(self) -> Self {
        Reindeer {
            position: self.position,
            orientation: self.orientation.turn_right(),
        }
    }
}

/// Search the reindeer maze from Start to End and return the score
// I'm doing this from memory and intuition, might not be pure djikstra
fn djikstraish(maze: &ReindeerMaze) -> Option<u32> {
    let start = maze
        .iter()
        .find_map(|(point, tile)| (*tile == Tile::Start).then_some(point))?;
    let mut queue = PriorityQueue::new();
    queue.push(
        Reindeer {
            position: start,
            orientation: Direction::Right,
        },
        Reverse(0),
    );

    while let Some((reindeer, Reverse(score))) = queue.pop() {
        match maze[reindeer.position] {
            Tile::End => return Some(score),
            Tile::Wall => continue,
            Tile::Empty | Tile::Start => (),
        }

        // use `push_increase` here to avoid duplication of checks:
        // it can increase the priority of an existing item, or insert it if not present,
        // but has no effect if an existing queue item has a lower priority
        queue.push_increase(reindeer.fwd(), Reverse(score + 1));
        queue.push_increase(reindeer.turn_left(), Reverse(score + 1000));
        queue.push_increase(reindeer.turn_right(), Reverse(score + 1000));
    }
    None
}

pub fn part1(input: &Path) -> Result<()> {
    let maze = <ReindeerMaze as TryFrom<&Path>>::try_from(input).context("parsing input")?;
    let score = djikstraish(&maze).context("no solution found")?;

    println!("min score: {score}");
    Ok(())
}

pub fn part2(input: &Path) -> Result<()> {
    unimplemented!("input file: {:?}", input)
}
