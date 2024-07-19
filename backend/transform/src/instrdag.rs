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
	pub branch: Option<Box<dyn RiscvInstrTrait>>,
}
fn preprocess_call(
	node: &RiscvNode,
	call_related: &mut Vec<Vec<Box<dyn RiscvInstrTrait>>>,
	call_write: &mut Vec<Option<RiscvTemp>>,
) -> Vec<Box<dyn RiscvInstrTrait>> {
	let mut instrs = Vec::new();
	let mut save_instr = false;
	let mut my_call_related = Vec::new();
	let mut is_last_restore = false;
	for (idx, i) in node.borrow().instrs.iter().enumerate() {
		if is_last_restore {
			is_last_restore = false;
			if i.get_riscv_read().len() == 1 {
				if let RiscvTemp::PhysReg(A0) = i.get_riscv_read()[0] {
					my_call_related.push(i.clone());
					call_related.push(my_call_related);
					my_call_related = Vec::new();
					call_write.push(Some(i.get_riscv_write()[0]));
					continue;
				} else {
					call_write.push(None);
				}
			} else {
				call_write.push(None);
			}
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
			if idx == node.borrow().instrs.len() - 1 {
				call_related.push(my_call_related);
				call_write.push(None);
				return instrs;
			}
		} else if i.is_call() {
			instrs.push(i.clone());
			my_call_related.push(i.clone());
		} else if save_instr {
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
	branch_related: Option<Box<dyn RiscvInstrTrait>>,
) -> Vec<Box<dyn RiscvInstrTrait>> {
	let mut my_instrs = Vec::new();
	for i in instrs {
		if i.is_call() {
			my_instrs.append(&mut call_related.remove(0));
		} else {
			my_instrs.push(i);
		}
	}
	if let Some(instr) = branch_related {
		my_instrs.push(instr);
	}
	// debug print
	// println!("postprocess call instrs:");
	// for i in my_instrs.iter() {
	// 	println!("{}", i);
	// }
	// println!("postprocess call instrs end");
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
		let mut last_branch: Option<Box<dyn RiscvInstrTrait>> = None;
		let mut call_write = Vec::new();
		// preprocessing call related: 把 call 前后的 从 save 到 restore 的若干条指令保存在 call_related 里面,然后加入到 is_filtered_idx 之后遍历instrs 的时候遇到就直接continue
		// println!("original instrs :");
		// for i in node.borrow().instrs.iter() {
		// 	println!("{}", i);
		// }
		let mut processed_instrs =
			preprocess_call(node, &mut call_related, &mut call_write);
		if processed_instrs.len() > 0 {
			let last_instr = processed_instrs.last().unwrap();
			if last_instr.is_branch() {
				last_branch = Some(last_instr.clone());
				let _ = processed_instrs.pop();
			}
		}
		// println!("call related instructions:");
		// for i in call_related.iter() {
		// 	for j in i.iter() {
		// 		println!("{}", j);
		// 	}println!("----");
		// }
		// println!("processed_instrs len: {}",processed_instrs.len());
		// for i in processed_instrs.iter() {
		// 	println!("{}",i);
		// }
		// println!("---------------------------");
		for (idx, instr) in processed_instrs.iter().rev().enumerate() {
			// println!("instr id:{} {}",instr, idx);
			let node = Rc::new(RefCell::new(InstrNode::new(instr, idx)));
			let mut instr_node_succ = Vec::new();
			let instructions_write = instr.get_riscv_write().clone();
			if instr.is_call() == false {
				for instr_write in instructions_write {
					instr_node_succ.extend(
						uses.get(&instr_write).unwrap_or(&Vec::new()).iter().cloned(),
					);
					//  println!("in instr {} write extending..",node.borrow().id);
					//  for i in uses.get(&instr_write).unwrap_or(&Vec::<Rc<RefCell<InstrNode>>>::new()).iter().map(|z| z.borrow().id).collect::<Vec<_>>() {
					//  	println!("intr write extending to id: {}", i);
					//  }
					uses.remove(&instr_write);
				}
			} else {
				let tmp = call_write.pop().unwrap();
				if let Some(tmp) = tmp {
					instr_node_succ
						.extend(uses.get(&tmp).unwrap_or(&Vec::new()).iter().cloned());
					uses.remove(&tmp);
				}
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
				// 先考虑一下那个最后一条 mov other reg a0
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
			//  println!("node id: {}", node.borrow().id);
			//  println!("node successors: {:?}", node.borrow().succ.iter().map(|s| s.borrow().id).collect::<Vec<usize>>());
			//  println!("---------");
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
			branch: last_branch,
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
