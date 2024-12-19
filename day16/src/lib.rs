use aoclib::geometry::{tile::DisplayWidth, Direction, Point};
use color_eyre::{
    eyre::{Context as _, ContextCompat as _},
    Result,
};
use priority_queue::PriorityQueue;
use std::{
    cmp::Reverse,
    collections::HashSet,
    ops::{Index, IndexMut},
    path::Path,
};

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

type ActionQueue<H> = PriorityQueue<Reindeer<H>, Reverse<u32>>;

fn initialize_queue<H>(maze: &ReindeerMaze) -> Option<(Point, ActionQueue<H>)>
where
    H: Default + Eq + std::hash::Hash + RecordHistory + Clone,
{
    let start = maze
        .iter()
        .find_map(|(point, tile)| (*tile == Tile::Start).then_some(point))?;
    let mut queue = PriorityQueue::new();
    queue.push(Reindeer::<H>::new(start), Reverse(0));
    // There is exactly one case where turning 180 degrees might potentially be useful:
    // if the best path involves going west directly from the start. In all other cases, a path
    // behind must necessarily have come from there at a lower cost, so there's no point in
    // even investigating turning around.
    //
    // Let's handle the special case by hard-coding it.
    {
        let mut reverse_reindeer = Reindeer::<H>::new(start);
        reverse_reindeer = reverse_reindeer.turn_left();
        reverse_reindeer = reverse_reindeer.turn_left();
        if maze[reverse_reindeer.ahead()] != Tile::Wall {
            queue.push(reverse_reindeer.fwd(), Reverse(2001));
        }
    }

    Some((start, queue))
}

/// Search the reindeer maze from Start to End and return the score
// I'm doing this from memory and intuition, might not be pure djikstra
fn djikstraish(maze: &ReindeerMaze) -> Option<u32> {
    let (_start, mut queue) = initialize_queue::<()>(maze)?;

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

#[derive(Debug, Default, Clone, Copy)]
struct Scores {
    vertical: Option<u32>,
    horizontal: Option<u32>,
}

impl Index<Direction> for Scores {
    type Output = Option<u32>;

    fn index(&self, index: Direction) -> &Self::Output {
        match index {
            Direction::Right | Direction::Left => &self.horizontal,
            Direction::Up | Direction::Down => &self.vertical,
        }
    }
}

impl IndexMut<Direction> for Scores {
    fn index_mut(&mut self, index: Direction) -> &mut Self::Output {
        match index {
            Direction::Right | Direction::Left => &mut self.horizontal,
            Direction::Up | Direction::Down => &mut self.vertical,
        }
    }
}

impl Scores {
    fn min(&self) -> Option<u32> {
        self.vertical.min(self.horizontal)
    }
}

fn tiles_on_best_paths(maze: &ReindeerMaze) -> Option<usize> {
    let (start, mut queue) = initialize_queue::<ActionsPerformed>(maze)?;

    let mut lowest_score_by_point =
        aoclib::geometry::map::Map::<Scores>::new(maze.width(), maze.height());

    let mut best_histories = Vec::new();

    while let Some((reindeer, Reverse(score))) = queue.pop() {
        // eprintln!("{reindeer} @ {score}:");
        match lowest_score_by_point[reindeer.position][reindeer.orientation] {
            None => lowest_score_by_point[reindeer.position][reindeer.orientation] = Some(score),
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
                    .min()
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

    // eprintln!("found {} distinct paths", best_histories.len());

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

    debug_assert!(!visited_points
        .iter()
        .any(|point| maze[*point] == Tile::Wall));

    // eprintln!(
    //     "{}",
    //     maze.to_string_with_override(|point, _tile| visited_points
    //         .contains(&point)
    //         .then_some("O".into()))
    // );

    Some(visited_points.len())
}

pub fn part1(input: &Path) -> Result<()> {
    let maze = <ReindeerMaze as TryFrom<&Path>>::try_from(input).context("parsing input")?;
    let score = djikstraish(&maze).context("no solution found")?;

    println!("min score: {score}");
    Ok(())
}

// [Reddit] has been very helpful, with one comment in particular explaining a potential issue:
// imagine a T where two potential paths approach. The leg of the T is winning! It will get there
// a short handful of points before the cross of the T. But that's a problem: the leg of the T
// necessarily has to turn at the intersection; the code as it stands records the arrival score
// from the base of the T, so when it evaluates the path crossing the top of the T, that path
// gives up there. We end up recording the incorrectly-shorter path, instead of the correct longer path.
//
// The solution looks like this:
//
// - instead of recording one score at each visited point, record two: vertical and horizontal
// - when checking whether to terminate early at a point, you now need to abort not if there is any lower score, but if:
//   - there is a lower arrival score coming from the same direction as you, or
//   - there is a lower arrival score coming from the opposite direction as you (backtracking is never helpful)
//
// With these two rules in place, the race at the T plays out like this:
//
// - head from the bottom of the T arrives first and records its score and provenance
// - head from the bottom of the T adds successors turning left and right
// - head from the left of the T arrives and records its score and provenance
// - head from the left of the T adds successors proceeding straight, and turning right
// - successor from the left has the lowest score, so proceeds forward, recording its score and provenance
// - successor from the bottom going left discovers a lower score from the opposite direction and gives up
// - successor from the bottom going right discovers a lower score from the same direction and gives up
// - successor from the left going down discovers a lower score from the opposite direction and gives up
//
// [Reddit]: https://www.reddit.com/r/adventofcode/comments/1hfz425/2024_day_16_part_2rust/
pub fn part2(input: &Path) -> Result<()> {
    let maze = <ReindeerMaze as TryFrom<&Path>>::try_from(input).context("parsing input")?;
    let best_paths_tiles = tiles_on_best_paths(&maze).context("no solution found")?;

    println!("tiles on best paths: {best_paths_tiles}");
    Ok(())
}
