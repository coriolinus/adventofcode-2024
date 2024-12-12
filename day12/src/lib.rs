use aoclib::geometry::{tile::DisplayWidth, Direction, Point};
use color_eyre::Result;
use std::{collections::HashSet, path::Path};

#[derive(Debug, Clone, Copy, derive_more::FromStr, derive_more::Into)]
struct Char(char);

impl DisplayWidth for Char {
    const DISPLAY_WIDTH: usize = 1;
}

type RawMap = aoclib::geometry::map::Map<Char>;
type RegionId = u16;
type RegionMap = aoclib::geometry::Map<RegionId>;

fn create_region_map<Tile>(map: &aoclib::geometry::Map<Tile>) -> RegionMap
where
    Tile: Copy + Eq,
{
    fn paint_region<TileInner>(
        map: &aoclib::geometry::Map<TileInner>,
        region_map: &mut RegionMap,
        value: TileInner,
        region_id: RegionId,
        point: Point,
    ) where
        TileInner: Copy + Eq,
    {
        if !map.in_bounds(point) || map[point] != value || region_map[point] != 0 {
            return;
        }
        region_map[point] = region_id;
        for direction in Direction::iter() {
            paint_region(map, region_map, value, region_id, point + direction);
        }
    }

    let mut region_map = RegionMap::new(map.width(), map.height());
    let mut next_region_id = 1;
    for (point, tile) in map.iter() {
        if region_map[point] == 0 {
            paint_region(map, &mut region_map, *tile, next_region_id, point);
            next_region_id += 1;
        }
    }

    debug_assert!(!region_map.iter().any(|(_point, tile)| *tile == 0));

    region_map
}

#[derive(Debug)]
struct RegionGeometry {
    #[allow(unused)]
    first_point: Point,
    area: u32,
    perimeter: u32,
    num_sides: u32,
}

impl RegionGeometry {
    fn analyze(region_map: &RegionMap, id: RegionId) -> Option<Self> {
        let mut first_point = None;
        let mut area = 0;
        let mut perimeter = 0;

        for (point, tile) in region_map.iter() {
            if *tile == id {
                if first_point.is_none() {
                    first_point = Some(point);
                }

                area += 1;
                perimeter += 4;
                for direction in [Direction::Right, Direction::Down] {
                    let adj = point + direction;
                    if region_map.in_bounds(adj) && region_map[adj] == id {
                        perimeter -= 2;
                    }
                }
            }
        }

        let in_region = |point| region_map.in_bounds(point) && region_map[point] == id;

        let first_point = first_point?;
        let mut point = first_point;
        let mut num_sides = Direction::iter()
            .filter(|&direction| !in_region(point + direction))
            .count() as _;
        if num_sides < 4 {
            // subtract 1 side because we're going to add it back in at the end
            num_sides -= 1;
            // start by finding the first direction which is in-region which is adjacent to a direction out of region
            // scan probably comes from the left most times, so this should return early most times
            let mut travel_direction = Direction::Up;
            let mut was_in_region = { in_region(point + travel_direction.turn_left()) }; // default to kick off the search
            for _ in 0..4 {
                let is_in_region = in_region(point + travel_direction);
                if is_in_region && !was_in_region {
                    // we've found our valid initial travel direction
                    break;
                }
                travel_direction = travel_direction.turn_right();
                was_in_region = is_in_region;
            }

            // now that we have a valid point and starting direction, we can trace the perimeter (clockwise),
            // adding sides each time we turn
            let mut visited_points = HashSet::with_capacity(area);
            visited_points.insert(point);
            point += travel_direction;
            while visited_points.len() < area {
                if point == first_point {
                    // we have completed a loop but not yet found all of our points, so we need to reset somehow
                    todo!()
                }
                if in_region(point + travel_direction.turn_left()) {
                    travel_direction = travel_direction.turn_left();
                    num_sides += 1;
                } else if in_region(point + travel_direction) {
                    // no change in number of sides or travel direction, but we need to catch the case
                } else if in_region(point + travel_direction.turn_right()) {
                    travel_direction = travel_direction.turn_right();
                    num_sides += 1;
                } else {
                    travel_direction = travel_direction.reverse();
                    num_sides += 2;
                }
                visited_points.insert(point);
                point += travel_direction;
            }
        }

        let area = area
            .try_into()
            .expect("we don't overflow u32 in the number of visited points");

        Some(Self {
            first_point,
            area,
            perimeter,
            num_sides,
        })
    }

    fn fence_price(&self) -> u32 {
        self.area * self.perimeter
    }

    fn fence_price_pt2(&self) -> u32 {
        self.area * self.num_sides
    }
}

pub fn part1(input: &Path) -> Result<()> {
    let map = <RawMap as TryFrom<&Path>>::try_from(input)?.convert_tile_type::<char>();
    let region_map = create_region_map(&map);
    let mut total_fence_price = 0;

    for region_id in 1.. {
        let Some(geometry) = RegionGeometry::analyze(&region_map, region_id) else {
            break;
        };

        total_fence_price += geometry.fence_price();
    }

    println!("total fence price: {total_fence_price}");
    Ok(())
}

pub fn part2(input: &Path) -> Result<()> {
    let map = <RawMap as TryFrom<&Path>>::try_from(input)?.convert_tile_type::<char>();
    let region_map = create_region_map(&map);
    let mut total_fence_price = 0;

    for region_id in 1.. {
        let Some(geometry) = RegionGeometry::analyze(&region_map, region_id) else {
            break;
        };

        total_fence_price += geometry.fence_price_pt2();
    }

    println!("total fence price: {total_fence_price}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::*;
    type CMap = aoclib::geometry::Map<char>;

    fn tiny_example() -> CMap {
        let data = "
AAAA
BBCD
BBCC
EEEC
        "
        .trim();
        <RawMap as TryFrom<&str>>::try_from(data)
            .unwrap()
            .convert_tile_type()
    }

    fn small_example() -> CMap {
        let data = "
OOOOO
OXOXO
OOOOO
OXOXO
OOOOO
        "
        .trim();
        <RawMap as TryFrom<&str>>::try_from(data)
            .unwrap()
            .convert_tile_type()
    }

    fn e_example() -> CMap {
        let data = "
EEEEE
EXXXX
EEEEE
EXXXX
EEEEE
        "
        .trim();
        <RawMap as TryFrom<&str>>::try_from(data)
            .unwrap()
            .convert_tile_type()
    }

    fn abba_example() -> CMap {
        let data = "
AAAAAA
AAABBA
AAABBA
ABBAAA
ABBAAA
AAAAAA
        "
        .trim();
        <RawMap as TryFrom<&str>>::try_from(data)
            .unwrap()
            .convert_tile_type()
    }

    fn big_example() -> CMap {
        let data = "
RRRRIICCFF
RRRRIICCCF
VVRRRCCFFF
VVRCCCJFFF
VVVVCJJCFE
VVIVCCJJEE
VVIIICJJEE
MIIIIIJJEE
MIIISIJEEE
MMMISSJEEE
        "
        .trim();
        <RawMap as TryFrom<&str>>::try_from(data)
            .unwrap()
            .convert_tile_type()
    }

    mod part1 {
        use super::*;
        use rstest::rstest;

        #[rstest]
        #[case::tiny(tiny_example(), 5, 140)]
        #[case::small(small_example(), 5, 772)]
        #[case::big(big_example(), 11, 1930)]
        fn analyze_regions(
            #[case] map: CMap,
            #[case] expect_n_regions: RegionId,
            #[case] expect_total_price: u32,
        ) {
            let region_map = create_region_map(&map);
            assert!(!region_map.iter().any(|(_point, id)| *id == 0));

            let max_region_id = region_map.iter().map(|(_point, id)| *id).max().unwrap();
            assert_eq!(max_region_id, expect_n_regions);

            let mut total_price = 0;
            for id in 1..=max_region_id {
                let geometry = RegionGeometry::analyze(&region_map, id).unwrap();
                dbg!(id, &geometry);
                total_price += geometry.fence_price();
            }
            assert_eq!(total_price, expect_total_price);
        }
    }

    mod part2 {
        use super::*;
        use rstest::rstest;

        #[rstest]
        #[case::tiny(tiny_example(), 80)]
        #[case::small(small_example(), 436)]
        #[case::e(e_example(), 236)]
        #[case::abba(abba_example(), 368)]
        #[case::big(big_example(), 1206)]
        fn analyze_regions(#[case] map: CMap, #[case] expect_total_price_pt2: u32) {
            let region_map = create_region_map(&map);

            let mut total_price = 0;
            for region_id in 1.. {
                let Some(geometry) = RegionGeometry::analyze(&region_map, region_id) else {
                    break;
                };
                dbg!(&geometry);

                total_price += geometry.fence_price_pt2();
            }
            assert_eq!(total_price, expect_total_price_pt2);
        }
    }
}
