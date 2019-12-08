use std::io::{self, Read};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T, E = Error> = std::result::Result<T, E>;

const TRACE: bool = false;

macro_rules! trace {
	($($arg:tt)+) => {
		if TRACE {
			println!($($arg)+);
		}
	}
}

fn main() -> Result<()> {
	let mut input = String::new();
	io::stdin().read_to_string(&mut input)?;

	let ram = parse(&input)?;
	
	let mut p1 = std::isize::MIN;
	let mut phases = [0,1,2,3,4];
	while permute(&mut phases) {
		p1 = p1.max(evaluate(&phases, &ram));
	}

	let mut p2 = std::isize::MIN;
	let mut phases = [5,6,7,8,9];
	while permute(&mut phases) {
		p2 = p2.max(evaluate(&phases, &ram));
	}
	
	println!("p1: {}", p1);
	println!("p2: {}", p2);
	
	Ok(())
}

fn parse(input: &str) -> Result<Box<[isize]>> {
	let ram: Vec<isize> = input
		.trim()
		.split(',')
		.map(|s| s.parse::<isize>())
		.collect::<Result<Vec<_>, _>>()?;

	Ok(ram.into_boxed_slice())
}

fn permute(data: &mut [u8]) -> bool {
	// Finding the longest non-increasing suffix.
	let mut i = data.len() - 1;
	while i > 0 && data[i - 1] >= data[i] {
		i -= 1;
	}
	
	if i <= 0 {
		return false;
	}

	// data[i - 1] is the pivot, so finding the rightmost
	// element, which exceeds it.
	let mut j = data.len() - 1;
	while data[j] <= data[i - 1] {
		j -= 1;
	}

	// data[j] will be the new pivot.
	debug_assert!(j >= i, "wrong pivot");

	data.swap(i - 1, j);

	// Reversing the suffix.
	j = data.len() - 1;
	while i < j {
		data.swap(i, j);
		i += 1;
		j -= 1;
	}
	
	true
}

fn evaluate(phases: &[u8], code: &[isize]) -> isize {
	let mut machines: Vec<Machine> = phases.iter().map(|_| Machine::from(&code)).collect();

	for (phase, machine) in phases.iter().zip(machines.iter_mut()) {
		machine.feed(*phase as isize);
	}

	let mut output = Vec::new();
	let mut signal = 0;

	loop {
		let mut n = 0;
		for m in machines.iter_mut() {
			if m.is_halted() {
				continue;
			}

			n += 1;

			m.feed(signal);
			
			output.clear();
			let _ = m.run(&mut output);

			if let Some(s) = output.last() {
				signal = *s;
			}
		}

		if n == 0 {
			break;
		}
	}
	
	signal
}

struct Machine {
	ram: Box<[isize]>,
	pc: usize,
	state: State,

	input: Vec<isize>,
	consumed: usize,
}

impl Machine {
	fn from(rom: &[isize]) -> Machine {
		let ram = rom.to_vec().into_boxed_slice();
		Machine {
			ram: ram,
			pc: 0,
			state: State::Ready,
			input: Vec::new(),
			consumed: 0,
		}
	}

	fn feed(&mut self, x: isize) {
		self.input.push(x);
	}

	fn is_halted(&self) -> bool {
		self.state == State::Halted
	}
	
	fn run(&mut self, output: &mut Vec<isize>) -> State {
		if self.state == State::Halted {
			return self.state;
		}

		loop {
			let (op, len) = decode(&self.ram[self.pc..]);

			trace!("{:?}", op);
			
			match op {
				Op::Add(a, b, Param::Pos(c)) => {
					let a = load_value(&self.ram, a);
					let b = load_value(&self.ram, b);
					self.ram[c] = a + b
				},
				Op::Mul(a, b, Param::Pos(c)) => {
					let a = load_value(&self.ram, a);
					let b = load_value(&self.ram, b);
					self.ram[c] = a * b
				},
				Op::In(Param::Pos(d)) => {
					if self.consumed < self.input.len() {
						let x = self.input[self.consumed];
						self.ram[d] = x;
						self.consumed += 1;
						trace!(";; read {}", x);
					} else {
						trace!(";; suspend due to input waiting");
						self.state = State::NeedsInput;
						break;
					}
				},
				Op::Out(a) => {
					let a = load_value(&self.ram, a);
					output.push(a);
					trace!(";; wrote {}", a);
				},
				Op::JmpTrue(a, b) => {
					let a = load_value(&self.ram, a);
					let b = load_value(&self.ram, b);
					if a != 0 {
						self.pc = b as usize;
						trace!(";; jumped to {}", self.pc);
						continue;
					}
				},
				Op::JmpFalse(a, b) => {
					let a = load_value(&self.ram, a);
					let b = load_value(&self.ram, b);
					if a == 0 {
						self.pc = b as usize;
						trace!(";; jumped to {}", self.pc);
						continue;
					}
				},
				Op::CmpLess(a, b, Param::Pos(c)) => {
					let a = load_value(&self.ram, a);
					let b = load_value(&self.ram, b);
					self.ram[c] = if a < b { 1 } else { 0 };
					trace!(";; wrote {}", self.ram[c]);
				},
				Op::CmpEq(a, b, Param::Pos(c)) => {
					let a = load_value(&self.ram, a);
					let b = load_value(&self.ram, b);
					self.ram[c] = if a == b { 1 } else { 0 };
					trace!(";; wrote {}", self.ram[c]);
				},
				Op::Halt => {
					self.state = State::Halted;
					break;
				},
				_ => panic!("unknown opcode"),
			}

			self.pc += len;
		}

		self.state
	}
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Debug)]
enum State {
	Ready = 0,
	NeedsInput,
	Halted,
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
	fn permutations() {
		let mut data = [0, 1, 2];
		assert_eq!(permute(&mut data), true);
		assert_eq!(data, [0, 2, 1]);
		assert_eq!(permute(&mut data), true);
		assert_eq!(data, [1, 0, 2]);
		assert_eq!(permute(&mut data), true);
		assert_eq!(data, [1, 2, 0]);
		assert_eq!(permute(&mut data), true);
		assert_eq!(data, [2, 0, 1]);
		assert_eq!(permute(&mut data), true);
		assert_eq!(data, [2, 1, 0]);
		assert_eq!(permute(&mut data), false);
		assert_eq!(data, [2, 1, 0]);
	}
	
	#[test]
	fn examples1() {
		assert_eq!(run(&[4,3,2,1,0], "3,15,3,16,1002,16,10,16,1,16,15,15,4,15,99,0,0"), 43210);
	}

	#[test]
	fn examples2() {
		assert_eq!(run(&[0,1,2,3,4], "3,23,3,24,1002,24,10,24,1002,23,-1,23,101,5,23,23,1,24,23,23,4,23,99,0,0"), 54321);
	}

	#[test]
	fn examples3() {
		assert_eq!(run(&[1,0,4,3,2], "3,31,3,32,1002,32,10,32,1001,31,-2,31,1007,31,0,33,1002,33,7,33,1,33,31,31,1,32,31,31,4,31,99,0,0,0"), 65210);
	}

	fn run(phases: &[u8], code: &str) -> isize {
		let rom = parse(code).unwrap();
		evaluate(phases, &rom)
	}

	#[test]
	fn examples4() {
		assert_eq!(run(&[9,8,7,6,5], "3,26,1001,26,-4,26,3,27,1002,27,2,27,1,27,26,27,4,27,1001,28,-1,28,1005,28,6,99,0,0,5"), 139629729);
	}
	
	#[test]
	fn examples5() {
		assert_eq!(run(&[9,7,8,5,6], "3,52,1001,52,-5,52,3,53,1,52,56,54,1007,54,5,55,1005,55,26,1001,54,-5,54,1105,1,12,1,53,54,53,1008,54,0,55,1001,55,1,55,2,53,55,53,4,53,1001,56,-1,56,1005,56,6,99,0,0,0,0,10"), 18216);
	}

	fn run(phases: &[u8], code: &str) -> isize {
		let rom = parse(code).unwrap();
		evaluate(phases, &rom)
	}
}
