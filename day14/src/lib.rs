use aoclib::{geometry::Point, parse};
use color_eyre::{
    eyre::{eyre, Context as _},
    Result,
};
use core::f64;
use lazy_static::lazy_static;
use regex::Regex;
use std::{fmt, i32, path::Path, str::FromStr};

#[derive(Debug, Clone)]
struct Robot {
    position: Point,
    velocity: Point,
}

impl FromStr for Robot {
    type Err = color_eyre::eyre::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex =
                Regex::new(r"^p=(?<px>\d+),(?<py>\d+) v=(?<vx>-?\d+),(?<vy>-?\d+)$")
                    .expect("this regex is valid");
        };

        let captures = RE.captures(s).ok_or(eyre!("robot regex did not match"))?;
        let parse = |name| {
            captures
                .name(name)
                .unwrap_or_else(|| panic!("{name} not optional"))
                .as_str()
                .parse()
                .context(name)
        };
        Ok(Self {
            position: Point::new(parse("px")?, parse("py")?),
            velocity: Point::new(parse("vx")?, parse("vy")?),
        })
    }
}

impl Robot {
    fn teleport_in_bounds(&mut self, width: i32, height: i32) {
        self.position.x %= width;
        if self.position.x < 0 {
            self.position.x += width;
        }
        self.position.y %= height;
        if self.position.y < 0 {
            self.position.y += height;
        }
        debug_assert!(self.position.x >= 0);
        debug_assert!(self.position.y >= 0);
        debug_assert!(self.position.x < width);
        debug_assert!(self.position.y < height);
    }
}

#[derive(Debug, Clone)]
struct Simulation {
    width: i32,
    height: i32,
    elapsed_seconds: u32,
    robots: Vec<Robot>,
}

#[derive(Debug, Default, Clone, Copy)]
struct Digit(u8);

impl fmt::Display for Digit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;
        let v = self.0;
        match v {
            0 => f.write_char('.'),
            10.. => f.write_char('^'),
            _ => write!(f, "{v}"),
        }
    }
}

impl aoclib::geometry::tile::DisplayWidth for Digit {
    const DISPLAY_WIDTH: usize = 1;
}

type Map = aoclib::geometry::Map<Digit>;

impl Simulation {
    fn new(width: i32, height: i32, robots: impl IntoIterator<Item = Robot>) -> Self {
        Self {
            width,
            height,
            elapsed_seconds: 0,
            robots: robots.into_iter().collect(),
        }
    }

    fn tick(&mut self) {
        self.multitick(1);
    }

    fn multitick(&mut self, seconds: u32) {
        self.elapsed_seconds += seconds;
        for robot in self.robots.iter_mut() {
            robot.position += robot.velocity * seconds as _;
            robot.teleport_in_bounds(self.width, self.height);
        }
    }

    fn safety_factor(&self) -> u32 {
        let bottom_left = self
            .robots
            .iter()
            .filter(|robot| robot.position.x < self.width / 2 && robot.position.y < self.height / 2)
            .count();
        let top_left = self
            .robots
            .iter()
            .filter(|robot| robot.position.x < self.width / 2 && robot.position.y > self.height / 2)
            .count();
        let bottom_right = self
            .robots
            .iter()
            .filter(|robot| robot.position.x > self.width / 2 && robot.position.y < self.height / 2)
            .count();
        let top_right = self
            .robots
            .iter()
            .filter(|robot| robot.position.x > self.width / 2 && robot.position.y > self.height / 2)
            .count();
        // dbg!(bottom_left, top_left, bottom_right, top_right);
        (bottom_left * top_left * bottom_right * top_right)
            .try_into()
            .expect("safety factor fits in 32 bits")
    }

    fn stdev(&self, dimension: impl Fn(&Robot) -> i32) -> f64 {
        let n = self.robots.len() as f64;
        let mean = self
            .robots
            .iter()
            .map(&dimension)
            .map(|d| d as f64)
            .sum::<f64>()
            / n;
        (self
            .robots
            .iter()
            .map(&dimension)
            .map(|d| {
                let delta = d as f64 - mean;
                delta * delta
            })
            .sum::<f64>()
            / n)
            .sqrt()
    }

    fn cluster(&self) -> f64 {
        self.stdev(|robot| robot.position.x) * self.stdev(|robot| robot.position.y)
    }

    fn make_map(&self) -> Map {
        let mut map = Map::new(self.width as _, self.height as _);
        for robot in &self.robots {
            assert!(map.in_bounds(robot.position));
            map[robot.position].0 += 1;
        }
        map.flip_vertical()
    }
}

pub fn part1(input: &Path) -> Result<()> {
    let mut simulation = Simulation::new(101, 103, parse::<Robot>(input)?);
    simulation.multitick(100);
    let safety_factor = simulation.safety_factor();
    println!("safety factor: {safety_factor}");
    Ok(())
}

pub fn part2(input: &Path) -> Result<()> {
    const N_TO_CHECK: usize = 25_000;
    let mut simulation = Simulation::new(101, 103, parse::<Robot>(input)?);
    let mut min_cluster = f64::MAX;

    for _ in 0..N_TO_CHECK {
        simulation.tick();
        let cluster = simulation.cluster();
        if cluster < min_cluster {
            println!(
                "{} ({cluster}):\n{}",
                simulation.elapsed_seconds,
                simulation.make_map()
            );
            min_cluster = cluster;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    mod part1 {
        use aoclib::input::parse_str;

        use crate::*;

        fn example() -> Simulation {
            let robots = "
p=0,4 v=3,-3
p=6,3 v=-1,-3
p=10,3 v=-1,2
p=2,0 v=2,-1
p=0,0 v=1,3
p=3,0 v=-2,-2
p=7,6 v=-1,-3
p=3,0 v=-1,-2
p=9,3 v=2,3
p=7,3 v=-1,2
p=2,4 v=2,-3
p=9,5 v=-3,-3
            "
            .trim();
            Simulation::new(11, 7, parse_str(robots).expect("can parse example robots"))
        }

        #[test]
        fn teleport() {
            let mut simulation = example();
            simulation.robots = vec![Robot {
                position: Point::new(2, 4),
                velocity: Point::new(2, -3),
            }];

            assert_eq!(
                simulation.robots[0].position,
                Point::new(2, 4),
                "initial state"
            );
            simulation.tick();
            assert_eq!(
                simulation.robots[0].position,
                Point::new(4, 1),
                "after 1 second"
            );
            simulation.tick();
            assert_eq!(
                simulation.robots[0].position,
                Point::new(6, 5),
                "after 2 seconds"
            );
            simulation.tick();
            assert_eq!(
                simulation.robots[0].position,
                Point::new(8, 2),
                "after 3 seconds"
            );
            simulation.tick();
            assert_eq!(
                simulation.robots[0].position,
                Point::new(10, 6),
                "after 4 seconds"
            );
            simulation.tick();
            assert_eq!(
                simulation.robots[0].position,
                Point::new(1, 3),
                "after 5 seconds"
            );
            assert_eq!(simulation.elapsed_seconds, 5);
        }

        #[test]
        fn quadrants() {
            let mut simulation = example();
            eprintln!("{}", simulation.make_map());

            simulation.multitick(100);
            assert_eq!(simulation.elapsed_seconds, 100);

            eprint!("{}", simulation.make_map());

            assert_eq!(simulation.safety_factor(), 12);
        }
    }
}
