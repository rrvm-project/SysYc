use std::collections::HashMap;

use llvm::{
	ArithInstr, GEPInstr, LlvmInstr, LlvmTemp, PhiInstr, Value, VarType,
};
use utils::Label;

use crate::loops::{indvar::IndVar, indvar_type::IndVarType, temp_graph::Node};

use super::OneLoopSolver;

impl<'a> OneLoopSolver<'a> {
	// 返回成功与否
	pub fn try_strength_reduce(
		&mut self,
		target: &LlvmTemp,
		iv: &IndVar,
		reduce_map: &mut HashMap<LlvmTemp, LlvmTemp>,
	) -> bool {
		// 被我 reduce 的变量所在的基本块一定要支配 loop 唯一的 latch 块
		let def_bb = self.loopdata.def_map[target].clone();
		let latch_bb = self
			.cur_loop
			.borrow()
			.get_loop_latch(&self.loopdata.loop_map)
			.expect("single latch block not found");
		if !self.dom_tree.dominates[&def_bb.borrow().id].contains(&latch_bb) {
			#[cfg(feature = "debug")]
			eprintln!("SR: not reducing iv: {} because def block does not dominate latch block", iv);
			return false;
		}
		if iv.get_type() == IndVarType::Ordinary {
			#[cfg(feature = "debug")]
			eprintln!(
				"SR: reducing iv: {} {} \nwhich is defined as {}",
				target, iv, self.loopdata.temp_graph.temp_to_instr[target].instr
			);
			// return false;
			let new_temp = self.temp_mgr.new_temp(target.var_type, false);
			let new_instr: LlvmInstr = match new_temp.var_type {
				VarType::I32 => Box::new(ArithInstr {
					target: new_temp.clone(),
					op: llvm::ArithOp::Add,
					var_type: new_temp.var_type,
					lhs: Value::Temp(target.clone()),
					rhs: iv.step[0].clone(),
				}),
				VarType::I32Ptr | VarType::F32Ptr => Box::new(GEPInstr {
					target: new_temp.clone(),
					var_type: new_temp.var_type,
					addr: Value::Temp(target.clone()),
					offset: iv.step[0].clone(),
				}),
				_ => unreachable!(),
			};
			reduce_map.insert(target.clone(), new_temp.clone());
			let preheader_label = self.get_cur_loop_preheader().borrow().label();
			let other_labels: Vec<Label> = self
				.cur_loop
				.borrow()
				.header
				.borrow()
				.prev
				.iter()
				.filter(|node| {
					node.borrow().id != self.get_cur_loop_preheader().borrow().id
				})
				.map(|node| node.borrow().label())
				.collect();
			let mut new_sources = vec![(iv.base.clone(), preheader_label)];
			new_sources.extend(
				other_labels
					.into_iter()
					.map(|label| (Value::Temp(new_temp.clone()), label)),
			);
			let new_phi = PhiInstr {
				target: target.clone(),
				var_type: target.var_type,
				source: new_sources,
			};
			self
				.cur_loop
				.borrow_mut()
				.header
				.borrow_mut()
				.phi_instrs
				.push(new_phi.clone());
			if let Some(t) = iv.base.unwrap_temp() {
				self.place_temp_into_cfg(&t);
			}
			if let Some(t) = iv.step[0].unwrap_temp() {
				self.place_temp_into_cfg(&t);
			}
			self.flag = true;

			self
				.loopdata
				.def_map
				.insert(new_temp.clone(), self.loopdata.def_map[target].clone());
			self
				.loopdata
				.def_map
				.insert(target.clone(), self.cur_loop.borrow().header.clone());
			self.loopdata.temp_graph.temp_to_instr.insert(
				target.clone(),
				Node {
					instr: Box::new(new_phi.clone()),
				},
			);
			self
				.loopdata
				.temp_graph
				.temp_to_instr
				.insert(new_temp.clone(), Node { instr: new_instr });
			true
		} else {
			#[cfg(feature = "debug")]
			eprintln!("SR: not reducing iv: {}", iv);
			false
		}
	}
}
