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

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
enum Operation {
    #[default]
    Add,
    Mul,
    Concat,
}

impl Operation {
    /// Continue to the next operation in sequence.
    ///
    /// Return `true` if this has overflowed back to the initial state.
    fn increment(&mut self) -> bool {
        *self = match self {
            Operation::Add => Operation::Mul,
            Operation::Mul => Operation::Concat,
            Operation::Concat => Operation::Add,
        };
        *self == Self::Add
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Operations(Box<[Operation]>);

impl Operations {
    fn new(dimension: usize) -> Self {
        Self(vec![Operation::default(); dimension].into())
    }

    /// Continue to the next operation in sequence.
    ///
    /// Return `true` if this has overflowed back to the initial state.
    fn increment(&mut self) -> bool {
        let mut idx = 0;
        let mut incr = true;
        while incr && idx < self.0.len() {
            incr = self.0[idx].increment();
            idx += 1;
        }
        debug_assert_eq!(incr, *self == Self::new(self.0.len()));
        incr
    }
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

    /// Evaluate the values of this equation according to the supplied operands list.
    fn evaluate_operands(&self, operands: &Operations) -> Option<i64> {
        debug_assert_eq!(
            operands.0.len(),
            self.values.0.len() - 1,
            "operands must interleave between equation terms"
        );
        let mut value = self.values.0[0];
        for (operand, v) in operands.0.iter().zip(self.values.0.iter().skip(1)) {
            match *operand {
                Operation::Add => value = value.checked_add(*v)?,
                Operation::Mul => value = value.checked_mul(*v)?,
                Operation::Concat => {
                    let n_digits = v.ilog10() + 1;
                    value = value.checked_mul(10_u32.pow(n_digits) as i64)?;
                    value = value.checked_add(*v)?;
                }
            }
        }
        Some(value)
    }

    fn can_evaluate_true(&self) -> bool {
        self.operand_maps()
            .any(|map| self.evaluate(map) == Some(self.test_value))
    }

    fn can_evaluate_true_with_concat(&self) -> bool {
        let mut operands = Operations::new(self.values.0.len() - 1);
        let mut overflowed = false;

        while !overflowed {
            if self.evaluate_operands(&operands) == Some(self.test_value) {
                return true;
            }
            overflowed = operands.increment();
        }
        false
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
    let calibration = parse::<Equation>(input)?
        .filter(Equation::can_evaluate_true_with_concat)
        .map(|equation| equation.test_value)
        .sum::<i64>();
    println!("sum of calibration with concat: {calibration}");
    Ok(())
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

#[cfg(test)]
mod tests {
    mod part2 {
        use crate::*;
        use rstest::rstest;

        #[rstest]
        #[case(190, &[10, 19], true)]
        #[case(3267, &[81, 40, 27], true)]
        #[case(83, &[17, 5], false)]
        #[case(156, &[15, 6], true)]
        #[case(7290, &[6, 8, 6, 15], true)]
        #[case(161011, &[16, 10, 13], false)]
        #[case(192, &[17, 8, 14], true)]
        #[case(21037, &[9, 7, 18, 13], false)]
        #[case(292, &[11, 6, 16, 20], true)]
        fn examples(#[case] test_value: i64, #[case] values: &[i64], #[case] expect: bool) {
            let equation = Equation {
                test_value,
                values: Values(values.into()),
            };
            assert_eq!(equation.can_evaluate_true_with_concat(), expect);
        }

        #[rstest]
        #[case(156, &[15, 6])]
        #[case(159, &[15, 9])]
        #[case(1510, &[15, 10])]
        #[case(1511, &[15, 11])]
        fn concat(#[case] test_value: i64, #[case] values: &[i64]) {
            let equation = Equation {
                test_value,
                values: Values(values.into()),
            };
            let operations = Operations([Operation::Concat].into());
            assert_eq!(equation.evaluate_operands(&operations), Some(test_value));
        }
    }
}
