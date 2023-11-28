use std::{cell::RefCell, collections::HashSet, fmt::Display, rc::Rc};

use llvm::{temp::Temp, JumpInstr, LlvmInstr, PhiInstr};
use utils::Label;

pub type Node<T> = Rc<RefCell<BasicBlock<T>>>;

pub struct BasicBlock<T: Display> {
	pub id: i32,
	pub prev: Vec<Node<T>>,
	pub succ: Vec<Node<T>>, // 这个不用修了
	pub defs: HashSet<Temp>,
	pub uses: HashSet<Temp>,
	pub live_in: HashSet<Temp>,
	pub live_out: HashSet<Temp>,
	pub phi_instrs: Vec<PhiInstr>,
	pub instrs: Vec<T>,
}

impl<T: Display> BasicBlock<T> {
	pub fn new(id: i32) -> BasicBlock<T> {
		BasicBlock {
			id,
			prev: Vec::new(),
			succ: Vec::new(),
			defs: HashSet::new(),
			uses: HashSet::new(),
			live_in: HashSet::new(),
			live_out: HashSet::new(),
			phi_instrs: Vec::new(),
			instrs: Vec::new(),
		}
	}
	pub fn new_node(id: i32) -> Node<T> {
		Rc::new(RefCell::new(Self::new(id)))
	}
	pub fn label(&self) -> Label {
		match self.id {
			0 => Label::new("entry", Some(0)),
			_ => Label::new(format!("B{}", self.id), Some(self.id)),
		}
	}
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
}

impl BasicBlock<LlvmInstr> {
	pub fn gen_jump(&mut self) {
		if self.succ.len() == 1 {
			let instr = Box::new(JumpInstr {
				target: self.succ.first().unwrap().borrow().label(),
			});
			self.instrs.push(instr);
		}
	}
}

#[cfg(not(feature = "debug"))]
impl<T: Display> Display for BasicBlock<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let instrs = self
			.phi_instrs
			.iter()
			.map(|v| format!("  {}", v))
			.chain(self.instrs.iter().map(|v| format!("  {}", v)))
			.collect::<Vec<_>>()
			.join("\n");
		write!(f, "  {}:\n{}", self.label(), instrs)
	}
}

#[cfg(feature = "debug")]
impl<T: Display> Display for BasicBlock<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let prev: Vec<_> = self.prev.iter().map(|v| v.borrow().id).collect();
		let succ: Vec<_> = self.succ.iter().map(|v| v.borrow().id).collect();
		let defs: Vec<_> = self.defs.iter().map(|v| v.name.as_str()).collect();
		let uses: Vec<_> = self.uses.iter().map(|v| v.name.as_str()).collect();
		let live_in: Vec<_> =
			self.live_in.iter().map(|v| v.name.as_str()).collect();
		let live_out: Vec<_> =
			self.live_out.iter().map(|v| v.name.as_str()).collect();
		let instrs = self
			.phi_instrs
			.iter()
			.map(|v| format!("  {}", v))
			.chain(self.instrs.iter().map(|v| format!("  {}", v)))
			.collect::<Vec<_>>()
			.join("\n");
		write!(
			f,
			"  {}: prev: {:?} succ: {:?} uses: {:?} defs: {:?} livein: {:?} liveout: {:?}\n{}",
			self.label(), prev, succ, uses, defs, live_in, live_out,  instrs
		)
	}
}
