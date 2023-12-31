use super::OSR;

use llvm::{ArithInstr, ArithOp, Temp, Value};
use rrvm::LlvmCFG;

impl OSR {
	// helper function
	// 这里函数参数不直接传 instr，是为了避免持续保留对cfg的引用
	pub fn is_candidate_operation(
		&self,
		cfg: &LlvmCFG,
		bb_id: usize,
		instr_id: usize,
	) -> Option<(Temp, Value)> {
		let instr = &cfg.blocks[bb_id].borrow().instrs[instr_id];
		if let Some(op) = instr.is_candidate_operator() {
			match op {
				ArithOp::Add | ArithOp::Mul | ArithOp::Fadd | ArithOp::Fmul => {
					let (lhs, rhs) = instr.get_lhs_and_rhs().unwrap();
					if let Some((iv, header)) = self.is_induction_value(lhs.clone()) {
						if let Some(rc) = self.is_regional_constant(header.clone(), rhs) {
							Some((iv, rc))
						} else {
							None
						}
					} else if let Some((iv, header)) =
						self.is_induction_value(rhs.clone())
					{
						if let Some(rc) = self.is_regional_constant(header.clone(), lhs) {
							Some((iv, rc))
						} else {
							None
						}
					} else {
						None
					}
				}
				ArithOp::Sub | ArithOp::Fsub => {
					let (lhs, rhs) = instr.get_lhs_and_rhs().unwrap();
					if let Some((iv, header)) = self.is_induction_value(lhs.clone()) {
						if let Some(rc) = self.is_regional_constant(header.clone(), rhs) {
							Some((iv, rc))
						} else {
							None
						}
					} else {
						None
					}
				}
				_ => None,
			}
		} else {
			None
		}
	}
	// 返回 induction variable 和它的 header
	pub fn is_induction_value(&self, v: Value) -> Option<(Temp, Temp)> {
		match v {
			Value::Temp(t) => {
				if let Some(h) = self.header.get(&t) {
					Some((t, h.clone()))
				} else {
					None
				}
			}
			_ => None,
		}
	}
	// 这里没有考虑rc是否是源自一个立即数操作，因为如果rc本身是个常量，它应该在常量传播时被删掉了
	pub fn is_regional_constant(
		&self,
		iv_header: Temp,
		rc: Value,
	) -> Option<Value> {
		if let Value::Temp(t) = rc.clone() {
			let iv_header_bb_id = self.temp_to_instr[&iv_header].0;
			let rc_bb_id = self.temp_to_instr[&t].0;
			if self
				.dominates
				.get(&rc_bb_id)
				.unwrap()
				.iter()
				.any(|bb| bb.borrow().id == iv_header_bb_id)
			{
				Some(rc)
			} else {
				None
			}
		} else {
			// 否则 rc 是个立即数
			Some(rc)
		}
	}
	// 将一个候选操作转变为一个复制操作
	pub fn replace_to_copy(
		&mut self,
		cfg: &mut LlvmCFG,
		bb_id: usize,
		instr_id: usize,
		from: Temp,
	) {
		let target =
			cfg.blocks[bb_id].borrow().instrs[instr_id].get_write().unwrap();
		let is_float = target.var_type.is_float();
		let copy_instr = ArithInstr {
			target: target.clone(),
			op: if is_float {
				ArithOp::Add
			} else {
				ArithOp::Fadd
			},
			var_type: target.var_type,
			lhs: Value::Temp(from),
			rhs: if is_float {
				Value::Int(0)
			} else {
				Value::Float(0.0)
			},
		};
		cfg.blocks[bb_id].borrow_mut().instrs[instr_id] = Box::new(copy_instr);
		self.flag = true;
	}
}
