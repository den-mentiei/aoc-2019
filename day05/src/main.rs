use std::io::{self, Read};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T, E = Error> = std::result::Result<T, E>;

fn main() -> Result<()> {
	let mut input = String::new();
	io::stdin().read_to_string(&mut input)?;

	let p1 = run_sub(&input, 1)?;
	let p2 = run_sub(&input, 5)?;
	println!("p1: {:?}", p1);
	println!("p2: {:?}", p2);

	Ok(())
}

fn run_sub(program: &str, id: isize) -> Result<Vec<isize>> {
	let mut ram = parse(program)?;
	let input = [ id ];
	Ok(execute(&input, &mut ram))
}

fn parse(input: &str) -> Result<Box<[isize]>> {
	let ram: Vec<isize> = input
		.trim()
		.split(',')
		.map(|s| s.parse::<isize>())
		.collect::<Result<Vec<_>, _>>()?;

	Ok(ram.into_boxed_slice())
}

fn execute(mut input: &[isize], ram: &mut [isize]) -> Vec<isize> {
	let mut pc: usize = 0;

	let mut output = Vec::new();
	
	loop {
		let (op, len) = decode(&ram[pc..]);

		println!("{:?}", op);
		
		match op {
			Op::Add(a, b, Param::Pos(c)) => {
				let a = load_value(&ram, a);
				let b = load_value(&ram, b);
				ram[c] = a + b
			},
			Op::Mul(a, b, Param::Pos(c)) => {
				let a = load_value(&ram, a);
				let b = load_value(&ram, b);
				ram[c] = a * b
			},
			Op::In(Param::Pos(d)) => {
				let x = input[0];
				input = &input[1..];
				ram[d] = x;
				println!(";; read {}", x);
			},
			Op::Out(a) => {
				let a = load_value(&ram, a);
				output.push(a);
				println!(";; wrote {}", a);
			},
			Op::JmpTrue(a, b) => {
				let a = load_value(&ram, a);
				let b = load_value(&ram, b);
				if a != 0 {
					pc = b as usize;
					println!(";; jumped to {}", pc);
					continue;
				}
			},
			Op::JmpFalse(a, b) => {
				let a = load_value(&ram, a);
				let b = load_value(&ram, b);
				if a == 0 {
					pc = b as usize;
					println!(";; jumped to {}", pc);
					continue;
				}
			},
			Op::CmpLess(a, b, Param::Pos(c)) => {
				let a = load_value(&ram, a);
				let b = load_value(&ram, b);
				ram[c] = if a < b { 1 } else { 0 };
				println!(";; wrote {}", ram[c]);
			},
			Op::CmpEq(a, b, Param::Pos(c)) => {
				let a = load_value(&ram, a);
				let b = load_value(&ram, b);
				ram[c] = if a == b { 1 } else { 0 };
				println!(";; wrote {}", ram[c]);
			},
			Op::Halt => break,
			_ => panic!("unknown opcode"),
		}

		pc += len;
	}

	output
}

fn load_value(ram: &[isize], p: Param) -> isize {
	match p {
		Param::Pos(i) => ram[i],
		Param::Imm(x) => x,
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
enum Param {
	Pos(usize),
	Imm(isize),
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
enum Op {
	Add(Param, Param, Param),
	Mul(Param, Param, Param),
	In(Param),
	Out(Param),
	JmpTrue(Param, Param),
	JmpFalse(Param, Param),
	CmpLess(Param, Param, Param),
	CmpEq(Param, Param, Param),
	Halt,
}

fn decode(ram: &[isize]) -> (Op, usize) {
	let opcode = ram[0];
	let op = opcode % 100;
	match op {
		1 => {
			let (am, bm, cm) = decode_triple_modes(opcode);
			let (av, bv, cv) = (ram[1], ram[2], ram[3]);
			let a = decode_param(am, av);
			let b = decode_param(bm, bv);
			let c = decode_param(cm, cv);
			(Op::Add(a, b, c), 4)
		},
		2 => {
			let (am, bm, cm) = decode_triple_modes(opcode);
			let (av, bv, cv) = (ram[1], ram[2], ram[3]);
			let a = decode_param(am, av);
			let b = decode_param(bm, bv);
			let c = decode_param(cm, cv);
			(Op::Mul(a, b, c), 4)
		},
		3 => {
			let i = ram[1] as usize;
			(Op::In(Param::Pos(i)), 2)
		},
		4 => {
			let (am, _, _) = decode_triple_modes(opcode);
			let av = ram[1];
			let a = decode_param(am, av);
			(Op::Out(a), 2)
		},
		5 => {
			let (am, bm, _) = decode_triple_modes(opcode);
			let (av, bv) = (ram[1], ram[2]);
			let a = decode_param(am, av);
			let b = decode_param(bm, bv);
			(Op::JmpTrue(a, b), 3)
		},
		6 => {
			let (am, bm, _) = decode_triple_modes(opcode);
			let (av, bv) = (ram[1], ram[2]);
			let a = decode_param(am, av);
			let b = decode_param(bm, bv);
			(Op::JmpFalse(a, b), 3)
		},
		7 => {
			let (am, bm, cm) = decode_triple_modes(opcode);
			let (av, bv, cv) = (ram[1], ram[2], ram[3]);
			let a = decode_param(am, av);
			let b = decode_param(bm, bv);
			let c = decode_param(cm, cv);
			(Op::CmpLess(a, b, c), 4)
		},
		8 => {
			let (am, bm, cm) = decode_triple_modes(opcode);
			let (av, bv, cv) = (ram[1], ram[2], ram[3]);
			let a = decode_param(am, av);
			let b = decode_param(bm, bv);
			let c = decode_param(cm, cv);
			(Op::CmpEq(a, b, c), 4)
		},
		99 => (Op::Halt, 1),
		_ => panic!("unknown instruction"),
	}
}

// cbaop
fn decode_triple_modes(mut opcode: isize) -> (u8, u8, u8) {
	opcode /= 100;
	let a = (opcode % 10) as u8;
	opcode /= 10;
	let b = (opcode % 10) as u8;
	opcode /= 10;
	let c = (opcode % 10) as u8;

	(a, b, c)
}

fn decode_param(mode: u8, value: isize) -> Param {
	match mode {
		0 => Param::Pos(value as usize),
		1 => Param::Imm(value),
		_ => panic!("unknown parameter mode"),
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parsing() {
		let source = "1,0,0,3,99,-7";
		let ram = parse(&source).unwrap();
		assert_eq!(*ram, [1,0,0,3,99,-7]);
	}

	#[test]
	fn decoding() {
		let mut ram = parse(&"1002,4,3,4").unwrap();
		let i = decode(&mut ram);
		assert_eq!(i, (Op::Mul(Param::Pos(4), Param::Imm(3), Param::Pos(4)), 4));
	}

	#[test]
	fn input() {
		let mut ram = parse(&"3,0,99").unwrap();
		let input = [42];
		let _ = execute(&input, &mut ram);
		assert_eq!(ram[0], input[0]);
	}

	#[test]
	fn output() {
		let mut ram = parse(&"4,3,99,42").unwrap();
		let input = [];
		let output = execute(&input, &mut ram);
		assert_eq!(output[0], 42);
	}

	#[test]
	fn example1() {
		let mut ram = parse(&"3,0,4,0,99").unwrap();
		let input = [42];
		let output = execute(&input, &mut ram);
		assert_eq!(output[0], input[0]);
	}
	
	#[test]
	fn example2() {
		let mut ram = parse(&"1101,100,-1,4,0").unwrap();
		let input = [];
		let _ = execute(&input, &mut ram);
		assert_eq!(ram[4], 100 + -1);
	}

	#[test]
	fn example3_eq() {
		let mut ram = parse(&"3,9,8,9,10,9,4,9,99,-1,8").unwrap();
		let input = [8];
		let output = execute(&input, &mut ram);
		assert_eq!(output[0], 1);
	}

	#[test]
	fn example3_ne() {
		let mut ram = parse(&"3,9,8,9,10,9,4,9,99,-1,8").unwrap();
		let input = [9];
		let output = execute(&input, &mut ram);
		assert_eq!(output[0], 0);
	}

	#[test]
	fn example4_lt() {
		let mut ram = parse(&"3,9,7,9,10,9,4,9,99,-1,8").unwrap();
		let input = [7];
		let output = execute(&input, &mut ram);
		assert_eq!(output[0], 1);
	}

	#[test]
	fn example4_gt() {
		let mut ram = parse(&"3,9,7,9,10,9,4,9,99,-1,8").unwrap();
		let input = [9];
		let output = execute(&input, &mut ram);
		assert_eq!(output[0], 0);
	}

	#[test]
	fn example5_eq() {
		let mut ram = parse(&"3,3,1108,-1,8,3,4,3,99").unwrap();
		let input = [8];
		let output = execute(&input, &mut ram);
		assert_eq!(output[0], 1);
	}

	#[test]
	fn example5_ne() {
		let mut ram = parse(&"3,3,1108,-1,8,3,4,3,99").unwrap();
		let input = [9];
		let output = execute(&input, &mut ram);
		assert_eq!(output[0], 0);
	}

	#[test]
	fn example6_lt() {
		let mut ram = parse(&"3,3,1107,-1,8,3,4,3,99").unwrap();
		let input = [1];
		let output = execute(&input, &mut ram);
		assert_eq!(output[0], 1);
	}

	#[test]
	fn example6_eq() {
		let mut ram = parse(&"3,3,1107,-1,8,3,4,3,99").unwrap();
		let input = [8];
		let output = execute(&input, &mut ram);
		assert_eq!(output[0], 0);
	}

	#[test]
	fn example7_zero() {
		let mut ram = parse(&"3,12,6,12,15,1,13,14,13,4,13,99,-1,0,1,9").unwrap();
		let input = [0];
		let output = execute(&input, &mut ram);
		assert_eq!(output[0], 0);
	}

	#[test]
	fn example7_non_zero() {
		let mut ram = parse(&"3,12,6,12,15,1,13,14,13,4,13,99,-1,0,1,9").unwrap();
		let input = [42];
		let output = execute(&input, &mut ram);
		assert_eq!(output[0], 1);
	}

	#[test]
	fn example8_zero() {
		let mut ram = parse(&"3,3,1105,-1,9,1101,0,0,12,4,12,99,1").unwrap();
		let input = [0];
		let output = execute(&input, &mut ram);
		assert_eq!(output[0], 0);
	}

	#[test]
	fn example8_non_zero() {
		let mut ram = parse(&"3,3,1105,-1,9,1101,0,0,12,4,12,99,1").unwrap();
		let input = [42];
		let output = execute(&input, &mut ram);
		assert_eq!(output[0], 1);
	}

	#[test]
	fn example9_lt() {
		let mut ram = parse(&"3,21,1008,21,8,20,1005,20,22,107,8,21,20,1006,20,31,1106,0,36,98,0,0,1002,21,125,20,4,20,1105,1,46,104,999,1105,1,46,1101,1000,1,20,4,20,1105,1,46,98,99").unwrap();
		let input = [7];
		let output = execute(&input, &mut ram);
		assert_eq!(output[0], 999);
	}

	#[test]
	fn example9_eq() {
		let mut ram = parse(&"3,21,1008,21,8,20,1005,20,22,107,8,21,20,1006,20,31,1106,0,36,98,0,0,1002,21,125,20,4,20,1105,1,46,104,999,1105,1,46,1101,1000,1,20,4,20,1105,1,46,98,99").unwrap();
		let input = [8];
		let output = execute(&input, &mut ram);
		assert_eq!(output[0], 1000);
	}

	#[test]
	fn example9_gt() {
		let mut ram = parse(&"3,21,1008,21,8,20,1005,20,22,107,8,21,20,1006,20,31,1106,0,36,98,0,0,1002,21,125,20,4,20,1105,1,46,104,999,1105,1,46,1101,1000,1,20,4,20,1105,1,46,98,99").unwrap();
		let input = [9];
		let output = execute(&input, &mut ram);
		assert_eq!(output[0], 1001);
	}
}
