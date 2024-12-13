use aoclib::input::parse_newline_sep;
use color_eyre::{
    eyre::{bail, Context},
    Result,
};
use std::{
    io::{BufRead, Cursor, Read},
    path::Path,
    str::FromStr,
};

#[derive(Debug, parse_display::FromStr)]
#[display("Button {ident}: X+{x}, Y+{y}")]
struct Button {
    ident: char,
    x: u32,
    y: u32,
}

#[derive(Debug, parse_display::FromStr)]
#[display("Prize: X={x}, Y={y}")]
struct Prize {
    x: u32,
    y: u32,
}

#[derive(Debug)]
struct ClawMachine {
    a: Button,
    b: Button,
    prize: Prize,
}

impl FromStr for ClawMachine {
    type Err = color_eyre::eyre::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut cursor = Cursor::new(s);
        let mut line = String::new();
        cursor.read_line(&mut line).context("reading for a")?;
        let a = line.trim().parse::<Button>().context("parsing a")?;
        if a.ident != 'A' {
            bail!("a ident unexpected: '{}'", a.ident);
        }
        line.clear();
        cursor.read_line(&mut line).context("reading for b")?;
        let b = line.trim().parse::<Button>().context("parsing b")?;
        if b.ident != 'B' {
            bail!("b ident unexpected: '{}'", b.ident);
        }
        line.clear();
        cursor.read_line(&mut line).context("reading for prize")?;
        let prize = line.trim().parse().context("parsing prize")?;
        debug_assert_eq!(
            {
                line.clear();
                cursor
                    .read_to_end(unsafe { line.as_mut_vec() })
                    .expect("can read to end of line");
                line.trim()
            },
            "",
            "no non-whitespace input must remain"
        );
        Ok(Self { a, b, prize })
    }
}

pub fn part1(input: &Path) -> Result<()> {
    let parsed_inputs = parse_newline_sep::<ClawMachine>(input)?.count();
    println!("parsed {parsed_inputs} claw machines");
    Ok(())
}

pub fn part2(input: &Path) -> Result<()> {
    unimplemented!("input file: {:?}", input)
}
