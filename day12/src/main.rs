use std::io::{self, Read};
use std::cmp::Ordering;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T, E = Error> = std::result::Result<T, E>;

type Vec3 = [i64; 3];

fn main() -> Result<()> {
	let mut input = String::new();
	io::stdin().read_to_string(&mut input)?;

	println!("p1: {}", solve_part1(&input)?);
	println!("p2: {}", solve_part2(&input)?);
	
	Ok(())
}

fn solve_part1(input: &str) -> Result<i64> {
	let mut ps: Vec<Vec3> = input.trim().lines().map(parse_vec).collect::<Result<_>>()?;
	let mut vs = vec![[0, 0, 0]; ps.len()];
	let mut e = 0;
	for _ in 0..1000 {
		step(&mut ps, &mut vs);
		e = energy(&ps, &vs);
	}
	Ok(e)
}

fn solve_part2(input: &str) -> Result<i64> {
	let mut ps: Vec<Vec3> = input.trim().lines().map(parse_vec).collect::<Result<_>>()?;
	let mut vs = vec![[0, 0, 0]; ps.len()];
	let initial = ps.clone();
	let mut steps = 1;
	let mut cycles = [-1, -1, -1];
	let mut known = 0;
	while known != cycles.len() {
		step(&mut ps, &mut vs);
		steps += 1;

		for c in 0..cycles.len() {
			if cycles[c] != -1 {
				continue;
			}
			let mut reset = true;
			for i in 0..ps.len() {
				if ps[i][c] != initial[i][c] {
					reset = false;
					break;
				}
			}
			if reset {
				cycles[c] = steps;
				known += 1;
			}
		}
	}

	Ok(lcm(cycles[0], lcm(cycles[1], cycles[2])))
}

fn gcd(mut x: i64, mut y: i64) -> i64 {
	while y != 0 {
		let t = y;
		y = x % y;
		x = t;
	}
	x
}

fn lcm(x: i64, y: i64) -> i64 {
	if x == y && x == 0 {
		0
	} else {
		x.abs() * y.abs() / gcd(x, y)
	}
}

fn energy(ps: &[Vec3], vs: &[Vec3]) -> i64 {
	let mut e = 0;
	for i in 0..ps.len() {
		let pot: i64 = ps[i].iter().map(|v| v.abs()).sum();
		let kin: i64 = vs[i].iter().map(|v| v.abs()).sum();
		e += pot * kin;
	}
	e
}

fn step(ps: &mut [Vec3], vs: &mut [Vec3]) {
	for i in 0..ps.len() {
		for j in 0..ps.len() {
			if i == j {
				continue;
			}

			for k in 0..3 {
				vs[i][k] = vs[i][k] + delta(ps[i][k], ps[j][k]);
			}
		}
	}

	for i in 0..ps.len() {
		for k in 0..3 {
			ps[i][k] += vs[i][k];
		}
	}
}

fn delta(left: i64, right: i64) -> i64 {
	match left.cmp(&right) {
		Ordering::Less => 1,
		Ordering::Equal => 0,
		Ordering::Greater => -1,
	}
}

fn parse_vec(input: &str) -> Result<Vec3> {
	let input = &input[0..input.len() - 1];
	let mut coords = input.trim().split(',');
	let (x, y, z) = (coords.next(), coords.next(), coords.next());
	match (x, y, z) {
		(Some(x), Some(y), Some(z)) => {
			let x = parse_coord(x)?;
			let y = parse_coord(y)?;
			let z = parse_coord(z)?;
			Ok([x ,y ,z])
		},
		_ => Err("failed to parse x,y,z")?,
	}
}

fn parse_coord(input: &str) -> Result<i64> {
	if let Some(v) = input.split('=').skip(1).next() {
		Ok(v.parse()?)
	} else {
		Err("failed to parse coordinate")?
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn vec_parsing() {
		let input = "<x=-4, y=-9, z=-3>";
		assert_eq!(parse_vec(&input).unwrap(), [-4, -9, -3]);
	}

	#[test]
	fn stepping() {
		let mut ps = [[-1, 0, 2], [2, -10, -7], [4, -8, 8], [3, 5, -1]];
		let mut vs = [[0, 0, 0], [0, 0, 0], [0, 0, 0], [0, 0, 0]];
		step(&mut ps, &mut vs);
		assert_eq![&ps[..], &[[2, -1, 1], [3, -7, -4], [1, -7, 5], [2, 2, 0]]];
		assert_eq![&vs[..], &[[3, -1, -1], [1, 3, 3], [-3, 1, -3], [-1, -3, 1]]];
	}

	#[test]
	fn energy_calculation() {
		let ps = [[2, 1, 3], [1, 8, 0], [3, 6, 1], [2, 0, 4]];
		let vs = [[3, 2, 1], [1, 1, 3], [3, 2, 3], [1, 1, 1]];
		assert_eq!(energy(&ps, &vs), 179);
	}

	#[test]
	fn gcd_calculation() {
		assert_eq!(gcd(1, 1), 1);
		assert_eq!(gcd(2, 1), 1);
		assert_eq!(gcd(7, 31), 1);
		assert_eq!(gcd(4, 16), 4);
		assert_eq!(gcd(5 * 3, 5 * 4), 5);
	}

	#[test]
	fn lcm_calculation() {
		assert_eq!(lcm(0, 0), 0);
		assert_eq!(lcm(1, 0), 0);
		assert_eq!(lcm(0, 1), 0);
		assert_eq!(lcm(1, 1), 1);
		assert_eq!(lcm(10, 20), 20);
		assert_eq!(lcm(5, 7), 5*7);
	}
}
