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

	let rom = parse(&input)?;
	
	println!("p1: {:?}", execute(&rom, 1)?);
	println!("p2: {:?}", execute(&rom, 2)?);
	
	Ok(())
}

fn execute(rom: &[isize], program: isize) -> Result<Vec<isize>> {
	let mut m = Machine::from(&rom);
	m.feed(program);
	let mut output = Vec::new();
	if m.run(&mut output) != State::Halted {
		Err("something went wrong")?;
	}
	Ok(output)
}

fn parse(input: &str) -> Result<Box<[isize]>> {
	let ram: Vec<isize> = input
		.trim()
		.split(',')
		.map(|s| s.parse::<isize>())
		.collect::<Result<Vec<_>, _>>()?;

	Ok(ram.into_boxed_slice())
}

struct Machine {
	ram: Vec<isize>,
	pc: usize,
	rb: usize,
	state: State,

	input: Vec<isize>,
	consumed: usize,
}

impl Machine {
	fn from(rom: &[isize]) -> Machine {
		Machine {
			ram: rom.to_vec(),
			pc: 0,
			rb: 0,
			state: State::Ready,
			input: Vec::new(),
			consumed: 0,
		}
	}

	fn feed(&mut self, x: isize) {
		self.input.push(x);
	}

	fn write_value(&mut self, p: Param, x: isize) {
		let i = match p {
			Param::Pos(i) => i,
			Param::Rel(i) => (self.rb as isize + i) as usize,
			Param::Imm(_) => panic!("incorrect destination"),
		};
		
		if i >= self.ram.len() {
			self.ram.resize(i + 1, 0);
		}

		self.ram[i] = x;
	}

	fn load(&mut self, i: usize) -> isize {
		if i >= self.ram.len() {
			self.ram.resize(i + 1, 0);
		}
		self.ram[i]
	}
	
	fn load_value(&mut self, p: Param) -> isize {
		match p {
			Param::Pos(i) => self.load(i),
			Param::Imm(x) => x,
			Param::Rel(i) => self.load((self.rb as isize + i) as usize),
		}
	}
	
	fn run(&mut self, output: &mut Vec<isize>) -> State {
		if self.state == State::Halted {
			return self.state;
		}

		loop {
			let (op, len) = decode(&self.ram[self.pc..]);

			trace!("{:?}", op);
			
			match op {
				Op::Add(a, b, c) => {
					let a = self.load_value(a);
					let b = self.load_value(b);
					self.write_value(c, a + b);
				},
				Op::Mul(a, b, c) => {
					let a = self.load_value(a);
					let b = self.load_value(b);
					self.write_value(c, a * b);
				},
				Op::In(a) => {
					if self.consumed < self.input.len() {
						let x = self.input[self.consumed];
						self.write_value(a, x);
						self.consumed += 1;
						trace!(";; read {}", x);
					} else {
						trace!(";; suspend due to input waiting");
						self.state = State::NeedsInput;
						break;
					}
				},
				Op::Out(a) => {
					let a = self.load_value(a);
					output.push(a);
					trace!(";; wrote {}", a);
				},
				Op::JmpTrue(a, b) => {
					let a = self.load_value(a);
					let b = self.load_value(b);
					if a != 0 {
						self.pc = b as usize;
						trace!(";; jumped to {}", self.pc);
						continue;
					}
				},
				Op::JmpFalse(a, b) => {
					let a = self.load_value(a);
					let b = self.load_value(b);
					if a == 0 {
						self.pc = b as usize;
						trace!(";; jumped to {}", self.pc);
						continue;
					}
				},
				Op::CmpLess(a, b, c) => {
					let a = self.load_value(a);
					let b = self.load_value(b);
					let x = if a < b { 1 } else { 0 };
					self.write_value(c, x);
					trace!(";; wrote {}", x);
				},
				Op::CmpEq(a, b, c) => {
					let a = self.load_value(a);
					let b = self.load_value(b);
					let x = if a == b { 1 } else { 0 };
					self.write_value(c, x);
					trace!(";; wrote {}", x);
				},
				Op::AdjustBase(a) => {
					self.rb = ((self.rb as isize) + self.load_value(a)) as usize;
					trace!(";; adjusted base {}", self.rb);
				},
				Op::Halt => {
					self.state = State::Halted;
					break;
				},
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

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
enum Param {
	Pos(usize),
	Imm(isize),
	Rel(isize),
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
	AdjustBase(Param),
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
			let (am, _, _) = decode_triple_modes(opcode);
			let av = ram[1];
			let a = decode_param(am, av);
			(Op::In(a), 2)
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
		9 => {
			let (am, _, _) = decode_triple_modes(opcode);
			let av = ram[1];
			let a = decode_param(am, av);
			(Op::AdjustBase(a), 2)
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
		2 => Param::Rel(value),
		_ => panic!("unknown parameter mode"),
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn example1() {
		let output = run(&"109,1,204,-1,1001,100,1,100,1008,100,16,101,1006,101,0,99");
		assert_eq!(&output[..], &[109,1,204,-1,1001,100,1,100,1008,100,16,101,1006,101,0,99]);
	}

	#[test]
	fn example2() {
		let output = run(&"1102,34915192,34915192,7,4,7,99,0");
		assert_eq!(output[0], 1219070632396864);
	}

	#[test]
	fn example3() {
		let output = run(&"104,1125899906842624,99");
		assert_eq!(output[0], 1125899906842624);
	}

	fn run(code: &str) -> Vec<isize> {
		let rom = parse(code).unwrap();
		let mut m = Machine::from(&rom);
		let mut output = Vec::<isize>::new();
		assert_eq!(m.run(&mut output), State::Halted);
		output
	}
}
