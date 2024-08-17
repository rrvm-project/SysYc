use std::collections::HashMap;

use instruction::riscv::{
	prelude::{CloneRiscvInstr, RTriInstr, RiscvInstrVariant},
	riscvop::*,
	value::RiscvTemp::{self},
};
use rrvm::{
	dominator::RiscvDomTree,
	program::{RiscvFunc, RiscvProgram},
	RiscvNode,
};

struct Solver {
	dom_tree: RiscvDomTree,
	shift_temps: HashMap<RiscvTemp, (RiscvTemp, i32)>,
	use_count: HashMap<RiscvTemp, i32>,
}

impl Solver {
	fn new(func: &RiscvFunc) -> Self {
		Self {
			dom_tree: RiscvDomTree::new(&func.cfg, false),
			shift_temps: HashMap::new(),
			use_count: HashMap::new(),
		}
	}
	fn calc_use_state(&mut self, node: &RiscvNode) {
		let block = node.borrow();
		for instr in block.instrs.iter() {
			if let RiscvInstrVariant::ITriInstr(instr) = instr.get_variant() {
				if (instr.op == Slli || instr.op == Slliw)
					&& instr.rs2.get_i32().is_some()
				{
					self
						.shift_temps
						.insert(instr.rd, (instr.rs1, instr.rs2.get_i32().unwrap()));
				}
			}
			for temp in instr.get_riscv_read() {
				*self.use_count.entry(temp).or_default() += 1;
			}
		}
	}
	pub fn get_unique_use(&mut self, node: RiscvNode) {
		self.calc_use_state(&node);
		let children = self.dom_tree.get_children(node.borrow().id).clone();
		for v in children {
			self.get_unique_use(v);
		}
	}
	fn rewrite_instrs(&mut self, node: &RiscvNode) {
		fn get_shadd_op(offset: i32) -> Option<RTriInstrOp> {
			match offset {
				1 => Some(Sh1add),
				2 => Some(Sh2add),
				3 => Some(Sh3add),
				_ => None,
			}
		}

		let mut block = node.borrow_mut();
		let instrs = std::mem::take(&mut block.instrs);
		for instr in instrs.into_iter().rev() {
			match instr.get_variant() {
				RiscvInstrVariant::ITriInstr(instr)
					if (instr.op == Slli || instr.op == Slliw)
						&& instr.rs2.get_i32().unwrap() <= 3 =>
				{
					if self.use_count.get(&instr.rd).map_or(false, |v| *v > 0) {
						block.instrs.push(instr.clone_box());
					}
				}
				RiscvInstrVariant::RTriInstr(instr)
					if instr.op == Add || instr.op == Addw =>
				{
					if let Some((rs1, offset)) = self.shift_temps.get(&instr.rs2) {
						if let Some(op) = get_shadd_op(*offset) {
							if let Some(count) = self.use_count.get_mut(&instr.rs2) {
								if *count == 1 {
									let new_instr = RTriInstr::new(op, instr.rd, *rs1, instr.rs1);
									block.instrs.push(new_instr);
									*count -= 1;
									continue;
								}
							}
						}
					}
					if let Some((rs1, offset)) = self.shift_temps.get(&instr.rs1) {
						if let Some(op) = get_shadd_op(*offset) {
							if let Some(count) = self.use_count.get_mut(&instr.rs1) {
								if *count == 1 {
									let new_instr = RTriInstr::new(op, instr.rd, *rs1, instr.rs2);
									block.instrs.push(new_instr);
									*count -= 1;
									continue;
								}
							}
						}
					}
					block.instrs.push(instr.clone_box());
				}
				_ => block.instrs.push(instr.clone_box()),
			}
		}
		block.instrs.reverse();
	}
	pub fn solve_shift_add(&mut self, node: RiscvNode) {
		let children = self.dom_tree.get_children(node.borrow().id).clone();
		for v in children {
			self.solve_shift_add(v);
		}
		self.rewrite_instrs(&node);
	}
}

pub fn shift_add(program: &mut RiscvProgram) {
	for func in program.funcs.iter_mut() {
		let mut solver = Solver::new(func);
		solver.get_unique_use(func.cfg.get_entry());
		solver.solve_shift_add(func.cfg.get_entry());
	}
}
