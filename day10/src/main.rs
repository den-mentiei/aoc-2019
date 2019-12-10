use std::io::{self, Read};
use std::collections::HashSet;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T, E = Error> = std::result::Result<T, E>;

fn main() -> Result<()> {
	let mut input = String::new();
	io::stdin().read_to_string(&mut input)?;

	let asteroids = parse(&input);

	println!("p1: {}", solve_part1(&asteroids));
	let (x, y) = vaporize(&asteroids, 200);
	println!("p2: {:?}", x * 100 + y);

	Ok(())
}

fn solve_part1(asteroids: &[(i64, i64)]) -> usize {
	let (_, los) = place_station(asteroids);
	los.len()
}

fn vaporize(asteroids: &[(i64, i64)], i: usize) -> (i64, i64) {
	let (s, los) = place_station(asteroids);
	let mut rays = los
		.iter()
		.map(|&(dx, dy)| ((dx, dy), (dx as f32).atan2(dy as f32)))
		.collect::<Vec<_>>();

	rays.sort_by(|(_, a0), (_, a1)| a1.partial_cmp(a0).unwrap());

	let asteroids = asteroids.into_iter().collect::<HashSet<_>>();

	let (dx, dy) = rays[i - 1].0;
	let (mut x, mut y) = (s.0 + dx, s.1 + dy);
	while !asteroids.contains(&(x, y)) {
		x += dx;
		y += dy;
	}
	
	(x, y)
}

fn place_station(asteroids: &[(i64, i64)]) -> ((i64, i64), HashSet<(i64, i64)>) {
	let mut s = &asteroids[0];
	let mut los = HashSet::new();

	for a0 in asteroids {
		let l = find_visible(&a0, asteroids);
		if l.len() > los.len() {
			s = a0;
			los = l;
		}
	}

	(*s, los)
}

fn find_visible(a: &(i64, i64), asteroids: &[(i64, i64)]) -> HashSet<(i64, i64)> {
	let mut los = HashSet::new();
	for other in asteroids {
		if a == other {
			continue;
		}

		let dx = other.0 - a.0;
		let dy = other.1 - a.1;
		let d = gcd(dx, dy).abs();
		los.insert((dx / d, dy / d));
	}
	los
}

fn gcd(mut x: i64, mut y: i64) -> i64 {
	while y != 0 {
		let t = y;
		y = x % y;
		x = t;
	}
	x
}

fn parse(input: &str) -> Box<[(i64, i64)]> {
	input
		.trim()
		.lines()
		.enumerate()
		.flat_map(|(y, l)| {
			l.trim().bytes().enumerate().filter_map(move |(x, b)| {
				if b == b'#' { Some((x as i64, y as i64)) } else { None }
			}
		)})
		.collect::<Vec<_>>()
		.into_boxed_slice()
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parsing() {
		let result = parse(&r"
			..#
			#..
			.#.
		");

		assert_eq!(&result[..], &[(2,0), (0,1), (1,2)]);
	}

	#[test]
	fn check_gcd() {
		assert_eq!(gcd(1, 1), 1);
		assert_eq!(gcd(2, 1), 1);
		assert_eq!(gcd(7, 31), 1);
		assert_eq!(gcd(4, 16), 4);
		assert_eq!(gcd(5 * 3, 5 * 4), 5);
	}
	
	#[test]
	fn example1() {
		let input = r"
			.#..#
			.....
			#####
			....#
			...##
		";
		let asteroids = parse(&input);
		assert_eq!(solve_part1(&asteroids), 8);
	}

	#[test]
	fn example2() {
		let input = r"
			......#.#.
			#..#.#....
			..#######.
			.#.#.###..
			.#..#.....
			..#....#.#
			#..#....#.
			.##.#..###
			##...#..#.
			.#....####
		";
		let asteroids = parse(&input);
		assert_eq!(solve_part1(&asteroids), 33);
	}

	#[test]
	fn example3() {
		let input = r"
			#.#...#.#.
			.###....#.
			.#....#...
			##.#.#.#.#
			....#.#.#.
			.##..###.#
			..#...##..
			..##....##
			......#...
			.####.###.
		";
		let asteroids = parse(&input);
		assert_eq!(solve_part1(&asteroids), 35);
	}

	#[test]
	fn example4() {
		let input = r"
			.#..#..###
			####.###.#
			....###.#.
			..###.##.#
			##.##.#.#.
			....###..#
			..#.#..#.#
			#..#.#.###
			.##...##.#
			.....#.#..
		";
		let asteroids = parse(&input);
		assert_eq!(solve_part1(&asteroids), 41);
	}

	#[test]
	fn example5() {
		let input = r"
			.#..##.###...#######
			##.############..##.
			.#.######.########.#
			.###.#######.####.#.
			#####.##.#.##.###.##
			..#####..#.#########
			####################
			#.####....###.#.#.##
			##.#################
			#####.##.###..####..
			..######..##.#######
			####.##.####...##..#
			.#####..#.######.###
			##...#.##########...
			#.##########.#######
			.####.#.###.###.#.##
			....##.##.###..#####
			.#.#.###########.###
			#.#.#.#####.####.###
			###.##.####.##.#..##
		";
		let asteroids = parse(&input);
		assert_eq!(solve_part1(&asteroids), 210);
	}

	#[test]
	fn example6() {
		let input = r"
			.#..##.###...#######
			##.############..##.
			.#.######.########.#
			.###.#######.####.#.
			#####.##.#.##.###.##
			..#####..#.#########
			####################
			#.####....###.#.#.##
			##.#################
			#####.##.###..####..
			..######..##.#######
			####.##.####...##..#
			.#####..#.######.###
			##...#.##########...
			#.##########.#######
			.####.#.###.###.#.##
			....##.##.###..#####
			.#.#.###########.###
			#.#.#.#####.####.###
			###.##.####.##.#..##
		";
		let asteroids = parse(&input);
		assert_eq!(vaporize(&asteroids, 1),   (11, 12));
		assert_eq!(vaporize(&asteroids, 2),   (12, 1));
		assert_eq!(vaporize(&asteroids, 200), (8, 2));
	}
}
