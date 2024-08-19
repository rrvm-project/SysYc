use std::fmt::Display;

use llvm::Value;
// 每个 induction variable 具有通项公式： a_{n+1} = (scale / divisor) * a_n + step
#[derive(Debug, Clone)]
pub struct IndVar {
	pub base: Value,
	pub scale: Value,
	pub divisor: Value,
	pub step: Vec<Value>,
	pub zfp: Option<Value>,
}

impl IndVar {
	// 从一个循环不变量构造 0 阶归纳变量
	pub fn from_loop_invariant(base: Value) -> Self {
		Self {
			base,
			scale: Value::Int(1),
			divisor: Value::Int(1),
			step: Vec::new(),
			zfp: None,
		}
	}
	pub fn new(
		base: Value,
		scale: Value,
		divisor: Value,
		step: Vec<Value>,
		zfp: Option<Value>,
	) -> Self {
		assert!(scale != Value::Int(0));
		Self {
			base,
			scale,
			divisor,
			step,
			zfp,
		}
	}
}

impl Display for IndVar {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "base: {}", self.base)?;
		write!(f, " scale: {}", self.scale)?;
		write!(f, " divisor: {}", self.divisor)?;
		write!(f, " step: ")?;
		for s in self.step.iter() {
			write!(f, "{} ", s)?;
		}
		write!(f, "zfp: ")?;
		if let Some(z) = &self.zfp {
			write!(f, "{}", z)?;
		} else {
			write!(f, "None")?;
		}
		Ok(())
	}
}
