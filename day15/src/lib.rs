use aoclib::geometry::{tile::DisplayWidth, Direction};
use color_eyre::{
    eyre::{eyre, Context as _},
    Result,
};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, parse_display::Display, parse_display::FromStr)]
enum Tile {
    #[display("#")]
    Wall,
    #[display(".")]
    Empty,
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
        .ok_or(eyre!("no double newline to separate map from movements"))?;

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

pub fn part1(input: &Path) -> Result<()> {
    let (warehouse, movements) = parse(input).context("parsing input")?;
    println!("parsed with {} movements", movements.len());
    Ok(())
}

pub fn part2(input: &Path) -> Result<()> {
    unimplemented!("input file: {:?}", input)
}
