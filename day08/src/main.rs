use std::io::{self, Read};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T, E = Error> = std::result::Result<T, E>;

fn main() -> Result<()> {
	let mut input = String::new();
	io::stdin().read_to_string(&mut input)?;

	let w = 25;
	let h = 6;
	let data = input.trim().as_bytes();

	let (ones, twos) = solve_part1(&data, w * h).ok_or("failed to parse layers")?;
	println!("p1: {}", ones * twos);

	let buf = flatten(&data, w, h);
	println!("p2:");
	paint(&buf, w);
	
	Ok(())
}

fn solve_part1(data: &[u8], layer_size: usize) -> Option<(usize, usize)> {
	let (_, (c, _)) = data
		.chunks_exact(layer_size)
		.map(|c| (c, c.iter().fold(0, |acc, &x| if x == b'0' { acc + 1 } else { acc })))
		.enumerate()
		.min_by_key(|&x| (x.1).1)?;

	Some(c.iter().fold((0, 0), |acc, &x| {
		match x {
			b'1' => (acc.0 + 1, acc.1),
			b'2' => (acc.0, acc.1 + 1),
			_ => acc,
		}
	}))
}

fn flatten(data: &[u8], w: usize, h: usize) -> Box<[u8]> {
	let layer_size = w * h;

	let chunks = data.chunks_exact(layer_size);
	assert!(chunks.remainder().is_empty());

	let mut buf = vec![2; layer_size];
	for c in chunks.rev() {
		for i in 0..layer_size {
			if c[i] != b'2' {
				buf[i] = c[i];
			}
		}
	}

	buf.into_boxed_slice()
}

fn paint(data: &[u8], w: usize) {
	for row in data.chunks_exact(w) {
		for &x in row {
			print!("{}", if x == b'0' { ' ' } else { '█' });
		}
		println!();
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn example2() {
		let data = "0222112222120000".as_bytes();
		let f = flatten(&data, 2, 2);
		assert_eq!(&f[..], &[b'0',b'1',b'1',b'0']);
	}
}
