use aoclib::geometry::{tile::DisplayWidth, Direction, Point};
use color_eyre::Result;
use std::path::Path;

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

        let in_region = |point| region_map.in_bounds(point) && region_map[point] == id;

        for (point, tile) in region_map.iter() {
            if *tile == id {
                if first_point.is_none() {
                    first_point = Some(point);
                }

                area += 1;
                perimeter += 4;
                for direction in [Direction::Right, Direction::Down] {
                    if in_region(point + direction) {
                        perimeter -= 2;
                    }
                }
            }
        }

        let first_point = first_point?;

        let mut num_sides = 0;
        let count_edges =
            |travel_direction: Direction, scan_direction: Direction, initial_point: Point| {
                let (dx, dy) = travel_direction.deltas();
                let mut was_edge = false;
                let mut point_was_in_region = false;
                let mut edges = 0;

                for point in region_map.project(initial_point, dx, dy) {
                    let point_is_in_region = in_region(point);
                    let is_edge = point_is_in_region != in_region(point + scan_direction);

                    if is_edge && (!was_edge || point_is_in_region != point_was_in_region) {
                        edges += 1;
                    }

                    was_edge = is_edge;
                    point_was_in_region = point_is_in_region;
                }

                edges
            };

        // projecting up from the bottom, find all edges according to each projection
        num_sides += count_edges(Direction::Up, Direction::Left, region_map.bottom_left());
        for point in region_map.edge(Direction::Down) {
            num_sides += count_edges(Direction::Up, Direction::Right, point);
        }
        // projecting right from the left, find all edges according to the projections
        num_sides += count_edges(Direction::Right, Direction::Down, region_map.bottom_left());
        for point in region_map.edge(Direction::Left) {
            num_sides += count_edges(Direction::Right, Direction::Up, point);
        }

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
