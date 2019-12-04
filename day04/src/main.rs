type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T, E = Error> = std::result::Result<T, E>;

fn main() -> Result<()> {
	let input = "146810-612564";

	let mut parts = input.split('-');
	let lo: u32 = parts.next().ok_or("failed to parse from")?.parse()?;
	let hi: u32 = parts.next().ok_or("failed to parse to")?.parse()?;

	let (p1, p2) = solve(lo, hi);
	println!("p1 = {}", p1);
	println!("p2 = {}", p2);

	Ok(())
}

const SIZE: usize = 6;
type Parts = [u8; SIZE];

fn solve(lo: u32, hi: u32) -> (u32, u32) {
	let mut p1 = 0;
	let mut p2 = 0;

	for x in lo..=hi {
		let parts = split(x);
		if verify1(&parts) {
			p1 += 1;
		}
		if verify2(&parts) {
			p2 += 1;
		}
	}

	(p1, p2)
}

fn split(mut x: u32) -> Parts {
	let mut res: Parts = [0; SIZE];

	let mut i = SIZE;
	while x > 0 {
		i -= 1;
		res[i] = (x % 10) as u8;
		x /= 10;
	}
	
	return res;
}

fn verify1(x: &Parts) -> bool {
	verify_adjacent(x) && verify_increasing(x)
}

// 012345
// aa____
// _aa___
// __aa__
// ___aa_
// ____aa
fn verify_adjacent(x: &Parts) -> bool {
	for i in 0..=4 {
		if x[i] == x[i + 1] {
			return true;
		}
	}

	false
}

fn verify_increasing(x: &Parts) -> bool {
	for i in 1..SIZE {
		if x[i] < x[i - 1] {
			return false;
		}
	}

	true
}

fn verify2(x: &Parts) -> bool {
	verify_adjacent_not_larger(x) && verify_increasing(x)
}

// 012345
// aabbbb
// baabbb
// bbaabb
// bbbaab
// bbbbaa
fn verify_adjacent_not_larger(x: &Parts) -> bool {
	for i in 0..=4 {
		if x[i] == x[i + 1] && (i == 0 || x[i] != x[i - 1]) && (i == 4 || x[i] != x[i + 2]) {
			return true;
		}
	}

	false
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn examples1() {
		assert_eq!(verify1(&split(111111)), true);
		assert_eq!(verify1(&split(223450)), false);
		assert_eq!(verify1(&split(123789)), false);
	}

	#[test]
	fn examples2() {
		assert_eq!(verify2(&split(112233)), true);
		assert_eq!(verify2(&split(123444)), false);
		assert_eq!(verify2(&split(111122)), true);
	}
}
