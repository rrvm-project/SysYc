use super::OSR;

use llvm::{ArithInstr, ArithOp, LlvmTemp, LlvmTempManager, Value, VarType};
use rrvm::LlvmCFG;
use utils::UseTemp;

impl OSR {
	// helper function
	// 这里函数参数不直接传 instr，是为了避免持续保留对cfg的引用
	pub fn is_candidate_operation(
		&self,
		cfg: &LlvmCFG,
		bb_id: usize,
		instr_id: usize,
		is_phi: bool,
	) -> Option<(LlvmTemp, Value)> {
		if is_phi {
			return None;
		}
		let instr = &cfg.blocks[bb_id].borrow().instrs[instr_id];
		if let Some(op) = instr.is_candidate_operator() {
			match op {
				ArithOp::Add | ArithOp::Mul | ArithOp::Fadd | ArithOp::Fmul => {
					let (lhs, rhs) = instr.get_lhs_and_rhs().unwrap();
					if let Some((iv, header)) = self.is_induction_value(lhs.clone()) {
						self.is_regional_constant(header.clone(), rhs).map(|rc| (iv, rc))
					} else if let Some((iv, header)) =
						self.is_induction_value(rhs.clone())
					{
						self.is_regional_constant(header.clone(), lhs).map(|rc| (iv, rc))
					} else {
						None
					}
				}
				ArithOp::Sub | ArithOp::Fsub => {
					let (lhs, rhs) = instr.get_lhs_and_rhs().unwrap();
					if let Some((iv, header)) = self.is_induction_value(lhs.clone()) {
						self.is_regional_constant(header.clone(), rhs).map(|rc| (iv, rc))
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
	pub fn is_induction_value(&self, v: Value) -> Option<(LlvmTemp, LlvmTemp)> {
		match v {
			Value::Temp(t) => self.header.get(&t).map(|h| (t, h.clone())),
			_ => None,
		}
	}
	#[allow(clippy::if_same_then_else)]
	// 这里没有考虑rc是否是源自一个立即数操作，因为如果rc本身是个常量，它应该在常量传播时被删掉了
	pub fn is_regional_constant(
		&self,
		iv_header: LlvmTemp,
		rc: Value,
	) -> Option<Value> {
		if let Value::Temp(t) = rc.clone() {
			// 函数参数被当作常数看待
			if self.params.contains(&t) {
				return Some(rc);
			}
			if self.header.contains_key(&t) {
				return None;
			}
			let iv_header_bb_id = self.temp_to_instr[&iv_header].0;
			let rc_bb_id = self.temp_to_instr[&t].0;
			if self.dominates.get(&rc_bb_id).unwrap().iter().any(|bb| {
				bb.borrow().id == iv_header_bb_id && bb.borrow().id != rc_bb_id
			}) {
				None
			// Some(rc)
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
		cfg: &LlvmCFG,
		bb_id: usize,
		instr_id: usize,
		from: LlvmTemp,
	) {
		let target =
			cfg.blocks[bb_id].borrow().instrs[instr_id].get_write().unwrap();
		let is_float = target.var_type.is_float();
		let copy_instr = ArithInstr {
			target: target.clone(),
			op: if is_float {
				ArithOp::Fadd
			} else {
				ArithOp::Add
			},
			var_type: target.var_type,
			lhs: Value::Temp(from),
			rhs: if is_float {
				Value::Float(0.0)
			} else {
				Value::Int(0)
			},
		};
		cfg.blocks[bb_id].borrow_mut().instrs[instr_id] = Box::new(copy_instr);
		self.flag = true;
	}
	pub fn new_temp(
		&mut self,
		tp: VarType,
		mgr: &mut LlvmTempManager,
	) -> LlvmTemp {
		mgr.new_temp(tp, false)
	}
	pub fn get_instr_reads(
		&self,
		cfg: &LlvmCFG,
		temp: LlvmTemp,
	) -> Vec<LlvmTemp> {
		let (_, bb_index, instr_index, is_phi) = self.temp_to_instr[&temp];
		if is_phi {
			cfg.blocks[bb_index].borrow().phi_instrs[instr_index].get_read()
		} else {
			cfg.blocks[bb_index].borrow().instrs[instr_index].get_read()
		}
	}

	pub fn is_valid_update_temp(
		&mut self,
		cfg: &LlvmCFG,
		phi_temp: LlvmTemp,
		update_temp: LlvmTemp,
	) -> bool {
		let (_, bb_index, instr_index, is_phi) = self.temp_to_instr[&update_temp];
		if is_phi {
			return false;
		}
		let instr = &cfg.blocks[bb_index].borrow().instrs[instr_index];
		let iv_header = phi_temp.clone();
		if let Some(op) = instr.is_candidate_operator() {
			match op {
				ArithOp::Add | ArithOp::Fadd => {
					let (lhs, rhs) = instr.get_lhs_and_rhs().unwrap();
					if (lhs == Value::Temp(phi_temp.clone())
						&& self
							.is_regional_constant(iv_header.clone(), rhs.clone())
							.is_some()) || (rhs == Value::Temp(phi_temp.clone())
						&& self.is_regional_constant(iv_header, lhs).is_some())
					{
						return true;
					}
				}
				ArithOp::Sub | ArithOp::Fsub => {
					let (lhs, rhs) = instr.get_lhs_and_rhs().unwrap();
					if lhs == Value::Temp(phi_temp.clone())
						&& self.is_regional_constant(iv_header, rhs).is_some()
					{
						return true;
					}
				}
				_ => {}
			}
		}
		false
	}
}
