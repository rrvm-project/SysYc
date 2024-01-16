use std::collections::HashMap;

use llvm::{
	ArithInstr, ArithOp, ConvertInstr, ConvertOp, LlvmInstrTrait, LlvmTemp,
	LlvmTempManager, Value, VarType,
};
use rrvm::LlvmCFG;

use super::OSR;

#[derive(Clone)]
pub struct LSTFOperation {
	pub op: ArithOp,
	pub regional_constant: Value,
}
#[derive(Clone)]
pub struct LSTFEdge {
	pub target: LlvmTemp,
	pub ops: Vec<LSTFOperation>,
}

impl OSR {
	pub fn add_lstf_edge(
		&mut self,
		from: LlvmTemp,
		to: LlvmTemp,
		op: ArithOp,
		rc: Value,
	) {
		self.lstf_map.insert(
			from,
			LSTFEdge {
				target: to,
				ops: vec![LSTFOperation {
					op,
					regional_constant: rc,
				}],
			},
		);
	}
	pub fn fix_lstf_map(&mut self) {
		fn fix(k: LlvmTemp, map: &mut HashMap<LlvmTemp, LSTFEdge>) -> LSTFEdge {
			let v = map.get(&k).cloned().unwrap();
			if map.get(&v.target).is_some() {
				let v2 = fix(v.target.clone(), map);
				map.entry(k).and_modify(|e| {
					e.ops.extend(v2.ops.clone());
					e.target = v2.target.clone();
				});
				return v2;
			}
			v
		}
		let keys = self.lstf_map.keys().cloned().collect::<Vec<_>>();
		for k in keys {
			fix(k, &mut self.lstf_map);
		}
	}
	pub fn lstf(&mut self, cfg: &mut LlvmCFG, mgr: &mut LlvmTempManager) {
		self.fix_lstf_map();
		let mut cmps = Vec::new();
		for block in cfg.blocks.iter_mut() {
			let block = block.borrow_mut();
			for instr in block.instrs.iter() {
				if instr.is_cmp() {
					cmps.push(instr.get_write().unwrap());
				}
			}
		}
		for cmp in cmps {
			let (_, bb_index, instr_index, _) = self.temp_to_instr[&cmp];
			let (lhs, rhs) = cfg.blocks[bb_index].borrow().instrs[instr_index]
				.get_lhs_and_rhs()
				.unwrap();
			if lhs.unwrap_temp().is_some_and(|t| self.lstf_map.contains_key(&t)) {
				let t = lhs.unwrap_temp().unwrap();
				let edge = self.lstf_map.get(&t).cloned().unwrap();
				let mut new_instrs = Vec::new();
				let mut cur_temp = rhs;
				for op in edge.ops.iter() {
					cur_temp = Value::Temp(self.get_new_instr(
						op.op,
						cur_temp,
						op.regional_constant.clone(),
						&mut new_instrs,
						mgr,
					));
				}
				cfg.blocks[bb_index].borrow_mut().instrs[instr_index]
					.set_read_values(0, Value::Temp(edge.target.clone()));
				cfg.blocks[bb_index].borrow_mut().instrs[instr_index]
					.set_read_values(1, cur_temp);
				for instr in
					cfg.blocks[bb_index].borrow_mut().instrs.iter().skip(instr_index)
				{
					instr.get_write().map(|t| {
						self
							.temp_to_instr
							.entry(t)
							.and_modify(|(_, _, instr_id, _)| *instr_id += new_instrs.len())
					});
				}
				cfg.blocks[bb_index]
					.borrow_mut()
					.instrs
					.splice(instr_index..instr_index, new_instrs);
			}
		}
	}
	fn get_new_instr(
		&mut self,
		op: ArithOp,
		lhs: Value,
		rhs: Value,
		new_instrs: &mut Vec<Box<dyn LlvmInstrTrait>>,
		mgr: &mut LlvmTempManager,
	) -> LlvmTemp {
		match (lhs.get_type(), rhs.get_type()) {
			(VarType::I32, VarType::I32) => {
				let new_tmp = self.new_temp(VarType::I32, mgr);
				new_instrs.push(ArithInstr::new(
					new_tmp.clone(),
					lhs,
					op,
					rhs,
					VarType::I32,
				));
				new_tmp
			}
			(VarType::I32, VarType::F32) => {
				let new_tmp1 = self.new_temp(VarType::F32, mgr);
				new_instrs.push(ConvertInstr::new(
					new_tmp1.clone(),
					lhs,
					ConvertOp::Int2Float,
					VarType::I32,
					VarType::F32,
				));
				let new_tmp2 = self.new_temp(VarType::F32, mgr);
				new_instrs.push(ArithInstr::new(
					new_tmp2.clone(),
					new_tmp1,
					op,
					rhs,
					VarType::F32,
				));
				new_tmp2
			}
			(VarType::F32, VarType::I32) => {
				let new_tmp1 = self.new_temp(VarType::F32, mgr);
				new_instrs.push(ConvertInstr::new(
					new_tmp1.clone(),
					rhs,
					ConvertOp::Int2Float,
					VarType::I32,
					VarType::F32,
				));
				let new_tmp2 = self.new_temp(VarType::F32, mgr);
				new_instrs.push(ArithInstr::new(
					new_tmp2.clone(),
					lhs,
					op,
					new_tmp1,
					VarType::F32,
				));
				new_tmp2
			}
			(VarType::F32, VarType::F32) => {
				let new_tmp = self.new_temp(VarType::F32, mgr);
				new_instrs.push(ArithInstr::new(
					new_tmp.clone(),
					lhs,
					op,
					rhs,
					VarType::F32,
				));
				new_tmp
			}
			_ => unreachable!(),
		}
	}
}
