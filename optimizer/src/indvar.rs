use llvm::Value;

// 每个 induction variable 具有通项公式： a_{n+1} = scale * a_n + step
#[derive(Debug, Clone)]
pub struct IndVar {
	pub base: Value,
	pub scale: Value,
	pub step: Value,
	pub is_zfp: Option<Value>,
}

impl IndVar {
	// 从一个循环不变量构造 0 阶归纳变量
	pub fn from_loop_invariant(base: Value) -> Self {
		Self {
			base,
			scale: Value::Int(1),
			step: Value::Int(0),
			is_zfp: None,
		}
	}
	pub fn new(
		base: Value,
		scale: Value,
		step: Value,
		zfp: Option<Value>,
	) -> Self {
		assert!(scale != Value::Int(0));
		Self {
			base,
			scale,
			step,
			is_zfp: zfp,
		}
	}
}
