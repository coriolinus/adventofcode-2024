use aoclib::geometry::{tile::DisplayWidth, Direction, Point};
use color_eyre::{
    eyre::{Context as _, ContextCompat as _},
    Result,
};
use priority_queue::PriorityQueue;
use std::{cmp::Reverse, collections::HashSet, path::Path};

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

#[derive(Debug, Clone, PartialEq, Eq, Hash, parse_display::Display)]
#[display("R({position.x}, {position.y}; {orientation:?})")]
struct Reindeer<History> {
    position: Point,
    orientation: Direction,
    visited: History,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Action {
    Fwd,
    TurnLeft,
    TurnRight,
}

trait RecordHistory {
    fn record(&mut self, position: Point, orientation: Direction, action: Action);
}

impl RecordHistory for () {
    fn record(&mut self, _position: Point, _orientation: Direction, _action: Action) {}
}

type VisitedPoints = im::HashSet<Point>;

impl RecordHistory for VisitedPoints {
    fn record(&mut self, position: Point, _orientation: Direction, _action: Action) {
        self.insert(position);
    }
}

type ActionsPerformed = im::Vector<Action>;

impl RecordHistory for ActionsPerformed {
    fn record(&mut self, _position: Point, _orientation: Direction, action: Action) {
        self.push_back(action);
    }
}

impl<History> Reindeer<History>
where
    History: Default,
{
    fn new(position: Point) -> Self {
        Self {
            position,
            orientation: Direction::Right,
            visited: Default::default(),
        }
    }
}

impl<History> Reindeer<History>
where
    History: RecordHistory + Clone,
{
    fn fwd(&self) -> Self {
        let orientation = self.orientation;
        let position = self.position + orientation;
        let mut visited = self.visited.clone();
        visited.record(position, orientation, Action::Fwd);

        Reindeer {
            position,
            orientation,
            visited,
        }
    }

    fn turn_left(&self) -> Self {
        let orientation = self.orientation.turn_left();
        let position = self.position;
        let mut visited = self.visited.clone();
        visited.record(position, orientation, Action::TurnLeft);

        Reindeer {
            position,
            orientation,
            visited,
        }
    }

    fn turn_right(&self) -> Self {
        let orientation = self.orientation.turn_right();
        let position = self.position;
        let mut visited = self.visited.clone();
        visited.record(position, orientation, Action::TurnRight);

        Reindeer {
            position,
            orientation,
            visited,
        }
    }
}

impl<History> Reindeer<History> {
    fn ahead(&self) -> Point {
        self.position + self.orientation
    }

    fn left_side(&self) -> Point {
        self.position + self.orientation.turn_left()
    }

    fn right_side(&self) -> Point {
        self.position + self.orientation.turn_right()
    }
}

/// Search the reindeer maze from Start to End and return the score
// I'm doing this from memory and intuition, might not be pure djikstra
fn djikstraish(maze: &ReindeerMaze) -> Option<u32> {
    let start = maze
        .iter()
        .find_map(|(point, tile)| (*tile == Tile::Start).then_some(point))?;
    let mut queue = PriorityQueue::new();
    queue.push(Reindeer::<()>::new(start), Reverse(0));

    while let Some((reindeer, Reverse(score))) = queue.pop() {
        match maze[reindeer.position] {
            Tile::End => return Some(score),
            Tile::Wall => continue,
            Tile::Empty | Tile::Start => (),
        }

        let is_clear = |point: Point| !matches!(maze[point], Tile::Wall | Tile::Start);

        // use `push_increase` here to avoid duplication of checks:
        // it can lower the score of an existing item, or insert it if not present,
        // but has no effect if an existing queue item has a lower score
        if is_clear(reindeer.ahead()) {
            queue.push_increase(reindeer.fwd(), Reverse(score + 1));
        }
        if is_clear(reindeer.left_side()) {
            queue.push_increase(reindeer.turn_left(), Reverse(score + 1000));
        }
        if is_clear(reindeer.right_side()) {
            queue.push_increase(reindeer.turn_right(), Reverse(score + 1000));
        }
    }
    None
}

fn tiles_on_best_paths(maze: &ReindeerMaze) -> Option<usize> {
    let start = maze
        .iter()
        .find_map(|(point, tile)| (*tile == Tile::Start).then_some(point))?;
    let mut queue = PriorityQueue::new();
    queue.push(Reindeer::<ActionsPerformed>::new(start), Reverse(0));

    let mut lowest_score_by_point =
        aoclib::geometry::map::Map::<Option<u32>>::new(maze.width(), maze.height());

    let mut best_histories = Vec::new();

    while let Some((reindeer, Reverse(score))) = queue.pop() {
        // eprintln!("{reindeer} @ {score}:");
        match lowest_score_by_point[reindeer.position] {
            None => lowest_score_by_point[reindeer.position] = Some(score),
            Some(low_score) if low_score + 1000 == score => {
                // we just turned, but we should still keep processing in order to move forward
            }
            Some(low_score) => match low_score.cmp(&score) {
                std::cmp::Ordering::Less => {
                    // nothing we can do from here will improve on the score/route we have already computed
                    // for this point, so we can discard this whole branch of the search space
                    // eprintln!("  existing path at this point lower ({low_score}) than our score ({score}); discarding");
                    continue;
                }
                std::cmp::Ordering::Equal => {
                    // we have matched the status quo; we can proceed
                    // eprintln!("  found a matching score; we now have another pointer on the trail");
                }
                std::cmp::Ordering::Greater => {
                    unreachable!("priority queue must process items lowest score first")
                }
            },
        }
        match maze[reindeer.position] {
            Tile::End
                if lowest_score_by_point[reindeer.position]
                    .map_or(true, |low_score| low_score == score) =>
            {
                // eprintln!("reached the goal with score {score}");
                best_histories.push(reindeer.visited);
                continue;
            }
            Tile::End => {
                // we got here, but with one turn too many
                continue;
            }
            Tile::Wall => unreachable!("we already checked that this does not run into a wall"),
            Tile::Empty | Tile::Start => (),
        }

        let is_clear = |point: Point| !matches!(maze[point], Tile::Wall | Tile::Start);

        // use `push_increase` here to avoid duplication of checks:
        // it can increase the priority of an existing item, or insert it if not present,
        // but has no effect if an existing queue item has a lower priority
        if is_clear(reindeer.ahead()) {
            // eprintln!("  enqueueing fwd");
            queue.push_increase(reindeer.fwd(), Reverse(score + 1));
        }
        if is_clear(reindeer.left_side()) {
            // eprintln!("  enqueueing turn_left");
            queue.push_increase(reindeer.turn_left(), Reverse(score + 1000));
        }
        if is_clear(reindeer.right_side()) {
            // eprintln!("  enqueueing turn_right");
            queue.push_increase(reindeer.turn_right(), Reverse(score + 1000));
        }
    }

    if best_histories.is_empty() {
        return None;
    }

    eprintln!("found {} distinct paths", best_histories.len());

    let mut visited_points = HashSet::new();
    for history in best_histories {
        visited_points.insert(start);
        let mut reindeer = Reindeer::<()>::new(start);
        for action in history {
            reindeer = match action {
                Action::Fwd => reindeer.fwd(),
                Action::TurnLeft => reindeer.turn_left(),
                Action::TurnRight => reindeer.turn_right(),
            };
            visited_points.insert(reindeer.position);
        }
        assert_eq!(
            maze[reindeer.position],
            Tile::End,
            "reindeer must have navigated to the end"
        );
    }

    eprintln!(
        "{}",
        maze.to_string_with_override(|point, _tile| visited_points
            .contains(&point)
            .then_some("O".into()))
    );

    Some(visited_points.len())
}

pub fn part1(input: &Path) -> Result<()> {
    let maze = <ReindeerMaze as TryFrom<&Path>>::try_from(input).context("parsing input")?;
    let score = djikstraish(&maze).context("no solution found")?;

    println!("min score: {score}");
    Ok(())
}

// too high: 432
pub fn part2(input: &Path) -> Result<()> {
    let maze = <ReindeerMaze as TryFrom<&Path>>::try_from(input).context("parsing input")?;
    let best_paths_tiles = tiles_on_best_paths(&maze).context("no solution found")?;

    println!("tiles on best paths: {best_paths_tiles}");
    Ok(())
}
