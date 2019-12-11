use std::io::{self, Read};
use std::collections::HashMap;

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

	let (_, p1) = run(&rom, 0);
	println!("p1: {}", p1);

	let (map, _) = run(&rom, 1);
	println!("p1:");
	dump(&map);
	
	Ok(())
}

fn run(rom: &[isize], start: isize) -> (HashMap::<(i64, i64), u8>, i64) {
	let mut painted = 0;
	let mut map = HashMap::<(i64, i64), u8>::new();
	let mut pos = (0, 0);
	let mut dir = Dir::N;
	
	let mut scratch = Vec::new();
	let mut m = Machine::from(&rom);
	m.feed(start);
	loop {
		if m.run(&mut scratch) == State::Halted {
			break;
		}
		let color = scratch[0];
		let rotation = scratch[1];
		scratch.clear();

		*map.entry(pos).or_insert_with(|| {
			painted += 1;
			color as u8
		}) = color as u8;
		
		dir = rotate_robot(dir, rotation);
		pos = move_robot(pos, dir);

		let cam = *map.get(&pos).unwrap_or(&0);
		m.feed(cam as isize);
	}

	(map, painted)
}

fn dump(map: &HashMap<(i64, i64), u8>) {
	let mut xmin = std::i64::MAX;
	let mut xmax = std::i64::MIN;
	let mut ymin = std::i64::MAX;
	let mut ymax = std::i64::MIN;
	for (x, y) in map.keys() {
		xmin = xmin.min(*x);
		xmax = xmax.max(*x);
		ymin = ymin.min(*y);
		ymax = ymax.max(*y);
	}
	
	let w = (xmax - xmin + 1) as usize;
	let h = (ymax - ymin + 1) as usize;
	let mut rows = Vec::new();
	for _ in 0..h {
		let mut row = Vec::<u8>::new();
		row.resize(w, 0);
		rows.push(row);
	}
	for ((x, y), c) in map.iter() {
		let x = x - xmin;
		let y = y - ymin;
		rows[y as usize][x as usize] = *c;
	}

	for y in 0..h {
		for x in 0..w {
			print!("{}", if rows[y][x] == 1 { '█' } else { ' ' });
		}
		println!();
	}
}

fn rotate_robot(dir: Dir, r: isize) -> Dir {
	match (dir, r) {
		(Dir::N, 0) => Dir::W,
		(Dir::N, 1) => Dir::E,
		(Dir::S, 0) => Dir::E,
		(Dir::S, 1) => Dir::W,
		(Dir::W, 0) => Dir::S,
		(Dir::W, 1) => Dir::N,
		(Dir::E, 0) => Dir::N,
		(Dir::E, 1) => Dir::S,
		_ => panic!("unsupported rotation command"),
	}
}

fn move_robot(pos: (i64, i64), dir: Dir) -> (i64, i64) {
	let (dx, dy) = match dir {
		Dir::N => ( 0, -1),
		Dir::S => ( 0,  1),
		Dir::W => (-1,  0),
		Dir::E => ( 1,  0),
	};

	(pos.0 + dx, pos.1 + dy)
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy)]
enum Dir {
	N,
	S,
	W,
	E,
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
