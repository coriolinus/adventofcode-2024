use aoclib::geometry::{tile::DisplayWidth, Direction, Point};
use color_eyre::{
    eyre::{bail, eyre, Context as _, ContextCompat as _},
    Result,
};
use std::path::Path;

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, parse_display::Display, parse_display::FromStr,
)]
enum Tile {
    #[default]
    #[display(".")]
    Empty,
    #[display("#")]
    Wall,
    #[display("O")]
    Box,
    #[display("@")]
    Robot,
}

impl DisplayWidth for Tile {
    const DISPLAY_WIDTH: usize = 1;
}

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, parse_display::Display, parse_display::FromStr,
)]
enum TileWide {
    #[default]
    #[display(".")]
    Empty,
    #[display("#")]
    Wall,
    #[display("[")]
    BoxLeft,
    #[display("]")]
    BoxRight,
    #[display("@")]
    Robot,
}

impl DisplayWidth for TileWide {
    const DISPLAY_WIDTH: usize = 2;
}

type Warehouse = aoclib::geometry::Map<Tile>;
type WarehouseWide = aoclib::geometry::Map<TileWide>;

fn widen(map: Warehouse) -> WarehouseWide {
    let mut out = WarehouseWide::new(map.width() * 2, map.height());
    for (point, &tile) in map.iter() {
        let left = Point::new(point.x * 2, point.y);
        let right = Point::new(point.x * 2 + 1, point.y);
        match tile {
            Tile::Empty => {
                // default is empty
            }
            Tile::Robot => {
                out[left] = TileWide::Robot;
                // right is empty
            }
            Tile::Wall => {
                out[left] = TileWide::Wall;
                out[right] = TileWide::Wall;
            }
            Tile::Box => {
                out[left] = TileWide::BoxLeft;
                out[right] = TileWide::BoxRight;
            }
        }
    }
    out
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, derive_more::Deref, derive_more::From, derive_more::Into,
)]
struct Movement(Direction);

impl TryFrom<u8> for Movement {
    type Error = color_eyre::eyre::Report;

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            b'<' => Ok(Direction::Left.into()),
            b'^' => Ok(Direction::Up.into()),
            b'>' => Ok(Direction::Right.into()),
            b'v' => Ok(Direction::Down.into()),
            _ => Err(eyre!("unrecognized byte for direction")),
        }
    }
}

fn parse(input: &Path) -> Result<(Warehouse, Vec<Movement>)> {
    let data = std::fs::read_to_string(input).context("reading input file")?;
    let (map_data, movement_data) = data
        .split_once("\n\n")
        .context("no double newline to separate map from movements")?;

    let warehouse =
        <Warehouse as TryFrom<&str>>::try_from(map_data).context("parsing warehouse map")?;

    let mut movements = Vec::new();
    for byte in movement_data.as_bytes().iter().copied() {
        if byte.is_ascii_whitespace() {
            continue;
        }

        movements.push(byte.try_into()?);
    }

    Ok((warehouse, movements))
}

fn sum_of_box_gps(map: &Warehouse) -> i32 {
    let map = map.flip_vertical();
    map.iter()
        .filter(|&(_point, tile)| *tile == Tile::Box)
        .map(|(point, _tile)| 100 * point.y + point.x)
        .sum()
}

fn sum_of_box_gps_wide(map: &WarehouseWide) -> i32 {
    let map = map.flip_vertical();
    map.iter()
        .filter(|&(_point, tile)| *tile == TileWide::BoxLeft)
        .map(|(point, _tile)| 100 * point.y + point.x)
        .sum()
}

struct Robot {
    position: Point,
}

impl Robot {
    fn extract_from(map: &mut Warehouse) -> Result<Self> {
        let mut position = None;
        for (point, tile) in map.iter() {
            if *tile == Tile::Robot {
                if position.is_some() {
                    bail!("more than one robot found in warehouse");
                }
                position = Some(point);
            }
        }
        let Some(position) = position else {
            bail!("no robots found in warehouse");
        };
        map[position] = Tile::Empty;
        Ok(Self { position })
    }

    fn extract_from_wide(map: &mut WarehouseWide) -> Result<Self> {
        let mut position = None;
        for (point, tile) in map.iter() {
            if *tile == TileWide::Robot {
                if position.is_some() {
                    bail!("more than one robot found in warehouse");
                }
                position = Some(point);
            }
        }
        let Some(position) = position else {
            bail!("no robots found in warehouse");
        };
        map[position] = TileWide::Empty;
        Ok(Self { position })
    }

    fn push(&mut self, map: &mut Warehouse, movement: Movement) {
        let mut encountered_a_box = false;
        let mut empty_space = None;
        let (dx, dy) = movement.deltas();
        for point in map.project(self.position, dx, dy).skip(1) {
            match map[point] {
                Tile::Robot => unreachable!("no extra robots in map"),
                Tile::Box => {
                    // no problem, we can keep going, our robot is strong and can push many boxes
                    encountered_a_box = true;
                }
                Tile::Wall => {
                    // oh, no movement is possible because we're jammed up against a wall
                    // (possibly through many boxes)
                    return;
                }
                Tile::Empty => {
                    // we can push, so therefore we must
                    empty_space = Some(point);
                    break;
                }
            }
        }

        let empty_space = empty_space.expect("we can only get to this point in the code if we found an empty space or pushed off the map");

        self.position += *movement;
        if encountered_a_box {
            debug_assert_eq!(map[self.position], Tile::Box);
            debug_assert_eq!(map[empty_space], Tile::Empty);
            map[self.position] = Tile::Empty;
            map[empty_space] = Tile::Box;
        } else {
            debug_assert_eq!(map[self.position], Tile::Empty);
        }
    }

    fn push_wide_horizontal(&mut self, map: &mut WarehouseWide, movement: Movement) {
        debug_assert!(matches!(*movement, Direction::Left | Direction::Right));
        // we can more or less use the same algorithm as in part 1 here
        let mut empty_space = None;
        let (dx, dy) = movement.deltas();
        for point in map.project(self.position, dx, dy).skip(1) {
            match map[point] {
                TileWide::Robot => unreachable!("no extra robots in map"),
                TileWide::BoxLeft | TileWide::BoxRight => {
                    // no problem, we can keep going, our robot is strong and can push many boxes
                }
                TileWide::Wall => {
                    // oh, no movement is possible because we're jammed up against a wall
                    // (possibly through many boxes)
                    return;
                }
                TileWide::Empty => {
                    // we can push, so therefore we must
                    empty_space = Some(point);
                    break;
                }
            }
        }

        let empty_space = empty_space.expect("we can only get to this point in the code if we found an empty space or pushed off the map");

        let mut reverse_projection = map.project(empty_space, -dx, dy).peekable();
        while let Some(copy_into) = reverse_projection.next() {
            let copy_from = *reverse_projection
                .peek()
                .expect("we break before getting to the wall");
            map[copy_into] = map[copy_from];
            if copy_from == self.position {
                debug_assert_eq!(map[copy_from], TileWide::Empty);
                break;
            }
        }
        self.position += *movement;
    }

    fn can_push_wide_vertical(position: Point, map: &WarehouseWide, movement: Movement) -> bool {
        // we need to consider a whole range of points to determine whether we can move or not
        // let's be recursive
        match map[position] {
            TileWide::Robot => unreachable!("no extra robots on map"),
            TileWide::Empty => true,
            TileWide::Wall => false,
            TileWide::BoxLeft => {
                let next = position + *movement;
                let right = next + Direction::Right;
                Self::can_push_wide_vertical(next, map, movement)
                    && Self::can_push_wide_vertical(right, map, movement)
            }
            TileWide::BoxRight => {
                let next = position + *movement;
                let left = next + Direction::Left;
                Self::can_push_wide_vertical(next, map, movement)
                    && Self::can_push_wide_vertical(left, map, movement)
            }
        }
    }

    fn push_boxes_wide_vertical(position: Point, map: &mut WarehouseWide, movement: Movement) {
        match map[position] {
            TileWide::Robot => unreachable!("no extra robots on map"),
            TileWide::Wall => {
                panic!("we should have ensured we could do this before trying to do it")
            }
            TileWide::BoxLeft => {
                let next = position + *movement;
                let right = next + Direction::Right;
                Self::push_boxes_wide_vertical(next, map, movement);
                Self::push_boxes_wide_vertical(right, map, movement);
            }
            TileWide::BoxRight => {
                let next = position + *movement;
                let left = next + Direction::Left;
                Self::push_boxes_wide_vertical(next, map, movement);
                Self::push_boxes_wide_vertical(left, map, movement);
            }
            TileWide::Empty => {
                // no special action needed here
            }
        }
        let from = position + movement.reverse();
        map[position] = map[from];
        // this ensures we don't leave partial boxes behind
        map[from] = TileWide::Empty;
    }

    fn push_wide_vertical(&mut self, map: &mut WarehouseWide, movement: Movement) {
        debug_assert!(matches!(*movement, Direction::Up | Direction::Down));
        if Self::can_push_wide_vertical(self.position + *movement, map, movement) {
            Self::push_boxes_wide_vertical(self.position + *movement, map, movement);
            self.position += *movement;
        }
    }

    fn push_wide(&mut self, map: &mut WarehouseWide, movement: Movement) {
        if matches!(*movement, Direction::Left | Direction::Right) {
            self.push_wide_horizontal(map, movement);
        } else {
            self.push_wide_vertical(map, movement);
        }
    }
}

pub fn part1(input: &Path) -> Result<()> {
    let (mut warehouse, movements) = parse(input).context("parsing input")?;
    let mut robot = Robot::extract_from(&mut warehouse)?;
    for movement in movements {
        robot.push(&mut warehouse, movement);
    }
    let sum_of_gps = sum_of_box_gps(&warehouse);
    println!("sum of box gps: {sum_of_gps}");
    Ok(())
}

pub fn part2(input: &Path) -> Result<()> {
    let (warehouse, movements) = parse(input).context("parsing input")?;
    let mut warehouse = widen(warehouse);
    // eprintln!("{warehouse}");
    let mut robot = Robot::extract_from_wide(&mut warehouse)?;
    for movement in movements {
        // eprintln!("{movement:?}");
        robot.push_wide(&mut warehouse, movement);
        // eprintln!(
        //     "robot @ ({}, {}):\n{warehouse}",
        //     robot.position.x, robot.position.y
        // );
    }
    let sum_of_gps = sum_of_box_gps_wide(&warehouse);
    println!("sum of box gps (wide): {sum_of_gps}");
    Ok(())
}
