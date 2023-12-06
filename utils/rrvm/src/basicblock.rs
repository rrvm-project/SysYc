use std::{
	cell::RefCell,
	collections::{HashMap, HashSet},
	fmt::Display,
	rc::Rc,
};

use instruction::riscv::{reg::RiscvReg, RiscvInstr};
use llvm::{JumpInstr, LlvmInstr, PhiInstr, RetInstr, VarType};
use utils::{InstrTrait, Label, TempTrait, UseTemp};

pub type Node<T, U> = Rc<RefCell<BasicBlock<T, U>>>;

pub struct BasicBlock<T: InstrTrait<U>, U: TempTrait> {
	pub id: i32,
	pub weight: f64,
	pub prev: Vec<Node<T, U>>,
	pub succ: Vec<Node<T, U>>,
	pub defs: HashSet<U>,
	pub uses: HashSet<U>,
	pub live_in: HashSet<U>,
	pub live_out: HashSet<U>,
	pub phi_instrs: Vec<PhiInstr>,
	pub instrs: Vec<T>,
	pub jump_instr: Option<T>,
}

impl<T: InstrTrait<U>, U: TempTrait> BasicBlock<T, U> {
	pub fn new(id: i32, weight: f64) -> BasicBlock<T, U> {
		BasicBlock {
			id,
			weight,
			prev: Vec::new(),
			succ: Vec::new(),
			defs: HashSet::new(),
			uses: HashSet::new(),
			live_in: HashSet::new(),
			live_out: HashSet::new(),
			phi_instrs: Vec::new(),
			instrs: Vec::new(),
			jump_instr: None,
		}
	}
	pub fn new_node(id: i32, weight: f64) -> Node<T, U> {
		Rc::new(RefCell::new(Self::new(id, weight)))
	}
	pub fn label(&self) -> Label {
		match self.id {
			0 => Label::new("entry"),
			_ => Label::new(format!("B{}", self.id)),
		}
	}
	// Use this before drop a BasicBlock, or may lead to memory leak
	pub fn clear(&mut self) {
		self.prev.clear();
		self.succ.clear();
	}
	pub fn push(&mut self, instr: T) {
		self.instrs.push(instr);
	}
	pub fn push_phi(&mut self, instr: PhiInstr) {
		self.phi_instrs.push(instr);
	}
	pub fn single_prev(&self) -> bool {
		self.prev.len() == 1
	}
	pub fn single_succ(&self) -> bool {
		self.succ.len() == 1
	}
	pub fn get_succ(&self) -> Node<T, U> {
		self.succ.first().unwrap().clone()
	}
	pub fn no_phi(&self) -> bool {
		self.phi_instrs.is_empty()
	}
	pub fn replace_prev(&mut self, label: &Label, target: Node<T, U>) {
		let new_label = target.borrow().label();
		for instr in self.phi_instrs.iter_mut() {
			if let Some((_, v)) = instr.source.iter_mut().find(|(_, v)| v == label) {
				*v = new_label.clone();
			}
		}
		if let Some(prev) =
			self.prev.iter_mut().find(|v| v.borrow().label() == *label)
		{
			*prev = target
		} else {
			unreachable!()
		}
	}
	pub fn make_pretty(&mut self) {
		self.phi_instrs.sort_unstable_by(|x, y| x.target.cmp(&y.target));
	}
	pub fn set_jump(&mut self, instr: Option<T>) {
		self.jump_instr = instr;
	}
	pub fn init(&mut self) {
		for instr in self.instrs.iter().chain(self.jump_instr.iter()) {
			for temp in instr.get_read() {
				self.uses.insert(temp);
			}
			if let Some(temp) = instr.get_write() {
				self.defs.insert(temp);
			}
		}
		self.uses.retain(|v| !self.defs.contains(v));
	}
}

impl BasicBlock<LlvmInstr, llvm::Temp> {
	pub fn gen_jump(&mut self, var_type: VarType) {
		if self.jump_instr.is_none() {
			self.jump_instr = Some(match self.succ.len() {
				1 => Box::new(JumpInstr {
					target: self.succ.first().unwrap().borrow().label(),
				}),
				0 => Box::new(RetInstr {
					value: var_type.default_value_option(),
				}),
				_ => unreachable!(),
			});
		}
	}
	pub fn init_phi(&mut self) {
		for instr in self.phi_instrs.iter() {
			for temp in instr.get_read() {
				self.uses.insert(temp);
			}
			if let Some(temp) = instr.get_write() {
				self.defs.insert(temp);
			}
		}
	}
}

impl BasicBlock<RiscvInstr, instruction::Temp> {
	pub fn map_temp(&mut self, map: &HashMap<instruction::Temp, RiscvReg>) {
		self.instrs.iter_mut().for_each(|v| v.map_temp(map))
	}
}

fn instr_format<T: Display>(v: T) -> String {
	format!("  {}", v)
}

#[cfg(not(feature = "debug"))]
impl<T: InstrTrait<U>, U: TempTrait> Display for BasicBlock<T, U> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let instrs = self
			.phi_instrs
			.iter()
			.map(instr_format)
			.chain(self.instrs.iter().map(instr_format))
			.chain(self.jump_instr.iter().map(instr_format))
			.collect::<Vec<_>>()
			.join("\n");
		write!(f, "  {}:\n{}", self.label(), instrs)
	}
}

#[cfg(feature = "debug")]
impl<T: InstrTrait<U>, U: TempTrait> Display for BasicBlock<T, U> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let prev: Vec<_> = self.prev.iter().map(|v| v.borrow().id).collect();
		let succ: Vec<_> = self.succ.iter().map(|v| v.borrow().id).collect();
		let defs: Vec<_> = self.defs.iter().map(|v| v.to_string()).collect();
		let uses: Vec<_> = self.uses.iter().map(|v| v.to_string()).collect();
		let live_in: Vec<_> = self.live_in.iter().map(|v| v.to_string()).collect();
		let live_out: Vec<_> =
			self.live_out.iter().map(|v| v.to_string()).collect();
		let instrs = self
			.phi_instrs
			.iter()
			.map(instr_format)
			.chain(self.instrs.iter().map(instr_format))
			.chain(self.jump_instr.iter().map(instr_format))
			.collect::<Vec<_>>()
			.join("\n");
		write!(
			f,
			"  {}:
    prev: {:?} succ: {:?}
    uses: {:?}
    defs: {:?}
    livein: {:?}
    liveout: {:?}\n{}",
			self.label(),
			prev,
			succ,
			uses,
			defs,
			live_in,
			live_out,
			instrs
		)
	}
}
