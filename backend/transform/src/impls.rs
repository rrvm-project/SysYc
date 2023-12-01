use std::fmt::Display;

use crate::InstrSet;

impl Display for InstrSet {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let contest: Vec<_> = match self {
			Self::LlvmInstrSet(v) => v.iter().map(|v| format!("{}\n", v)).collect(),
			Self::RiscvInstrSet(v) => v.iter().map(|v| format!("{}\n", v)).collect(),
		};
		write!(f, "{}", contest.join(""))
	}
}
