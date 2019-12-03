use std::io::{self};
use std::str::FromStr;
use std::collections::HashMap;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T, E = Error> = std::result::Result<T, E>;

fn main() -> Result<()> {
	let mut path1 = String::new();
	io::stdin().read_line(&mut path1)?;
	let mut path2 = String::new();
	io::stdin().read_line(&mut path2)?;

	let (d, s) = solve(&path1, &path2)?;
	println!("p1 = {}", d);
	println!("p2 = {}", s);
	
	Ok(())
}

fn solve(path1: &str, path2: &str) -> Result<(i32, i32)> {
	let w1 = trace(&path1)?;
	let w2 = trace(&path2)?;

	let mut distance = std::i32::MAX;
	let mut steps = std::i32::MAX;
	
	for (p, s1) in &w1 {
		if let Some(s2) = w2.get(p) {
			let d = p.0.abs() + p.1.abs();
			distance = distance.min(d);

			steps = steps.min(s1 + s2);
		}
	}

	Ok((distance, steps))
}

fn trace(input: &str) -> Result<HashMap<(i32, i32), i32>> {
	let mut p: (i32, i32) = (0, 0);
	let mut w = HashMap::new();
	let mut s = 1;

	for c in input.trim().split(',') {
		let c = c.parse::<Cmd>()?;
		let (dx, dy) = match c.dir {
			Dir::U => ( 0,  1),
			Dir::D => ( 0, -1),
			Dir::L => (-1,  0),
			Dir::R => ( 1,  0),
		};
		for _ in 0..c.len {
			p.0 += dx;
			p.1 += dy;

			w.entry(p).or_insert(s);
			s += 1;
		}
	}
	
	Ok(w)
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
enum Dir {
	U = 0,
	D,
	L,
	R,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
struct Cmd {
	dir: Dir,
	len: i32
}

impl FromStr for Cmd {
	type Err = Error;

	fn from_str(s: &str) -> Result<Cmd> {
		let dir = match s.chars().nth(0) {
			Some('U') => Dir::U,
			Some('D') => Dir::D,
			Some('L') => Dir::L,
			Some('R') => Dir::R,
			_ => Err("ivalid direction")? ,
		};

		let len: i32 = s[1..].parse()?;
		Ok(Cmd { dir, len })
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn cmd_parsing() {
		assert_eq!("U1".parse().ok(),    Some(Cmd { dir: Dir::U, len: 1 }));
		assert_eq!("D12".parse().ok(),   Some(Cmd { dir: Dir::D, len: 12 }));
		assert_eq!("L123".parse().ok(),  Some(Cmd { dir: Dir::L, len: 123 }));
		assert_eq!("R1234".parse().ok(), Some(Cmd { dir: Dir::R, len: 1234 }));
	}

	#[test]
	fn example1() {
		let path1 = "R8,U5,L5,D3";
		let path2 = "U7,R6,D4,L4";
		assert_eq!(solve(&path1, &path2).ok(), Some((6, 30)));
	}

	#[test]
	fn example2() {
		let path1 = "R75,D30,R83,U83,L12,D49,R71,U7,L72";
		let path2 = "U62,R66,U55,R34,D71,R55,D58,R83";
		assert_eq!(solve(&path1, &path2).ok(), Some((159, 610)));
	}

	#[test]
	fn example3() {
		let path1 = "R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51";
		let path2 = "U98,R91,D20,R16,D67,R40,U7,R15,U6,R7";
		assert_eq!(solve(&path1, &path2).ok(), Some((135, 410)));
	}
}
