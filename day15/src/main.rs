use std::io::{self, Read};
use std::collections::VecDeque;

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
	let mut drone = Drone::new(&rom);
	drone.discover();
	dump(&drone.map, W, H);

	let sys = drone.sys.ok_or("no oxygen system point")?;
	let steps = flood_fill(&drone.map, W, H, sys);
	let p1 = steps[to_offset((0, 0), W, H)].ok_or("failed to find the path")?;
	println!("p1: {}", p1);

	let max = steps.iter().map(|s| s.unwrap_or(0)).max().ok_or("failed to find the path")?;
	println!("p2: {}", max);
	
	Ok(())
}

fn flood_fill(map: &[Cell], w: usize, h: usize, start: Pos) -> Box<[Option<usize>]> {
	let mut q = VecDeque::new();
	q.push_back((1, start));

	let mut steps = vec![None; w * h];
	steps[to_offset(start, w, h)] = Some(1);
	
	while !q.is_empty() {
		let (s, p) = q.pop_front().unwrap();
		for &dir in Dir::ALL.iter() {
			let n = dir.apply(p);
			let c = map[to_offset(n, w, h)];
			if steps[to_offset(n, w, h)].is_some() {
				continue;
			}
			if c == Cell::Wall {
				continue;
			}
			steps[to_offset(n, w, h)] = Some(s);
			q.push_back((s + 1, n));
		}
	}

	steps.into_boxed_slice()
}

fn dump(map: &[Cell], w: usize, h: usize) {
	for y in 0..h {
		for x in 0..w {
			let c = match map[y * w + x] {
				Cell::Fog => '▒',
				Cell::Empty => ' ',
				Cell::Wall => '█',
				Cell::System => 'x',
			};
			if (y * w + x) == to_offset((0, 0), w, h) {
				print!("D");
			} else {
				print!("{}", c);
			}
		}
		println!();
	}
}

const W: usize = 45;
const H: usize = 45;

struct Drone {
	machine: Machine,
	output: Vec<isize>,
	map: [Cell; W * H],
	pos: Pos,
	sys: Option<Pos>,
}

impl Drone {
	fn new(rom: &[isize]) -> Drone {
		Drone {
			machine: Machine::from(rom),
			output: Vec::new(),
			map: [Cell::Fog; W * H],
			pos: (0, 0),
			sys: None,
		}
	}

	fn discover(&mut self) {
		for &dir in Dir::ALL.iter() {
			let p = dir.apply(self.pos);
			let offset = to_offset(p, W, H);
			if self.map[offset] == Cell::Fog {
				let cell = self.step(dir);
				self.map[offset] = cell;
				match cell {
					Cell::Wall => (),
					Cell::Empty | Cell::System => {
						if cell == Cell::System {
							self.sys = Some(p);
						}
						self.pos = p;
						self.discover();
						let back = dir.rev();
						self.step(back);
						self.pos = back.apply(self.pos);
					},
					Cell::Fog => unreachable!(),
				}
			}
		}
	}

	fn step(&mut self, dir: Dir) -> Cell {
		self.output.clear();
		self.machine.feed(match dir {
			Dir::N => 1,
			Dir::S => 2,
			Dir::W => 3,
			Dir::E => 4,
		});
		self.machine.run(&mut self.output);
		match self.output[0] {
			0 => Cell::Wall,
			1 => Cell::Empty,
			2 => Cell::System,
			_ => panic!("unknown cell type"),
		}
	}
}

type Pos = (i64, i64);

fn to_offset(p: Pos, w: usize, h: usize) -> usize {
	let x = offset(p.0, w);
	let y = offset(p.1, h);
	assert!(x < w && y < h);
	return y * w + x;
	
	fn offset(v: i64, dim: usize) -> usize {
		(v + (dim as i64) / 2) as usize
	}
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Debug)]
enum Dir {
	N,
	S,
	W,
	E
}

impl Dir {
	pub const ALL: [Dir; 4] = [
		Dir::N,
		Dir::S,
		Dir::W,
		Dir::E,
	];

	fn apply(self, p: Pos) -> Pos {
		match self {
			Dir::N => (p.0 + 0, p.1 - 1),
			Dir::S => (p.0 + 0, p.1 + 1),
			Dir::W => (p.0 - 1, p.1 + 0),
			Dir::E => (p.0 + 1, p.1 + 0),
		}
	}

	fn rev(self) -> Dir {
		match self {
			Dir::N => Dir::S,
			Dir::S => Dir::N,
			Dir::W => Dir::E,
			Dir::E => Dir::W,
		}
	}
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Debug)]
enum Cell {
	Fog,
	Empty,
	Wall,
	System
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
