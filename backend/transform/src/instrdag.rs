use std::{cell::RefCell, collections::HashMap, rc::Rc};

use instruction::riscv::{
	reg::RiscvReg::A0, riscvinstr::RiscvInstrTrait, value::RiscvTemp, RiscvInstr,
};
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
	pub call_related: Vec<Vec<Box<dyn RiscvInstrTrait>>>,
}
fn preprocess_call(
	node: &RiscvNode,
	call_related: &mut Vec<Vec<Box<dyn RiscvInstrTrait>>>,
	call_read_vec: &mut Vec<Vec<RiscvTemp>>,
	call_write_vec: &mut Vec<Vec<RiscvTemp>>,
) -> Vec<Box<dyn RiscvInstrTrait>> {
	let mut instrs = Vec::new();
	let mut save_instr = false;
	let mut my_call_related = Vec::new();
	let mut is_last_restore = false;
	let mut call_read = Vec::new();
	let mut call_write = Vec::new();
	for i in node.borrow().instrs.iter() {
		if is_last_restore {
			is_last_restore = false;
			if i.get_riscv_read().len() == 1 {
				if let RiscvTemp::PhysReg(A0) = i.get_riscv_read()[0] {
					my_call_related.push(i.clone());
					call_read.append(&mut i.get_riscv_read().clone());
					call_write.append(&mut i.get_riscv_write().clone());
					call_read_vec.push(call_read);
					call_write_vec.push(call_write);
					call_read = Vec::new();
					call_write = Vec::new();
					call_related.push(my_call_related);
					my_call_related = Vec::new();
					continue;
				}
			}
			call_read_vec.push(call_read);
			call_write_vec.push(call_write);
			call_read = Vec::new();
			call_write = Vec::new();
			call_related.push(my_call_related);
			my_call_related = Vec::new();
		}
		if i.is_save() {
			save_instr = true;
			my_call_related.push(i.clone());
		} else if i.is_restore() {
			save_instr = false;
			my_call_related.push(i.clone());
			is_last_restore = true;
		} else if i.is_call() {
			instrs.push(i.clone());
			my_call_related.push(i.clone());
		} else if save_instr {
			call_read.append(&mut i.get_riscv_read().clone());
			call_write.append(&mut i.get_riscv_write().clone());
			my_call_related.push(i.clone());
		} else {
			instrs.push(i.clone());
		}
	}
	instrs
}
pub fn postprocess_call(
	instrs: Vec<Box<dyn RiscvInstrTrait>>,
	call_related: &mut Vec<Vec<Box<dyn RiscvInstrTrait>>>,
) -> Vec<Box<dyn RiscvInstrTrait>> {
	let mut my_instrs = Vec::new();
	for i in instrs {
		if i.is_call() {
			my_instrs.append(&mut call_related.remove(0));
		} else {
			my_instrs.push(i);
		}
	}
	my_instrs
}
impl InstrDag {
	pub fn new(node: &RiscvNode) -> Result<Self, SysycError> {
		let mut nodes: Vec<Node> = Vec::new();
		let mut defs: HashMap<RiscvTemp, Rc<RefCell<InstrNode>>> = HashMap::new();
		let mut uses = HashMap::new();
		let mut last_call: Option<Node> = None;
		let mut last_loads: Vec<Node> = Vec::new();
		let mut call_related = Vec::new();
		let mut last_uses = HashMap::new();
		let mut call_read_vec = Vec::new();
		let mut call_write_vec = Vec::new();
		// preprocessing call related: 把 call 前后的 从 save 到 restore 的若干条指令保存在 call_related 里面,然后加入到 is_filtered_idx 之后遍历instrs 的时候遇到就直接continue
		let mut processed_instrs = preprocess_call(
			node,
			&mut call_related,
			&mut call_read_vec,
			&mut call_write_vec,
		);
		// println!(" after processed_instrs: {:?} {:?}",call_read_vec,call_write_vec);
		// println!("call related instructions:");
		// for i in call_related.iter() {
		// 	for j in i.iter() {
		// 		println!("{}", j);
		// 	}println!("----");
		// }
		// println!("---------------------------");
		for (idx, instr) in processed_instrs.iter().rev().enumerate() {
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
				let call_reads = call_read_vec.pop().unwrap_or(Vec::new());
				let call_writes = call_write_vec.pop().unwrap_or(Vec::new());
				for instr_write in call_writes {
					instr_node_succ.extend(
						uses.get(&instr_write).unwrap_or(&Vec::new()).iter().cloned(),
					);
					uses.remove(&instr_write);
				}
				for instr_read_temp in call_reads.iter() {
					if let Some(def_instr) = defs.get(instr_read_temp) {
						instr_node_succ.push(def_instr.clone());
					}
					uses.entry(*instr_read_temp).or_default().push(node.clone());
					if !last_uses.contains_key(instr_read_temp) {
						last_uses.insert(*instr_read_temp, idx);
					}
				}
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
		Ok(Self {
			nodes,
			call_related,
		})
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
