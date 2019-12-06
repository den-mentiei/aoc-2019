use std::io::{self, Read};
use std::collections::HashMap;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T, E = Error> = std::result::Result<T, E>;

fn main() -> Result<()> {
	let mut input = String::new();
	io::stdin().read_to_string(&mut input)?;

	let scheme = parse(&input)?;
	
	println!("p1: {}", solve_part1(&scheme));
	println!("p2: {}", solve_part2(&scheme));
	
	Ok(())
}

fn solve_part1(scheme: &Scheme) -> i64 {
	let mut total = 0;
	for (id, _) in scheme.items.iter().enumerate() {
		total += trace(&scheme, id, |_,_| None, |steps| Some(steps)).unwrap_or(0);
	}

	total
}

fn solve_part2(scheme: &Scheme) -> i64 {
	let san_id = *scheme.to_id.get("SAN").expect("Santa node is required");
	let you_id = *scheme.to_id.get("YOU").expect("You node is required");

	trace(
		&scheme,
		you_id,
		|steps, current| { try_reach(&scheme, san_id, current).map(|s| s + steps - 2) },
		|_| None)
		.unwrap_or(0)
}

fn try_reach(scheme: &Scheme, from: usize, to: usize) -> Option<i64> {
	trace(&scheme, from, |steps,id| if id == to { Some(steps) } else { None }, |_| None)
}

fn trace<F, U>(scheme: &Scheme, from: usize, should_stop: F, map_result: U) -> Option<i64>
	where F: Fn(i64, usize) -> Option<i64>,
		  U: Fn(i64) -> Option<i64>
{
	let mut steps = 0;
	
	let mut id = from;
	loop {
		if let Some(result) = should_stop(steps, id) {
			return Some(result);
		}

		if let Some(center) = scheme.links.get(&id) {
			steps += 1;
			id = *center;
		} else {
			return map_result(steps);
		}
	}
}

struct Scheme<'a> {
	items: Vec<&'a str>,
	to_id: HashMap<&'a str, usize>,
	// Object with id "key' orbits around an object with id "value".
	links: HashMap<usize, usize>,
}

fn parse<'a>(input: &'a str) -> Result<Scheme<'a>> {
	let mut items = Vec::new();
	let mut to_id = HashMap::new();
	let mut links = HashMap::new();
	
	for line in input.trim().lines().map(|l| l.trim()) {
		let mut pair = line.split(')');
		let a = pair.next().ok_or("failed to parse first item")?;
		let b = pair.next().ok_or("failed to parse second item")?;

		let a_id = *to_id.entry(a).or_insert_with(|| {
			items.push(a);
			items.len() - 1
		});
		let b_id = *to_id.entry(b).or_insert_with(|| {
			items.push(a);
			items.len() - 1
		});
		if let Some(_) = links.insert(b_id, a_id) {
			return Err("unexpected multiple orbits per object")?;
		}
	}

	Ok(Scheme { items: items, to_id: to_id, links: links })
}

#[cfg(test)]
mod tests {
	use super::*;

	/*
			G - H       J - K - L
			/           /
		COM - B - C - D - E - F
					\
					I
	*/
	const INPUT: &'static str = r"
		COM)B
		B)C
		C)D
		D)E
		E)F
		B)G
		G)H
		D)I
		E)J
		J)K
		K)L
	";

	#[test]
	fn reaching_possible() {
		let scheme = parse(&INPUT).unwrap();
		let root_id = *scheme.to_id.get("COM").unwrap();
		let node_id = *scheme.to_id.get("B").unwrap();
		let steps = try_reach(&scheme, node_id, root_id);
		assert_eq!(steps, Some(1));
	}

	#[test]
	fn reaching_impossible() {
		let scheme = parse(&INPUT).unwrap();
		let to_id = *scheme.to_id.get("I").unwrap();
		let from_id = *scheme.to_id.get("F").unwrap();
		let steps = try_reach(&scheme, from_id, to_id);
		assert_eq!(steps, None);
	}
	
	#[test]
	fn examples1() {
		let scheme = parse(&INPUT).unwrap();
		assert_eq!(solve_part1(&scheme), 42);
	}

	#[test]
	fn examples2() {
		let input = r"
			COM)B
			B)C
			C)D
			D)E
			E)F
			B)G
			G)H
			D)I
			E)J
			J)K
			K)L
			K)YOU
			I)SAN
		";
		let scheme = parse(&input).unwrap();
		assert_eq!(solve_part2(&scheme), 4);
	}
}
