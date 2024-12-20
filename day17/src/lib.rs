use color_eyre::{
    eyre::{bail, Context as _, ContextCompat as _},
    Result,
};
use itertools::Itertools;
use regex::Regex;
use std::{collections::VecDeque, path::Path};

type Register = u64;
type ThreeBit = u8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::FromRepr)]
#[repr(u8)]
enum Instruction {
    /// A Divide
    ///
    /// Numerator is value in A
    /// Denominator is 2**combo
    ///
    /// Result is truncated to 3 bits and written to A
    Adv = 0,
    /// B XOR Literal
    ///
    /// Bitwise XOR of B and the literal operand, stored in B
    Bxl = 1,
    /// B Store
    ///
    /// Combo operand mod 8, stored in B register
    Bst = 2,
    /// Jump Not Zero
    ///
    /// Nothing if A register is `0`.
    /// If A is nonzero, set instruction pointer to value of literal operand.
    Jnz = 3,
    /// B XOR C
    ///
    /// Bitwise Xor of B and C
    /// Result in B
    ///
    /// Ignores its operand
    Bxc = 4,
    /// Output
    ///
    /// Send combo operand mod 8 on output channel
    Out = 5,
    /// B Divide
    ///
    /// Numerator value in A
    /// Denominator is 2**combo
    ///
    /// Result truncated to 3 bits and stored in B
    Bdv = 6,
    /// C Divide
    ///
    /// Numerator value in A
    /// Denominator value is 2**combo
    ///
    /// Result truncated to 3 bits and stored in C
    Cdv = 7,
}

struct Computer {
    registers: [Register; 3],
    instruction_pointer: usize,
    program: Vec<ThreeBit>,
    output: Vec<ThreeBit>,
}

impl Computer {
    fn new(program: Vec<ThreeBit>) -> Self {
        Self {
            program,
            registers: Default::default(),
            instruction_pointer: Default::default(),
            output: Default::default(),
        }
    }

    fn from_input(input: &str) -> Result<Self> {
        let re = Regex::new(r"\d+").context("constructing digit regex")?;
        let mut numbers = re.find_iter(input);

        let a = numbers
            .next()
            .context("no value for register a")?
            .as_str()
            .parse()?;
        let b = numbers
            .next()
            .context("no value for register b")?
            .as_str()
            .parse()?;
        let c = numbers
            .next()
            .context("no value for register c")?
            .as_str()
            .parse()?;
        let program = numbers
            .map(|m| m.as_str().parse())
            .collect::<Result<_, _>>()
            .context("parsing program")?;

        let mut computer = Self::new(program);
        computer.registers = [a, b, c];
        Ok(computer)
    }

    fn operand(&self) -> Result<ThreeBit> {
        let Some(&operand) = self.program.get(self.instruction_pointer + 1) else {
            bail!("program terminated with instruction but no operand");
        };
        if operand & !0b111 != 0 {
            bail!("operand {operand:#08b} ({operand}) out of range for ThreeBit");
        }
        Ok(operand)
    }

    fn literal_operand(&self) -> Result<ThreeBit> {
        self.operand()
    }

    fn combo_operand(&self) -> Result<Register> {
        let operand = self.operand()?;
        let value = match operand {
            0..=3 => operand as _,
            4..=6 => self.registers[(operand - 4) as usize],
            7 => bail!("register 7 is reserved and not present in valid programs"),
            _ => unreachable!("{operand} out of range for ThreeBit"),
        };
        Ok(value)
    }

    fn next_ip(&self, instruction: Instruction) -> usize {
        if instruction == Instruction::Jnz && self.registers[0] != 0 {
            self.program[self.instruction_pointer + 1] as _
        } else {
            self.instruction_pointer + 2
        }
    }

    /// Implement `Adv`, `Bdv`, `Cdv`
    ///
    /// Note that dividing by `2 ** n` is equivalent to `>> n`
    fn right_shift(&mut self, combo_operand: Register, store_idx: usize) {
        self.registers[store_idx] = self.registers[0] >> combo_operand;
    }

    /// Process one instruction, updating internal state
    ///
    /// Returns `Ok(false)` when the program terminates
    fn tick(&mut self) -> Result<bool> {
        let Some(&instruction) = self.program.get(self.instruction_pointer) else {
            // program over; halt normally
            return Ok(false);
        };
        let instruction = Instruction::from_repr(instruction).context("invalid instruction")?;
        match instruction {
            Instruction::Adv => self.right_shift(self.combo_operand()?, 0),
            Instruction::Bdv => self.right_shift(self.combo_operand()?, 1),
            Instruction::Cdv => self.right_shift(self.combo_operand()?, 2),
            Instruction::Bxl => self.registers[1] ^= u64::from(self.literal_operand()?),
            Instruction::Bxc => self.registers[1] ^= self.registers[2],
            Instruction::Bst => self.registers[1] = self.combo_operand()? & 0b111,
            Instruction::Out => self.output.push((self.combo_operand()? & 0b111) as _),
            Instruction::Jnz => (),
        }
        self.instruction_pointer = self.next_ip(instruction);
        Ok(true)
    }

    fn prepare_output(&self) -> String {
        self.output.iter().map(ToString::to_string).join(",")
    }
}

pub fn part1(input: &Path) -> Result<()> {
    let input = std::fs::read_to_string(input).context("reading input to string")?;
    let mut computer = Computer::from_input(&input).context("parsing input as computer")?;
    // this processes all instructions
    while computer.tick().context("processing an instruction")? {}
    let output = computer.prepare_output();
    println!("output pt 1: {output}");
    Ok(())
}

/// Apply a cycle of the program to A, returning B
///
/// based on manually decompiling my program
///
/// ```notrust
/// while a != 0:
///     b = a & 0b111
///     b ^= 0b010
///     c = a >> b
///     b ^= c
///     b ^= 0b111
///     output b
///     a >>= 3
/// ```
fn apply_cycle(a: Register) -> Register {
    let mut b = a & 0b111;
    b ^= 0b010;
    let c = a >> b;
    b ^= c;
    b ^= 0b111;
    b
}

struct SearchNode {
    /// index from the right of current item in program
    ///
    /// gives us the expected output and a termination condition
    right_index: usize,
    /// value of A for the next operation
    successor_a: Register,
}

fn solve_part2(computer: &Computer) -> Option<Register> {
    let mut queue = VecDeque::new();
    queue.push_back(SearchNode {
        right_index: 0,
        successor_a: 0,
    });

    let mut min_a = None;

    while let Some(SearchNode {
        right_index,
        successor_a,
    }) = queue.pop_front()
    {
        let index = computer.program.len() - 1 - right_index;
        let expected_b = computer.program[index] as Register;

        for three_bits in 0..8 {
            let a = three_bits | (successor_a << 3);
            let b = apply_cycle(a);

            eprintln!("check: a           = {a:060b}");
            eprintln!("       successor_a = {successor_a:060b}");
            eprintln!("       {b} ==? {expected_b} (expected)");
            eprintln!();

            if b == expected_b {
                if index == 0 {
                    min_a = min_a.min(Some(a));
                } else {
                    queue.push_back(SearchNode {
                        right_index: right_index + 1,
                        successor_a: a,
                    });
                }
            }
        }
    }

    min_a
}

pub fn part2(input: &Path) -> Result<()> {
    println!(
        "WARNING! This is not a general solution! It just runs my particular input, backwards"
    );
    let input = std::fs::read_to_string(input).context("reading input file")?;
    let mut computer = Computer::from_input(&input).context("initializing computer")?;
    let a = solve_part2(&computer).context("no solution to part 2")?;
    // check our results
    debug_assert_eq!(
        {
            computer.registers[0] = a;
            while computer.tick().context("processing an instruction")? {}
            computer.output
        },
        computer.program,
        "program must be a quine"
    );
    println!("value of a for quine: {a}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::*;
    use rstest::rstest;

    #[rstest]
    #[case::part1([729,0,0],[0,1,5,4,3,0].into(),"4,6,3,5,6,3,5,2,1,0")]
    #[case::part2_quine([117440, 0, 0],[0,3,5,4,3,0].into(),"0,3,5,4,3,0")]
    fn example(
        #[case] registers: [Register; 3],
        #[case] program: Vec<ThreeBit>,
        #[case] expect: &str,
    ) {
        let mut computer = Computer::new(program);
        computer.registers = registers;
        // execute the whole program
        while computer.tick().expect("this program should work") {}
        let output = computer.prepare_output();
        assert_eq!(output, expect);
    }

    #[test]
    fn example_solve_part2() {
        let computer = Computer::new([0, 3, 5, 4, 3, 0].into());
        let computed_a = solve_part2(&computer);
        assert_eq!(computed_a, Some(117440));
    }
}
