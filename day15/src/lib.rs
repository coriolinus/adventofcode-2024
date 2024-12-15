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

type Warehouse = aoclib::geometry::Map<Tile>;

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
        .filter_map(|(point, tile)| (*tile == Tile::Box).then(|| 100 * point.y + point.x))
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
    unimplemented!("input file: {:?}", input)
}
