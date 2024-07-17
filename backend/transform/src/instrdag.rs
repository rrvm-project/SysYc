use std::{cell::RefCell, collections::HashMap, rc::Rc};

use instruction::riscv::{value::RiscvTemp, RiscvInstr};
use rrvm::RiscvNode;
use std::fmt;
use utils::SysycError;

type Node = Rc<RefCell<InstrNode>>;
#[derive(Clone)]
pub struct InstrNode {
	pub id: usize,
	pub in_deg: usize,
	pub instr: RiscvInstr,
	pub succ: Vec<Node>,
	pub last_use: usize,
}
impl InstrNode {
	pub fn new(instr: &RiscvInstr, id: usize) -> Self {
		Self {
			id,
			in_deg: 0,
			instr: instr.clone(),
			succ: Vec::new(),
			last_use: 0,
		}
	}
}

#[derive(Clone)]
pub struct InstrDag {
	pub nodes: Vec<Node>,
}
impl InstrDag {
	pub fn new(node: &RiscvNode) -> Result<Self, SysycError> {
		let mut nodes: Vec<Node> = Vec::new();
		let mut defs:HashMap<RiscvTemp, Rc<RefCell<InstrNode>>> = HashMap::new();
		let mut uses = HashMap::new();
		let mut last_call: Option<Node> = None;
		let mut last_loads: Vec<Node> = Vec::new();
		let mut last_uses = HashMap::new();
		for (idx, instr) in node.borrow().instrs.iter().rev().enumerate() {
			// println!("instr id:{} {}",instr, idx);
			let node = Rc::new(RefCell::new(InstrNode::new(instr, idx)));
			let mut instr_node_succ = Vec::new();
			let instructions_write = instr.get_riscv_write().clone();
			for instr_write in instructions_write {
				instr_node_succ.extend(
					uses.get(&instr_write).unwrap_or(&Vec::new()).iter().cloned(),
				);
				// println!("in instr {} write extending..",node.borrow().id);
				// for i in uses.get(&instr_write).unwrap_or(&Vec::<Rc<RefCell<InstrNode>>>::new()).iter().map(|z| z.borrow().id).collect::<Vec<_>>() {
				// 	println!("id: {}", i);
				// }
				uses.remove(&instr_write);
			}
			let instr_read = instr.get_riscv_read().clone();
			for instr_read_temp in instr_read.iter() {
				if let Some(def_instr) = defs.get(instr_read_temp) {
					instr_node_succ.push(def_instr.clone());
				//	println!("in instr def extending {}->{}",node.borrow().id,def_instr.borrow().id);
				}
				uses.entry(*instr_read_temp).or_default().push(node.clone());
				if !last_uses.contains_key(instr_read_temp) {
					last_uses.insert(*instr_read_temp, idx);
				}
			}
			// 处理 load call store 指令的依赖关系
			if instr.is_call() {
				instr_node_succ.extend(last_loads.iter().cloned());
			//	println!("in is_call {} extending loads {:?}",node.borrow().id,last_loads.iter().map(|x| x.borrow().id).collect::<Vec<_>>());
				last_loads.clear();
				last_call = Some(node.clone());
			} else if instr.is_load().unwrap_or(false) {
				if let Some(last_call) = last_call.clone() {
				//	println!("in is_load {} extending last_call {}",node.borrow().id,last_call.borrow().id);
					instr_node_succ.push(last_call);
				}
				last_loads.push(node.clone());
				last_call = None;
			} else if instr.is_store().unwrap_or(false) {
				instr_node_succ.extend(last_loads.iter().cloned());
				last_loads.clear();
				last_call = Some(node.clone());
			}
			node.borrow_mut().succ = instr_node_succ;
			nodes.push(node);
		}
		for node in nodes.iter() {
			// println!("node id: {}", node.borrow().id);
			// println!("node successors: {:?}", node.borrow().succ.iter().map(|s| s.borrow().id).collect::<Vec<usize>>());
			// println!("---------");
			for succ in node.borrow().succ.iter() {
				succ.borrow_mut().in_deg += 1;
			}
		}
		for (index, instr) in nodes.iter_mut().enumerate().rev() {
			instr.borrow_mut().last_use +=
				last_uses.iter().filter(|x| *x.1 == index).count();
		}
		Ok(Self { nodes })
	}
}
impl fmt::Display for InstrDag {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		for node in &self.nodes {
			let instr_node = node.borrow();
			writeln!(f, "Node ID: {}", instr_node.id)?;
			writeln!(f, "In-degree: {}", instr_node.in_deg)?;
			writeln!(f, "Instruction: {}", instr_node.instr)?;
			writeln!(
				f,
				"Successors: {:?}",
				instr_node.succ.iter().map(|x| x.borrow().id).collect::<Vec<usize>>()
			)?;
			// print successor's in degrees
			writeln!(
				f,
				"Successors' In-degree: {:?}",
				instr_node
					.succ
					.iter()
					.map(|x| x.borrow().in_deg)
					.collect::<Vec<usize>>()
			)?;
			writeln!(f, "Last Use: {}", instr_node.last_use)?;
			writeln!(f, "---------------------------")?;
		}
		Ok(())
	}
}
