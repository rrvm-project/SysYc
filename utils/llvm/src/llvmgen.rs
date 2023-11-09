use crate::{
	func::LlvmFunc,
	label::Label,
	llvminstr::*,
	llvmop::*,
	llvmvar::VarType,
	temp::{Temp, TempManager},
};

pub struct LlvmGen {
	func: Vec<Box<dyn LlvmInstr>>,
	temp_mgr: TempManager,
}

impl LlvmGen {
	pub fn visit_label(&mut self, label: Label) {
		self.func.push(Box::new(LabelInstr { label }))
	}
	pub fn visit_arith_instr(
		&mut self,
		lhs: Value,
		op: ArithOp,
		rhs: Value,
	) -> Temp {
		let target = self.temp_mgr.new_temp(op.oprand_type());
		let instr = ArithInstr {
			target: target.clone(),
			var_type: op.oprand_type(),
			lhs,
			op,
			rhs,
		};
		self.func.push(Box::new(instr));
		target
	}
	pub fn visit_comp_instr(
		&mut self,
		lhs: Value,
		op: CompOp,
		rhs: Value,
	) -> Temp {
		fn get_kind(op: &CompOp) -> CompKind {
			match op.oprand_type() {
				VarType::I32 => CompKind::Icmp,
				VarType::F32 => CompKind::Fcmp,
				_ => unreachable!(),
			}
		}
		let target = self.temp_mgr.new_temp(op.oprand_type());
		let instr = CompInstr {
			kind: get_kind(&op),
			target: target.clone(),
			var_type: op.oprand_type(),
			lhs,
			op,
			rhs,
		};
		self.func.push(Box::new(instr));
		target
	}
	pub fn visit_end(&self) -> LlvmFunc {
		todo!()
	}
}
