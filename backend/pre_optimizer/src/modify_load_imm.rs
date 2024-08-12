use std::collections::{HashMap, HashSet};

use instruction::{
	riscv::{
		prelude::{CloneRiscvInstr, RiscvInstrVariant::*},
		riscvop::*,
		value::{
			RiscvImm,
			RiscvTemp::{self, VirtReg},
		},
	},
	Temp,
};
use rrvm::{
	dominator::RiscvDomTree,
	program::{RiscvFunc, RiscvProgram},
	RiscvNode,
};

#[derive(Default, Clone)]
struct NodeInfo {
	pub li_instrs: HashMap<RiscvImm, RiscvTemp>,
	pub addi_instrs: HashMap<(RiscvTemp, RiscvImm), RiscvTemp>,
	pub convert_instrs: HashMap<RiscvTemp, RiscvTemp>,
	pub temp_mapper: HashMap<Temp, RiscvTemp>,
}

struct Solver {
	dom_tree: RiscvDomTree,
	stack: Vec<NodeInfo>,
}

impl Solver {
	fn new(func: &RiscvFunc) -> Self {
		Self {
			dom_tree: RiscvDomTree::new(&func.cfg, false),
			stack: Vec::new(),
		}
	}
	pub fn detect_load_imm(
		&mut self,
		node: RiscvNode,
		best_node: Option<RiscvNode>,
	) {
		fn get_node(node: RiscvNode, best_node: Option<RiscvNode>) -> RiscvNode {
			let block = &mut node.borrow_mut();
			match best_node {
				Some(best_node) if best_node.borrow().weight < block.weight => {
					let mut constants = HashSet::new();
					block.instrs.retain(|instr| match instr.get_variant() {
						IBinInstr(instr) if instr.op == Li => {
							!instr.rd.is_virtual() || {
								constants.insert(instr.rd);
								best_node.borrow_mut().instrs.push(instr.clone_box());
								false
							}
						}
						ITriInstr(instr) if instr.op == Addi => {
							!instr.rd.is_virtual() || !constants.contains(&instr.rs1) || {
								constants.insert(instr.rd);
								best_node.borrow_mut().instrs.push(instr.clone_box());
								false
							}
						}
						RBinInstr(instr) if instr.op == MvInt2Float => {
							!instr.rd.is_virtual() || {
								best_node.borrow_mut().instrs.push(instr.clone_box());
								false
							}
						}
						_ => true,
					});
					best_node
				}
				_ => node.clone(),
			}
		}
		let children = self.dom_tree.get_children(node.borrow().id).clone();
		let best_node = Some(get_node(node, best_node));
		for v in children {
			self.detect_load_imm(v, best_node.clone());
		}
	}
	pub fn solve_load_imm(&mut self, node: RiscvNode) {
		let block = &mut node.borrow_mut();
		let mut info = self.stack.last().cloned().unwrap_or_default();
		block.instrs.retain_mut(|instr| {
			instr.map_temp(&info.temp_mapper);
			match instr.get_variant() {
				IBinInstr(instr) if instr.op == Li => {
					if let VirtReg(rd) = instr.rd {
						if let Some(temp) = info.li_instrs.get(&instr.rs1) {
							info.temp_mapper.insert(rd, *temp);
							false
						} else {
							info.li_instrs.insert(instr.rs1.clone(), instr.rd);
							true
						}
					} else {
						true
					}
				}
				ITriInstr(instr) if instr.op == Addi => {
					if let VirtReg(rd) = instr.rd {
						if let Some(temp) =
							info.addi_instrs.get(&(instr.rs1, instr.rs2.clone()))
						{
							info.temp_mapper.insert(rd, *temp);
							false
						} else {
							info.addi_instrs.insert((instr.rs1, instr.rs2.clone()), instr.rd);
							true
						}
					} else {
						true
					}
				}
				RBinInstr(instr) if instr.op == MvInt2Float => {
					if let VirtReg(rd) = instr.rd {
						if let Some(temp) = info.convert_instrs.get(&instr.rs1) {
							info.temp_mapper.insert(rd, *temp);
							false
						} else {
							info.convert_instrs.insert(instr.rs1, instr.rd);
							true
						}
					} else {
						true
					}
				}
				_ => true,
			}
		});
		if let Some(v) = block.jump_instr.as_mut() {
			v.map_temp(&info.temp_mapper)
		}
		self.stack.push(info);
		let children = self.dom_tree.get_children(block.id).clone();
		for v in children {
			self.solve_load_imm(v);
		}
		self.stack.pop();
	}
}

pub fn modify_load_imm(program: &mut RiscvProgram) {
	for func in program.funcs.iter_mut() {
		let mut solver = Solver::new(func);
		solver.detect_load_imm(func.cfg.get_entry(), None);
		solver.solve_load_imm(func.cfg.get_entry());
	}
}
