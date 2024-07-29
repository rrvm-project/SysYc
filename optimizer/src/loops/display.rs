use std::fmt::Display;

use super::{OpType, TempGraph};

impl Display for TempGraph {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "==========TempGraph:==========")?;
		let mut res = String::new();
		for (temp, ops) in &self.temp_graph {
			res.push_str(&format!("{}: ", temp));
			for op in ops {
				res.push_str(&format!("{} ", op));
			}
			res.push('\n');
		}
		write!(f, "{}", res)
	}
}

impl Display for OpType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			OpType::Phi(t) => write!(f, "Phi({})", t),
			OpType::Add(t) => write!(f, "Add({})", t),
			OpType::Sub(t) => write!(f, "Sub({})", t),
			OpType::Fadd(t) => write!(f, "Fadd({})", t),
			OpType::Fsub(t) => write!(f, "Fsub({})", t),
			OpType::Mul(t) => write!(f, "Mul({})", t),
			OpType::Fmul(t) => write!(f, "Fmul({})", t),
			OpType::Div(t) => write!(f, "Div({})", t),
			OpType::Fdiv(t) => write!(f, "Fdiv({})", t),
			OpType::Mod(t) => write!(f, "Mod({})", t),
			OpType::Others(t) => write!(f, "Others({})", t),
		}
	}
}
