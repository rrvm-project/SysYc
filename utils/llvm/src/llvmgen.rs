use crate::{
	func::LlvmFunc,
	label::Label,
	llvminstr::*,
	llvmop::{ArithOp, Value},
	temp::TempManager,
};

pub struct LlvmGen {
	func: Vec<Box<dyn LlvmInstr>>,
	temp_mgr: TempManager,
}

impl LlvmGen {
	pub fn visit_label(&mut self, label: Label) {
		self.func.push(Box::new(LabelInstr { label }))
	}
	pub fn visit_arith_instr(&mut self, lhs: Value, op: ArithOp, rhs: Value) {
		let v = ArithInstr {
			target: self.temp_mgr.new_temp(op.oprand_type()),
			var_type: op.oprand_type(),
			lhs,
			op,
			rhs,
		};
		self.func.push(Box::new(v))
	}
	pub fn visit_end(&self) -> LlvmFunc {
		todo!()
	}
}
