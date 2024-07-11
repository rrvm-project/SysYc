use std::{
	cell::RefCell,
	collections::{HashMap, HashSet},
	fmt::Display,
	rc::Rc,
};

use instruction::riscv::RiscvInstr;
use llvm::{
	JumpInstr, LlvmInstr, LlvmInstrTrait, LlvmTemp, PhiInstr, RetInstr, Value,
	VarType,
};
use utils::{instr_format, to_label, InstrTrait, Label, TempTrait, UseTemp};

pub type Node<T, U> = Rc<RefCell<BasicBlock<T, U>>>;
pub type LlvmBasicBlock = BasicBlock<LlvmInstr, llvm::LlvmTemp>;

pub struct BasicBlock<T: InstrTrait<U>, U: TempTrait> {
	pub id: i32,
	pub weight: f64,
	pub kill_size: i32,
	pub prev: Vec<Node<T, U>>,
	pub succ: Vec<Node<T, U>>,
	pub defs: HashSet<U>,
	pub uses: HashSet<U>,
	pub kills: HashSet<U>,
	pub live_in: HashSet<U>,
	pub live_out: HashSet<U>,
	pub phi_instrs: Vec<PhiInstr>,
	pub phi_defs: HashSet<LlvmTemp>,
	pub instrs: Vec<T>,
	pub jump_instr: Option<T>,
}

fn get_other_label<T: InstrTrait<U>, U: TempTrait>(
	now: *const BasicBlock<T, U>,
	now_label: Label,
	other: &Node<T, U>,
) -> Label {
	if std::ptr::eq(now, other.as_ptr()) {
		now_label
	} else {
		other.borrow().label()
	}
}

impl<T: InstrTrait<U>, U: TempTrait> BasicBlock<T, U> {
	pub fn new(id: i32, weight: f64) -> Self {
		BasicBlock {
			id,
			weight,
			kill_size: 0,
			prev: Vec::new(),
			succ: Vec::new(),
			defs: HashSet::new(),
			kills: HashSet::new(),
			uses: HashSet::new(),
			live_in: HashSet::new(),
			live_out: HashSet::new(),
			phi_instrs: Vec::new(),
			phi_defs: HashSet::new(),
			instrs: Vec::new(),
			jump_instr: None,
		}
	}
	pub fn new_node(id: i32, weight: f64) -> Node<T, U> {
		Rc::new(RefCell::new(Self::new(id, weight)))
	}
	pub fn label(&self) -> Label {
		to_label(self.id)
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
	pub fn get_prev_iter(&self) -> std::slice::Iter<Node<T, U>> {
		self.prev.iter()
	}
	pub fn no_phi(&self) -> bool {
		self.phi_instrs.is_empty()
	}
	pub fn replace_prev(&mut self, label: &Label, target: Node<T, U>) {
		let new_label = get_other_label(self, self.label(), &target);
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
	pub fn replace_prevs(&mut self, label: &Label, targets: Vec<Node<T, U>>) {
		for instr in self.phi_instrs.iter_mut() {
			let value =
				instr.source.iter_mut().find(|(_, v)| v == label).unwrap().0.clone();
			instr.source.retain(|(_, l)| l != label);
			instr.source.append(
				&mut targets
					.iter()
					.map(|t| (value.clone(), t.borrow().label().clone()))
					.collect(),
			);
		}
		self.prev.retain(|v| v.borrow().label() != *label);
		self.prev.append(&mut targets.clone());
	}
	pub fn make_pretty(&mut self) {
		self.phi_instrs.sort_by(|x, y| x.target.cmp(&y.target));
	}
	pub fn set_jump(&mut self, instr: Option<T>) {
		self.jump_instr = instr;
	}
	pub fn init_data_flow(&mut self) {
		for instr in self.instrs.iter().chain(self.jump_instr.iter()) {
			for temp in instr.get_read() {
				if !self.defs.contains(&temp) {
					self.uses.insert(temp);
				}
			}
			if let Some(temp) = instr.get_write() {
				self.defs.insert(temp);
			}
		}
	}
	pub fn update_phi_def(&mut self) {
		for instr in self.phi_instrs.iter() {
			if let Some(temp) = instr.get_write() {
				self.phi_defs.insert(temp.clone());
			}
		}
	}

	pub fn clear_data_flow(&mut self) {
		self.uses.clear();
		self.defs.clear();
		self.live_in.clear();
		self.live_out.clear();
	}
}

impl LlvmBasicBlock {
	pub fn gen_jump(&mut self, var_type: VarType) {
		if self.jump_instr.is_none() {
			self.jump_instr = Some(match self.succ.len() {
				1 => JumpInstr::new(get_other_label(
					self,
					self.label(),
					self.succ.first().unwrap(),
				)),
				0 => Box::new(RetInstr {
					value: var_type.default_value_option(),
				}),
				_ => unreachable!(),
			});
		}
	}
	pub fn init_phi(&mut self) {
		for instr in self.phi_instrs.iter() {
			if let Some(temp) = instr.get_write() {
				self.defs.insert(temp);
			}
		}
	}
	pub fn update_weight(&mut self, weight: f64) {
		self.weight *= weight;
	}
}

impl BasicBlock<RiscvInstr, instruction::Temp> {
	pub fn sort_succ(&mut self) {
		if self.succ.is_empty() {
			return;
		}
		let label = self.jump_instr.as_ref().unwrap().get_read_label().unwrap();
		let now_label = self.label();
		let now = self as *const BasicBlock<_, _>;
		let (left, right) = self
			.succ
			.drain(..)
			.partition(|v| get_other_label(now, now_label.clone(), v) == label);
		self.succ = left;
		self.succ.extend(right);
	}
}

impl PartialEq for LlvmBasicBlock {
	fn eq(&self, other: &Self) -> bool {
		self.id == other.id
	}
}

impl Clone for LlvmBasicBlock {
	fn clone(&self) -> Self {
		Self {
			phi_instrs: self.phi_instrs.clone(),
			instrs: self.instrs.to_vec(),
			jump_instr: self.jump_instr.as_ref().cloned(),
			kill_size: self.kill_size,
			..Self::new(self.id, self.weight)
		}
	}
}

impl LlvmBasicBlock {
	pub fn map_temp(&mut self, map: &HashMap<LlvmTemp, Value>) {
		self.phi_instrs.iter_mut().for_each(|v| v.map_temp(map));
		self.instrs.iter_mut().for_each(|v| v.map_temp(map));
		if let Some(instr) = self.jump_instr.as_mut() {
			instr.map_temp(map);
		}
	}
	pub fn map_phi_label(&mut self, map: &HashMap<Label, Label>) {
		self.phi_instrs.iter_mut().for_each(|v| v.map_label(map));
	}
	pub fn map_label(&mut self, map: &HashMap<Label, Label>) {
		self.map_phi_label(map);
		if let Some(instr) = self.jump_instr.as_mut() {
			instr.map_label(map);
		}
	}
	pub fn tail_call_func(&self, name: &str) -> bool {
		if self.jump_instr.as_ref().map_or(false, |v| v.is_ret()) {
			if let Some(instr) = self.instrs.last() {
				if instr.is_call() && instr.get_label().name == name {
					return true;
				}
			}
		}
		false
	}
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
    liveout: {:?}
    kill_size: {}\n{}",
			self.label(),
			prev,
			succ,
			uses,
			defs,
			live_in,
			live_out,
			self.kill_size,
			instrs
		)
	}
}
