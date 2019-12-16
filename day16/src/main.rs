use std::io::{self, Read};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T, E = Error> = std::result::Result<T, E>;

fn main() -> Result<()> {
	let mut input = String::new();
	io::stdin().read_to_string(&mut input)?;

	println!("p1: {}", solve_part1(&input));
	println!("p2: {}", solve_part2(&input));

	Ok(())
}

fn solve_part1(input: &str) -> u64 {
	let base = [0, 1, 0, -1];

	let mut digits: Vec<u8> = input
		.trim()
		.bytes()
		.map(|b| b - b'0')
		.collect();

	for _ in 0..100 {
		for i in 0..digits.len() {
			let d: i64 = digits
				.iter()
				.enumerate()
				.skip(i)
				.map(|(j, d)| (*d as i64) * base[((j + 1) / (i + 1)) % 4])
				.sum();
			digits[i] = (d.abs() % 10) as u8;
		}
	}

	digits
		.iter()
		.take(8)
		.fold(0, |a, &x| x as u64 + a * 10)
}

fn solve_part2(input: &str) -> u64 {
	let input = input.trim();
	let offset: usize = input[0..7].parse().expect("invalid input");
	let digits = input.bytes().map(|b| b - b'0');
	let n = input.len();
	let mut digits: Vec<u8> = digits
		.cycle()
		.skip(offset % n)
		.take(n * 10_000 - offset)
		.collect();

	for _ in 0..100 {
		for i in (0..digits.len() - 1).rev() {
			digits[i] = (digits[i] + digits[i + 1]) % 10 as u8;
		}
	}

	digits
		.iter()
		.take(8)
		.fold(0, |a, &x| x as u64 + a * 10)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn examples1() {
		assert_eq!(solve_part1(&"80871224585914546619083218645595"), 24176176);
		assert_eq!(solve_part1(&"19617804207202209144916044189917"), 73745418);
		assert_eq!(solve_part1(&"69317163492948606335995924319873"), 52432133);
	}

	#[test]
	fn examples2() {
		assert_eq!(solve_part2(&"03036732577212944063491565474664"), 84462026);
		assert_eq!(solve_part2(&"02935109699940807407585447034323"), 78725270);
		assert_eq!(solve_part2(&"03081770884921959731165446850517"), 53553731);
	}
}
