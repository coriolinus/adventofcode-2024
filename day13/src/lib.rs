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
    x: i64,
    y: i64,
}

#[derive(Debug, parse_display::FromStr)]
#[display("Prize: X={x}, Y={y}")]
struct Prize {
    x: i64,
    y: i64,
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

impl ClawMachine {
    fn solve_tokens(self) -> Option<i64> {
        self.solve_tokens_with_offset(0, 0)
    }

    fn solve_tokens_with_offset(self, offset_x: i64, offset_y: i64) -> Option<i64> {
        // this is straightforwardly a math problem
        // happily, this guy is here to bring the math for us:
        // https://www.reddit.com/r/adventofcode/comments/1hd7irq/2024_day_13_an_explanation_of_the_mathematics/
        let ClawMachine { a, b, mut prize } = self;
        prize.x += offset_x;
        prize.y += offset_y;

        let determinant = a.x * b.y - a.y * b.x;
        let a_mul = (prize.x * b.y - prize.y * b.x) / determinant;
        let b_mul = (prize.y * a.x - prize.x * a.y) / determinant;
        (a_mul * a.x + b_mul * b.x == prize.x && a_mul * a.y + b_mul * b.y == prize.y)
            .then_some(3 * a_mul + b_mul)
    }
}

pub fn part1(input: &Path) -> Result<()> {
    let spent_tokens = parse_newline_sep::<ClawMachine>(input)?
        .filter_map(ClawMachine::solve_tokens)
        .sum::<i64>();
    println!("spent {spent_tokens} tokens winning claw machines");
    Ok(())
}

pub fn part2(input: &Path) -> Result<()> {
    const OFFSET: i64 = 10_000_000_000_000; // 10 trillion
    let spent_tokens = parse_newline_sep::<ClawMachine>(input)?
        .filter_map(|claw_machine| claw_machine.solve_tokens_with_offset(OFFSET, OFFSET))
        .sum::<i64>();
    println!("spent {spent_tokens} tokens winning claw machines");
    Ok(())
}
