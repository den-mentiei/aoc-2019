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

	let mut rom = parse(&input)?;

	let drone = Drone::from(&rom);
	let p1 = solve_part1(&drone);
	println!("p1: {}", p1);

	let path = compress(&drone.trace_path());
	println!("path: {:?}", path);

	// TODO: Solved by hand :<
	// A R8L4R4R10R8
	// A R8L4R4R10R8
	// C L12L12R8R8
	// B R10R4R4
	// C L12L12R8R8
	// B R10R4R4
	// C L12L12R8R8
	// B R10R4R4
	// B R10R4R4
	// A R8L4R4R10R8
	
	rom[0] = 2;
	let mut ctrl = Control::from(&rom);
	ctrl.run(State::NeedsInput)?;

	ctrl.feed_routine(&"AACBCBCBBA");
	ctrl.run(State::NeedsInput)?;

	ctrl.feed_function(&[Cmd::R(8), Cmd::L(4), Cmd::R(4), Cmd::R(10), Cmd::R(8)]);
	ctrl.run(State::NeedsInput)?;

	ctrl.feed_function(&[Cmd::R(10), Cmd::R(4), Cmd::R(4)]);
	ctrl.run(State::NeedsInput)?;

	ctrl.feed_function(&[Cmd::L(12), Cmd::L(12), Cmd::R(8), Cmd::R(8)]);
	ctrl.run(State::NeedsInput)?;

	ctrl.feed_cmd(b'n');
	ctrl.run(State::Halted)?;

	Ok(())
}

struct Control {
	machine: Machine,
	output: Vec<isize>,
}

impl Control {
	fn from(rom: &[isize]) -> Control {
		Control { machine: Machine::from(rom), output: Vec::new() }
	}

	fn run(&mut self, expected: State) -> Result<()> {
		self.output.clear();
		let s = self.machine.run(&mut self.output);
		self.dump();
		if s == expected {
			Ok(())
		} else {
			Err("ended in unexpected state")?
		}
	}
	
	fn dump(&self) {
		for x in self.output.iter() {
			if *x <= std::u8::MAX as isize {
				let c = (*x as u8) as char;
				print!("{}", c);
			} else {
				println!("{}", x);
			}	
		}
		// let mut rows = Vec::new();
		// rows.push(Vec::new());
		// let mut r = 0;
		// for i in 0..output.len() {
		// 	let b = output[i] as u8;
		// 	if b == 10 {
		// 		rows.push(Vec::new());
		// 		r += 1;
		// 	} else {
		// 		rows[r].push(b);
		// 	}
		// }
		// for y in 0..rows.len() {
		// 	for x in 0..rows[y].len() {
		// 		print!("{}", rows[y][x] as char);
		// 	}
		// 	println!();
		// }
	}

	fn feed_cmd(&mut self, c: u8) {
		self.machine.feed(c as isize);
		self.machine.feed(10);
	}
	
	fn feed_routine(&mut self, s: &str) {
		let bytes = s.as_bytes();
		for i in 0..bytes.len() {
			let b = bytes[i] as isize;
			self.machine.feed(b);
			if i != bytes.len() - 1 {
				self.machine.feed(b',' as isize);
			}
		}
		self.machine.feed(10);
	}

	fn feed_function(&mut self, cmds: &[Cmd]) {
		for (i, c) in cmds.iter().enumerate() {
			match c {
				Cmd::L(n) => {
					self.machine.feed(b'L' as isize);
					self.machine.feed(b',' as isize);
					self.feed_number(*n);
				},
				Cmd::R(n) => {
					self.machine.feed(b'R' as isize);
					self.machine.feed(b',' as isize);
					self.feed_number(*n);
				},
			}
			if i != cmds.len() - 1 {
				self.machine.feed(b',' as isize);
			}
		}
		self.machine.feed(10);
	}

	fn feed_number(&mut self, mut x: u8) {
		let d100 = (x / 100) as u8;
		x %= 100;
		let d10 = (x / 10) as u8;
		x %= 10;
		let d1 = x as u8;

		if d100 != 0 {
			self.machine.feed((b'0' + d100) as isize);
		}
		if (d100 != 0 && d10 == 0) || d10 != 0 {
			self.machine.feed((b'0' + d10) as isize);
		}
		self.machine.feed((b'0' + d1) as isize);
	}

}

struct Drone {
	pos: Pos,
	dir: Dir,
	map: Vec<Vec<u8>>,
}

impl Drone {
	fn from(rom: &[isize]) -> Drone {
		let mut m = Machine::from(&rom);
		let mut output = Vec::new();
		let _ = m.run(&mut output);

		let mut pos = (0, 0);
		let mut dir = Dir::N;
		let mut rows = Vec::new();
		rows.push(Vec::new());
		let mut r = 0;
		for i in 0..output.len() {
			let b = output[i] as u8;
			if b == 10 {
				rows.push(Vec::new());
				r += 1;
			} else {
				rows[r].push(b);
			}
			match b {
				b'v' => {
					let x = (rows[r].len() - 1) as i64;
					let y = r as i64;
					pos = (x, y);
					dir = Dir::S;
				},
				b'^' => {
					let x = (rows[r].len() - 1) as i64;
					let y = r as i64;
					pos = (x, y);
					dir = Dir::N;
				},
				b'<' => {
					let x = (rows[r].len() - 1) as i64;
					let y = r as i64;
					pos = (x, y);
					dir = Dir::W;
				},
				b'>' => {
					let x = (rows[r].len() - 1) as i64;
					let y = r as i64;
					pos = (x, y);
					dir = Dir::E;
				},
				_ => (),
			};
		}

		Drone { pos: pos, dir: dir, map: rows }
	}

	fn trace_path(&self) -> Vec<u8> {
		let rows = &self.map;
		let mut pos = self.pos;
		let mut dir = self.dir;
		let mut steps = Vec::new();
		let mut prev;
		loop {
			prev = pos;
			let mut next = dir.apply(pos);
			let cell = look(&rows, next);
			if cell == Some(b'.') || cell.is_none() {
				let prev_dir = dir;
				dir = dir.turn_left();
				next = dir.apply(pos);
				let cell = look(&rows, next);
				if let Some(b'#') = cell {
					if next != prev {
						steps.push(b'L');
						steps.push(b'1');
						pos = next;
					}
					continue;
				}
				dir = prev_dir.turn_right();
				next = dir.apply(pos);
				let cell = look(&rows, next);
				if let Some(b'#') = cell {
					if next != prev {
						steps.push(b'R');
						steps.push(b'1');
						pos = next;
					}
					continue;
				}

				break;
			} else {
				steps.push(b'1');
				pos = next;
			}
		}

		steps
	}
}

fn solve_part1(drone: &Drone) -> usize {
	let mut sum = 0;

	let rows = &drone.map;
	
	for y in 1..rows.len() - 1 {
		if rows[y + 1].len() == 0 {
			break;
		}
		for x in 1..rows[y].len() - 1 {
			if rows[y][x] == b'#' &&
			   rows[y - 1][x] == b'#' &&
			   rows[y + 1][x] == b'#' &&
			   rows[y][x - 1] == b'#' &&
			   rows[y][x + 1] == b'#'
			{
				sum += y * x;
			}
		}
	}

	sum
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Debug)]
enum Cmd {
	L(u8),
	R(u8),
}

fn compress(data: &[u8]) -> Box<[Cmd]> {
	let mut v = Vec::new();

	let mut acc = None;
	for x in data {
		match x {
			b'L' => {
				if let Some(c) = acc {
					v.push(c);
				}
				acc = Some(Cmd::L(0))
			},
			b'R' => {
				if let Some(c) = acc {
					v.push(c);
				}
				acc = Some(Cmd::R(0))
			},
			b'1' => {
				acc = acc.map(|c| match c {
					Cmd::L(n) => Cmd::L(n + 1),
					Cmd::R(n) => Cmd::R(n + 1),
				});
			},
			_ => panic!("unknown command"),
		}
	}
	if let Some(c) = acc {
		v.push(c);
	}
	
	v.into_boxed_slice()
}

fn look(map: &Vec<Vec<u8>>, pos: Pos) -> Option<u8> {
	if pos.1 < 0 || pos.1 >= map.len() as i64 {
		return None;
	}
	let row = &map[pos.1 as usize];
	if pos.0 < 0 || pos.0 >= row.len() as i64 {
		return None;
	}

	Some(row[pos.0 as usize])
}

type Pos = (i64, i64);

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Debug)]
enum Dir {
	N,
	S,
	W,
	E,
}

impl Dir {
	fn apply(self, p: Pos) -> Pos {
		match self {
			Dir::N => (p.0 + 0, p.1 - 1),
			Dir::S => (p.0 + 0, p.1 + 1),
			Dir::W => (p.0 - 1, p.1 + 0),
			Dir::E => (p.0 + 1, p.1 + 0),
		}
	}

	fn turn_left(self) -> Dir {
		match self {
			Dir::N => Dir::W,
			Dir::S => Dir::E,
			Dir::W => Dir::S,
			Dir::E => Dir::N,
		}
	}

	fn turn_right(self) -> Dir {
		self.turn_left().rev()
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

		for i in &self.input[self.consumed..] {
			print!("{}", (*i as u8) as char);
		}
		println!("");

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
