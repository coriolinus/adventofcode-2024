use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use aoclib::geometry::{Direction, Point};

type DigitMap = aoclib::geometry::map::Map<aoclib::geometry::map::tile::Digit>;
type Map = aoclib::geometry::map::Map<u8>;
type DistinctTrailsMemos = HashMap<Point, u32>;
type DistinctPeaksMemos = HashMap<Point, HashSet<Point>>;

fn distinct_trails_from_point(map: &Map, memos: &mut DistinctTrailsMemos, point: Point) -> u32 {
    if let Some(&score) = memos.get(&point) {
        return score;
    }
    if map[point] == 9 {
        return 1;
    }

    let mut score = 0;
    for adj in Direction::iter()
        .map(|direction| point + direction)
        .filter(|&adj| map.in_bounds(adj) && map[adj] == map[point] + 1)
    {
        let s = distinct_trails_from_point(map, memos, adj);
        score += s;
    }
    memos.insert(point, score);
    score
}

fn distinct_peaks_from_point(
    map: &Map,
    memos: &mut DistinctPeaksMemos,
    point: Point,
) -> Vec<Point> {
    if let Some(peaks) = memos.get(&point) {
        return peaks.iter().copied().collect();
    }
    if map[point] == 9 {
        let peaks = memos.entry(point).or_default();
        peaks.insert(point);
        return peaks.iter().copied().collect();
    }

    for adj in Direction::iter()
        .map(|direction| point + direction)
        .filter(|&adj| map.in_bounds(adj) && map[adj] == map[point] + 1)
    {
        for peak in distinct_peaks_from_point(map, memos, adj) {
            memos.entry(point).or_default().insert(peak);
        }
    }
    memos
        .get(&point)
        .map(|distinct_points| distinct_points.iter().copied().collect())
        .unwrap_or_default()
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let map = <DigitMap as TryFrom<&Path>>::try_from(input)?.convert_tile_type::<u8>();

    let mut memos = HashMap::new();
    let mut score_sum = 0;

    for point in map
        .iter()
        .filter_map(|(point, value)| (*value == 0).then_some(point))
    {
        let distinct_peaks = distinct_peaks_from_point(&map, &mut memos, point);
        score_sum += distinct_peaks.len();
    }

    println!("sum of scores of trailheads: {score_sum}");
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    let map = <DigitMap as TryFrom<&Path>>::try_from(input)?.convert_tile_type::<u8>();

    let mut memos = HashMap::new();
    let mut rating_sum = 0;

    for point in map
        .iter()
        .filter_map(|(point, value)| (*value == 0).then_some(point))
    {
        let rating = distinct_trails_from_point(&map, &mut memos, point);
        rating_sum += rating;
    }

    println!("sum of ratings of trailheads: {rating_sum}");
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("no solution found")]
    NoSolution,
}

#[cfg(test)]
mod tests {
    mod part1 {
        use crate::*;

        const EXAMPLE: &str = "
89010123
78121874
87430965
96549874
45678903
32019012
01329801
10456732
        ";

        fn example() -> Map {
            <DigitMap as TryFrom<&str>>::try_from(EXAMPLE.trim())
                .unwrap()
                .convert_tile_type()
        }

        #[test]
        fn trailhead_at_6_1() {
            let map = example();
            let mut memos = HashMap::new();
            let point = Point::new(6, 1);

            let score = distinct_peaks_from_point(&map, &mut memos, point).len();
            assert_eq!(score, 3);
        }
    }
}
