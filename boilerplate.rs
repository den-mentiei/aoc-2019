use std::io::{self, Read};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T, E = Error> = std::result::Result<T, E>;

fn main() -> Result<()> {
	let mut input = String::new();
	io::stdin().read_to_string(&mut input)?;

	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn examples() {
	}
}
