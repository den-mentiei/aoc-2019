use std::io::{self, Read};

type Error = Box<dyn std::error::Error>;
type Result<T, E = Error> = std::result::Result<T, E>;

fn parse(input: &String) -> Result<Box<[usize]>> {
	let ram: Vec<usize> = input
		.trim()
		.split(',')
		.map(|s| s.parse::<usize>())
		.collect::<Result<Vec<_>, _>>()?;

	Ok(ram.into_boxed_slice())
}

fn execute(ram: &mut [usize]) {
	let mut pc: usize = 0;

	loop {
		match ram[pc] {
			1 => {
				let ai = ram[pc + 1];
				let bi = ram[pc + 2];
				let ci = ram[pc + 3];
				ram[ci] = ram[ai] + ram[bi];
				pc += 4;
			},
			2 => {
				let ai = ram[pc + 1];
				let bi = ram[pc + 2];
				let ci = ram[pc + 3];
				ram[ci] = ram[ai] * ram[bi];
				pc += 4;
			},
			99 => break,
			_ => panic!("unknown opcode"),
		}
	}
}

fn main() -> Result<()> {
	let mut input = String::new();
	io::stdin().read_to_string(&mut input)?;

	let initial = parse(&input)?;
	let mut ram = initial.clone();
	
	ram[1] = 12;
	ram[2] = 2;
	execute(&mut ram);

	println!("first -> {}", ram[0]);

	for noun in 0..100 {
		for verb in 0..100 {
			ram.copy_from_slice(&initial);

			ram[1] = noun;
			ram[2] = verb;

			execute(&mut ram);

			if ram[0] == 19690720 {
				println!("second -> {}", 100 * noun + verb);
				return Ok(())
			}
		}
	}
	
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parsing() {
		let input = String::from("1,0,0,3,99");
		let ram = parse(&input).unwrap();
		assert_eq!(*ram, [1,0,0,3,99]);
	}

	#[test]
	fn halting() {
		let mut ram = [99];
		execute(&mut ram);
	}

	#[test]
	fn example1() {
		let mut ram = [1, 0, 0, 0, 99];
		execute(&mut ram);
		assert_eq!(ram, [2, 0, 0, 0, 99]);
	}

	#[test]
	fn example2() {
		let mut ram = [2, 3, 0, 3, 99];
		execute(&mut ram);
		assert_eq!(ram, [2, 3, 0, 6, 99]);
	}

	#[test]
	fn example3() {
		let mut ram = [2, 4, 4, 5, 99, 0];
		execute(&mut ram);
		assert_eq!(ram, [2, 4, 4, 5, 99, 9801]);
	}

	#[test]
	fn example4() {
		let mut ram = [1, 1, 1, 4, 99, 5, 6, 0, 99];
		execute(&mut ram);
		assert_eq!(ram, [30, 1, 1, 4, 2, 5, 6, 0, 99]);
	}
}
