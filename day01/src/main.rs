use std::io::{self, Read};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T, E = Error> = std::result::Result<T, E>;

fn calc_fuel(mass: i64) -> i64 {
	mass / 3 - 2
}

fn calc_fuel_inc(mass: i64) -> i64 {
	let mut total: i64 = 0;
	let mut extra = calc_fuel(mass);

	while extra > 0 {
		total += extra;
		extra = calc_fuel(extra);
	}

	total
}

fn main() -> Result<()> {
	let mut input = String::new();
	io::stdin().read_to_string(&mut input)?;

	let mut p1: i64 = 0;
	let mut p2: i64 = 0;
	
	for l in input.lines() {
		let mass: i64 = l.parse()?;
		p1 += calc_fuel(mass);
		p2 += calc_fuel_inc(mass);
	}

	println!("p1 = {}", p1);
	println!("p2 = {}", p2);
	
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn examples_p1() {
		assert_eq!(calc_fuel(12), 2);
		assert_eq!(calc_fuel(14), 2);
		assert_eq!(calc_fuel(1969), 654);
		assert_eq!(calc_fuel(100756), 33583);
	}

	#[test]
	fn examples_p2() {
		assert_eq!(calc_fuel_inc(14), 2);
		assert_eq!(calc_fuel_inc(1969), 966);
	}
}
