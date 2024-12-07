use aoclib::parse;
use std::{path::Path, str::FromStr};

#[derive(Debug)]
struct Values(Vec<i64>);

impl FromStr for Values {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split_whitespace()
            .map(str::parse)
            .collect::<Result<Vec<_>, _>>()
            .map(Self)
            .map_err(Into::into)
    }
}

#[derive(Debug, parse_display::FromStr)]
#[display("{test_value}: {values}")]
struct Equation {
    test_value: i64,
    values: Values,
}

impl Equation {
    /// Produce all valid operand maps for this equation's values
    fn operand_maps(&self) -> impl Iterator<Item = u32> {
        let max = 2_u32.pow(self.values.0.len() as u32 - 1);
        0..max
    }

    /// Evaluate the values of this equation according to the supplied operand map.
    ///
    /// In the map, `0` at a given index indicates addition, and `1` indicates multiplication.
    fn evaluate(&self, operand_map: u32) -> Option<i64> {
        assert!(
            self.values.0.len() <= 33,
            "this function can't handle long terms"
        );
        let mut value = self.values.0[0];
        for (idx, v) in self.values.0[1..].iter().copied().enumerate() {
            if operand_map & (1 << idx) == 0 {
                value = value.checked_add(v)?;
            } else {
                value = value.checked_mul(v)?;
            }
        }
        Some(value)
    }

    fn can_evaluate_true(&self) -> bool {
        self.operand_maps()
            .any(|map| self.evaluate(map) == Some(self.test_value))
    }
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let calibration = parse::<Equation>(input)?
        .filter(Equation::can_evaluate_true)
        .map(|equation| equation.test_value)
        .sum::<i64>();
    println!("sum of calibration: {calibration}");
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    unimplemented!("input file: {:?}", input)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("not a number")]
    NaN(#[from] std::num::ParseIntError),
    #[error("no solution found")]
    NoSolution,
}
