use std::io::{self, Read};
use std::collections::HashMap;
use std::collections::hash_map::Entry::Occupied;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T, E = Error> = std::result::Result<T, E>;

type Variable<'a> = (&'a str, i64);

fn main() -> Result<()> {
	let mut input = String::new();
	io::stdin().read_to_string(&mut input)?;

	let equations = parse(&input)?;
	let p1 = calc_ore(&equations, "FUEL", 1, &mut HashMap::new()).ok_or("failed to calculate required ore")?;
	println!("p1: {}", p1);

	let p2 = calc_fuel(&equations, 1000000000000).ok_or("failed to calculate available fuel")?;
	println!("p2: {}", p2);
	
	Ok(())
}

fn calc_fuel(
	eqs: &HashMap<&str, (i64, Vec<Variable>)>,
	ore: i64) -> Option<i64>
{
	let mut max = 1;
	loop {
		if calc_ore(&eqs, "FUEL", max, &mut HashMap::new())? > ore {
			break;
		} else {
			max *= 10;
		}
	}

	let mut min = max / 10;

	loop {
		if min == max {
			break;
		}

		let x = (min + max) / 2;
		let new_ore = calc_ore(&eqs, "FUEL", x, &mut HashMap::new())?;

		if new_ore < ore {
			min = x + 1;
		} else if new_ore > ore {
			max = x - 1;
		} else {
			return Some(x);
		}
	}
	
	Some(min)
}

fn calc_ore<'a>(
	eqs: &HashMap<&'a str, (i64, Vec<Variable<'a>>)>,
	target: &'a str,
	mut need: i64,
	stash: &mut HashMap<&'a str, i64>) -> Option<i64>
{
	if target == "ORE" {
		return Some(need);
	}
	
	if let Occupied(mut e) = stash.entry(target) {
		let e = e.get_mut();
		let ready = *e;
		if ready >= need {
			*e -= need;
			return Some(0);
		} else {
			*e -= ready;
			need -= ready;
		}
	}

	let (produced, inputs) = eqs.get(target)?;

	let times = need / produced + if need % produced != 0 { 1 } else { 0 };
	let produced = produced * times;
	let extra = produced - need;

	*stash.entry(target).or_insert(0) += extra;

	let mut ore = 0;
	for i in inputs {
		ore += calc_ore(eqs, i.0, i.1 * times, stash)?;
	}

	Some(ore)
}

fn parse<'a>(input: &'a str) -> Result<HashMap<&'a str, (i64, Vec<Variable<'a>>)>> {
	let mut names = HashMap::new();
	
	for l in input.trim().lines() {
		let mut parts = l.split("=>");
		let left = parts.next().ok_or("failed to parse equation")?.trim();
		let right = parts.next().ok_or("failed to parse equation")?.trim();
		let result = parse_pair(&right)?;
		let mut inputs = Vec::new();
		for p in left.split(',').map(|p| p.trim()) {
			inputs.push(parse_pair(&p)?);
		}
		let prev = names.insert(result.0, (result.1, inputs));
		assert!(prev.is_none(), "multiple ways to produce same chemical");
	}

	Ok(names)
}

fn parse_pair(input: &str) -> Result<(&str, i64)> {
	let mut parts = input.split(' ');
	let num = parts.next().ok_or("failed to parse equation input")?.parse::<i64>()?;
	let tag = parts.next().ok_or("failed to parse equation input")?;
	Ok((tag, num))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn example1() {
		let input = r"
			10 ORE => 10 A
			1 ORE => 1 B
			7 A, 1 B => 1 C
			7 A, 1 C => 1 D
			7 A, 1 D => 1 E
			7 A, 1 E => 1 FUEL
		";
		let eqs = parse(&input).expect("parsing failed");
		assert_eq!(calc_ore(&eqs, "FUEL", 1, &mut HashMap::new()).expect("calc failed"), 31);
	}

	#[test]
	fn example2() {
		let input = r"
			9 ORE => 2 A
			8 ORE => 3 B
			7 ORE => 5 C
			3 A, 4 B => 1 AB
			5 B, 7 C => 1 BC
			4 C, 1 A => 1 CA
			2 AB, 3 BC, 4 CA => 1 FUEL
		";
		let eqs = parse(&input).expect("parsing failed");
		assert_eq!(calc_ore(&eqs, "FUEL", 1, &mut HashMap::new()).expect("calc failed"), 165);
	}

	#[test]
	fn example3() {
		let input = r"
			157 ORE => 5 NZVS
			165 ORE => 6 DCFZ
			44 XJWVT, 5 KHKGT, 1 QDVJ, 29 NZVS, 9 GPVTF, 48 HKGWZ => 1 FUEL
			12 HKGWZ, 1 GPVTF, 8 PSHF => 9 QDVJ
			179 ORE => 7 PSHF
			177 ORE => 5 HKGWZ
			7 DCFZ, 7 PSHF => 2 XJWVT
			165 ORE => 2 GPVTF
			3 DCFZ, 7 NZVS, 5 HKGWZ, 10 PSHF => 8 KHKGT
		";
		let eqs = parse(&input).expect("parsing failed");
		assert_eq!(calc_ore(&eqs, "FUEL", 1, &mut HashMap::new()).expect("calc failed"), 13312);
		assert_eq!(calc_fuel(&eqs, 1000000000000).expect("calc failed"), 82892753);
	}

	#[test]
	fn example4() {
		let input = r"
			2 VPVL, 7 FWMGM, 2 CXFTF, 11 MNCFX => 1 STKFG
			17 NVRVD, 3 JNWZP => 8 VPVL
			53 STKFG, 6 MNCFX, 46 VJHF, 81 HVMC, 68 CXFTF, 25 GNMV => 1 FUEL
			22 VJHF, 37 MNCFX => 5 FWMGM
			139 ORE => 4 NVRVD
			144 ORE => 7 JNWZP
			5 MNCFX, 7 RFSQX, 2 FWMGM, 2 VPVL, 19 CXFTF => 3 HVMC
			5 VJHF, 7 MNCFX, 9 VPVL, 37 CXFTF => 6 GNMV
			145 ORE => 6 MNCFX
			1 NVRVD => 8 CXFTF
			1 VJHF, 6 MNCFX => 4 RFSQX
			176 ORE => 6 VJHF
		";
		let eqs = parse(&input).expect("parsing failed");
		assert_eq!(calc_ore(&eqs, "FUEL", 1, &mut HashMap::new()).expect("calc failed"), 180697);
		assert_eq!(calc_fuel(&eqs, 1000000000000).expect("calc failed"), 5586022);
	}

	#[test]
	fn example5() {
		let input = r"
			171 ORE => 8 CNZTR
			7 ZLQW, 3 BMBT, 9 XCVML, 26 XMNCP, 1 WPTQ, 2 MZWV, 1 RJRHP => 4 PLWSL
			114 ORE => 4 BHXH
			14 VRPVC => 6 BMBT
			6 BHXH, 18 KTJDG, 12 WPTQ, 7 PLWSL, 31 FHTLT, 37 ZDVW => 1 FUEL
			6 WPTQ, 2 BMBT, 8 ZLQW, 18 KTJDG, 1 XMNCP, 6 MZWV, 1 RJRHP => 6 FHTLT
			15 XDBXC, 2 LTCX, 1 VRPVC => 6 ZLQW
			13 WPTQ, 10 LTCX, 3 RJRHP, 14 XMNCP, 2 MZWV, 1 ZLQW => 1 ZDVW
			5 BMBT => 4 WPTQ
			189 ORE => 9 KTJDG
			1 MZWV, 17 XDBXC, 3 XCVML => 2 XMNCP
			12 VRPVC, 27 CNZTR => 2 XDBXC
			15 KTJDG, 12 BHXH => 5 XCVML
			3 BHXH, 2 VRPVC => 7 MZWV
			121 ORE => 7 VRPVC
			7 XCVML => 6 RJRHP
			5 BHXH, 4 VRPVC => 5 LTCX
		";
		let eqs = parse(&input).expect("parsing failed");
		assert_eq!(calc_ore(&eqs, "FUEL", 1, &mut HashMap::new()).expect("calc failed"), 2210736);
		assert_eq!(calc_fuel(&eqs, 1000000000000).expect("calc failed"), 460664);
	}
}
