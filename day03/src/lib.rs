use std::path::Path;

use regex::{Captures, Regex};

/// extract a number from a capture. assumes that the capture only captures valid numbers and the number is not optional.
fn expect_num(capture: &Captures, name: &str) -> u32 {
    capture
        .name(name)
        .expect("this group is non-optional")
        .as_str()
        .parse()
        .expect("this group always captures valid numbers")
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let data = std::fs::read_to_string(input)?;
    let mul_re = Regex::new(r"mul\((?<a>\d{1,3}),(?<b>\d{1,3})\)")?;
    let mul_sum = mul_re
        .captures_iter(&data)
        .map(|capture| {
            let a = expect_num(&capture, "a");
            let b = expect_num(&capture, "b");
            a * b
        })
        .sum::<u32>();

    println!("sum of multiplications: {mul_sum}");
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    let data = std::fs::read_to_string(input)?;
    let inst_re = Regex::new(
        r"(?<mul_instr>mul)\((?<a>\d{1,3}),(?<b>\d{1,3})\)|((?<enable_instr>do(n't)?)\(\))",
    )?;

    let mut enabled = true;
    let mut mul_sum = 0;

    for capture in inst_re.captures_iter(&data) {
        match (capture.name("mul_instr"), capture.name("enable_instr")) {
            (Some(_), None) => {
                if enabled {
                    let a = expect_num(&capture, "a");
                    let b = expect_num(&capture, "b");

                    mul_sum += a * b;
                }
            }
            (None, Some(instr)) => match instr.as_str() {
                "do" => enabled = true,
                "don't" => enabled = false,
                _ => unreachable!("instruction re doesn't match anything else"),
            },
            _ => unreachable!("instruction re should always match exactly mul or instr"),
        }
    }

    println!("sum of multiplication (part2): {mul_sum}");
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Re(#[from] regex::Error),
    #[error("no solution found")]
    NoSolution,
}
