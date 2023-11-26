use std::{cell::RefCell, collections::HashSet, fmt::Display, rc::Rc};

use instruction::InstrSet;
use llvm::temp::Temp;
use utils::Label;

pub type Node = Rc<RefCell<BasicBlock>>;

pub struct BasicBlock {
	pub id: i32,
	pub prev: Vec<Node>,
	pub succ: Vec<Node>, // 这个不用修了
	pub defs: HashSet<Temp>,
	pub uses: HashSet<Temp>,
	pub live_in: HashSet<Temp>,
	pub live_out: HashSet<Temp>,
	pub instrs: InstrSet,
}

impl BasicBlock {
	pub fn new(id: i32) -> BasicBlock {
		BasicBlock {
			id,
			prev: Vec::new(),
			succ: Vec::new(),
			defs: HashSet::new(),
			uses: HashSet::new(),
			live_in: HashSet::new(),
			live_out: HashSet::new(),
			instrs: InstrSet::LlvmInstrSet(Vec::new()),
		}
	}
	pub fn new_node(id: i32) -> Node {
		Rc::new(RefCell::new(Self::new(id)))
	}
	pub fn label(&self) -> Label {
		match self.id {
			0 => Label::new("entry"),
			1 => Label::new("exit"),
			_ => Label::new(format!("B{}", self.id - 2)),
		}
	}
}

#[cfg(not(feature = "debug"))]
impl Display for BasicBlock {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}:\n{}", self.label(), self.instrs)
	}
}

#[cfg(feature = "debug")]
impl Display for BasicBlock {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let prev: Vec<_> =
			self.prev.iter().map(|v| v.borrow().id.clone()).collect();
		let succ: Vec<_> =
			self.succ.iter().map(|v| v.borrow().id.clone()).collect();
		let defs: Vec<_> = self.defs.iter().map(|v| v.name.as_str()).collect();
		let uses: Vec<_> = self.uses.iter().map(|v| v.name.as_str()).collect();
		let live_in: Vec<_> =
			self.live_in.iter().map(|v| v.name.as_str()).collect();
		let live_out: Vec<_> =
			self.live_out.iter().map(|v| v.name.as_str()).collect();
		write!(
			f,
			"prev: {:?} succ: {:?}\nuses: {:?} defs: {:?}\nlivein: {:?} liveout:{:?}\nB{}:\n{}",
			prev, succ, uses, defs, live_in, live_out, self.label(), self.instrs
		)
	}
}
