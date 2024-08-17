use std::{
	cell::RefCell,
	cmp::max,
	collections::{HashMap, HashSet},
	rc::Rc,
};

use instruction::riscv::{
	reg::RiscvReg::{Fa0, A0, SP},
	riscvinstr::RiscvInstrTrait,
	value::RiscvTemp,
	RiscvInstr,
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
	pub pred: Vec<Node>,
	pub to_end: usize,
}
impl InstrNode {
	pub fn new(instr: &RiscvInstr, id: usize) -> Self {
		Self {
			id,
			in_deg: 0,
			instr: instr.clone(),
			succ: Vec::new(),
			last_use: 0,
			pred: Vec::new(),
			to_end: 0,
		}
	}
}

#[derive(Clone)]
pub struct InstrDag {
	pub nodes: Vec<Node>,
	pub call_related: HashMap<usize, Vec<Box<dyn RiscvInstrTrait>>>,
	pub branch: Option<Box<dyn RiscvInstrTrait>>,
	pub call_writes: Vec<Option<RiscvTemp>>,
	pub call_reads: Vec<Vec<RiscvTemp>>,
	pub li_ret: Option<Box<dyn RiscvInstrTrait>>,
}
fn preprocess_call(
	node: &RiscvNode,
	call_related: &mut Vec<Vec<Box<dyn RiscvInstrTrait>>>, // 换成一个 hashmap 用建完图之后的 node id 来索引
	call_write: &mut Vec<Option<RiscvTemp>>,
	call_reads: &mut Vec<Vec<RiscvTemp>>,
	li_ret: &mut Option<Box<dyn RiscvInstrTrait>>,
) -> Vec<Box<dyn RiscvInstrTrait>> {
	let mut instrs = Vec::new();
	let mut save_instr = false;
	let mut my_call_related = Vec::new();
	let mut is_last_restore = false;
	let mut push_this = false;
	for (idx, i) in node.borrow().instrs.iter().enumerate() {
		if push_this {
			push_this = false;
			my_call_related.push(i.clone());
			call_write.push(Some(i.get_riscv_write()[0]));
			call_related.push(my_call_related);
			my_call_related = Vec::new();
			continue;
		}
		if is_last_restore {
			is_last_restore = false;
			if i.get_riscv_read().len() == 1 {
				if let RiscvTemp::PhysReg(A0) = i.get_riscv_read()[0] {
					my_call_related.push(i.clone());
					push_this = true;
					continue;
				} else if let RiscvTemp::PhysReg(Fa0) = i.get_riscv_read()[0] {
					my_call_related.push(i.clone());
					push_this = true;
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
				break;
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
	// process call writes and call reads
	for call_instrs in call_related.iter() {
		// 获取所有 instr 中的riscv_reads 的并集
		let mut riscv_reads = HashSet::new();
		// 先把 SP 扔进 riscv_reads
		riscv_reads.insert(RiscvTemp::PhysReg(SP));
		for instr in call_instrs.iter() {
			riscv_reads.extend(instr.get_riscv_read().iter().cloned());
		}
		// 在 riscv_read 中删除 call 指令前传 param 的时候写的寄存器
		for instr in call_instrs.iter() {
			if instr.is_call() {
				break;
			}
			for i in instr.get_riscv_write().iter() {
				riscv_reads.remove(i);
			}
		}
		call_reads.push(riscv_reads.iter().cloned().collect());
	}
	// 判断最后一条
	if !instrs.is_empty() {
		let last_instr = instrs.pop().unwrap();
		if node.borrow().succ.is_empty()
			&& last_instr.get_riscv_write().len() == 1
			&& (last_instr.get_riscv_write()[0] == RiscvTemp::PhysReg(A0)
				|| last_instr.get_riscv_write()[0] == RiscvTemp::PhysReg(Fa0)
				|| if let RiscvTemp::VirtReg(t) = &last_instr.get_riscv_write()[0] {
					if let Some(pre) = t.pre_color {
						(pre == A0) || (pre == Fa0)
					} else {
						false
					}
				} else {
					false
				}) {
			*li_ret = Some(last_instr);
		} else {
			instrs.push(last_instr);
		}
	}
	instrs
}
pub fn postprocess_call(
	instrs: Vec<Box<dyn RiscvInstrTrait>>,
	call_related: &mut HashMap<usize, Vec<Box<dyn RiscvInstrTrait>>>,
	branch_related: Option<Box<dyn RiscvInstrTrait>>,
	call_idxs: &mut Vec<usize>,
	li_ret: Option<Box<dyn RiscvInstrTrait>>,
) -> Vec<Box<dyn RiscvInstrTrait>> {
	let mut my_instrs = Vec::new();
	for i in instrs {
		if i.is_call() {
			my_instrs.append(
				&mut call_related.get(&call_idxs.pop().unwrap()).unwrap().clone(),
			);
		} else {
			my_instrs.push(i);
		}
	}
	if let Some(instr) = li_ret {
		my_instrs.push(instr);
	}
	if let Some(instr) = branch_related {
		my_instrs.push(instr);
	}
	my_instrs
}

impl InstrDag {
	pub fn new(node: &RiscvNode) -> Result<Self, SysycError> {
		let mut nodes: Vec<Node> = Vec::new();
		let mut defs: HashMap<RiscvTemp, Rc<RefCell<InstrNode>>> = HashMap::new();
		let mut uses: HashMap<RiscvTemp, Vec<Rc<RefCell<InstrNode>>>> =
			HashMap::new();
		let mut last_call: Option<Node> = None;
		let mut last_loads: Vec<Node> = Vec::new();
		let mut call_related = Vec::new();
		let mut last_uses = HashMap::new();
		let mut last_branch: Option<Box<dyn RiscvInstrTrait>> = None;
		let mut call_write = Vec::new();
		let mut call_reads = Vec::new();
		let mut li_ret = None;
		let mut call_related_map = HashMap::new();
		let mut call_instrs: Vec<Rc<RefCell<InstrNode>>> = Vec::new();
		let mut my_call_write = None;
		// preprocessing call related: 把 call 前后的 从 save 到 restore 的若干条指令保存在 call_related 里面,然后加入到 is_filtered_idx 之后遍历instrs 的时候遇到就直接continue
		let mut processed_instrs = preprocess_call(
			node,
			&mut call_related,
			&mut call_write,
			&mut call_reads,
			&mut li_ret,
		);
		let ret_call_writes = call_write.clone();
		let ret_call_reads = call_reads.clone();
		if !processed_instrs.is_empty() {
			let last_instr = processed_instrs.last().unwrap();
			if last_instr.is_branch() {
				last_branch = Some(last_instr.clone());
				let _ = processed_instrs.pop();
			}
		}
		// 传参 call 回去 param read 会需要记录
		for i in call_related.iter() {
			let mut riscv_writes = HashSet::new();
			let mut riscv_reads = HashSet::new();
			for j in i.iter() {
				riscv_writes.extend(j.get_riscv_write().iter().cloned());
				riscv_reads.extend(j.get_riscv_read().iter().cloned());
			}
		}
		for (idx, instr) in processed_instrs.iter().rev().enumerate() {
			let node = Rc::new(RefCell::new(InstrNode::new(instr, idx)));
			let mut instr_node_succ = Vec::new();
			let instructions_write = instr.get_riscv_write().clone();
			if !instr.is_call() {
				for instr_write in instructions_write {
					instr_node_succ.extend(
						uses.get(&instr_write).unwrap_or(&Vec::new()).iter().cloned(),
					);
					// 同时 extend predecessors
					for i in uses.get(&instr_write).unwrap_or(&Vec::new()).iter() {
						i.borrow_mut().pred.push(node.clone());
					}
					uses.remove(&instr_write);
				}
			} else {
				let tmp = call_write.pop().unwrap();
				my_call_write.clone_from(&tmp);
				if let Some(tmp) = tmp {
					instr_node_succ
						.extend(uses.get(&tmp).unwrap_or(&Vec::new()).iter().cloned());
					uses.get(&tmp).unwrap_or(&Vec::new()).iter().for_each(|x| {
						x.borrow_mut().pred.push(node.clone());
					});
					uses.remove(&tmp);
				}
			}
			let instr_read = instr.get_riscv_read().clone();
			if !instr.is_call() {
				for instr_read_temp in instr_read.iter() {
					if let Some(def_instr) = defs.get(instr_read_temp) {
						instr_node_succ.push(def_instr.clone());
						def_instr.borrow_mut().pred.push(node.clone());
						//	println!("in instr def extending {}->{}",node.borrow().id,def_instr.borrow().id);
					}
					uses.entry(*instr_read_temp).or_default().push(node.clone());
					if !last_uses.contains_key(instr_read_temp) {
						last_uses.insert(*instr_read_temp, idx);
					}
				}
			} else {
				let tmp = call_reads.pop().unwrap();
				for instr_read_temp in tmp.iter() {
					if let Some(def_instr) = defs.get(instr_read_temp) {
						instr_node_succ.push(def_instr.clone());
						def_instr.borrow_mut().pred.push(node.clone());
					}
					uses.entry(*instr_read_temp).or_default().push(node.clone());
					if !last_uses.contains_key(instr_read_temp) {
						last_uses.insert(*instr_read_temp, idx);
					}
				}
			}
			// init defs
			if !instr.is_call() {
				let instructions_write = instr.get_riscv_write().clone();
				for instr_write in instructions_write.iter() {
					defs.insert(*instr_write, node.clone());
				}
			} else if let Some(tmp) = my_call_write {
				defs.insert(tmp, node.clone());
			}

			// 处理 load call store 指令的依赖关系
			if instr.is_call() {
				// 先考虑一下那个最后一条 mov other reg a0
				instr_node_succ.extend(last_loads.iter().cloned());
				last_loads.iter().for_each(|x| {
					x.borrow_mut().pred.push(node.clone());
				});
				last_loads.clear();
				last_call = Some(node.clone());
				for i in call_instrs.iter() {
					instr_node_succ.push(i.clone());
					i.borrow_mut().pred.push(node.clone());
				}
				call_instrs.push(node.clone());
			} else if instr.is_load() {
				if let Some(last_call) = last_call.clone() {
					instr_node_succ.push(last_call.clone());
					last_call.borrow_mut().pred.push(node.clone());
				}
				last_loads.push(node.clone());
			} else if instr.is_store() {
				instr_node_succ.extend(last_loads.iter().cloned());
				last_loads.iter().for_each(|x| {
					x.borrow_mut().pred.push(node.clone());
				});
				last_loads.clear();
				last_call = Some(node.clone());
				for i in call_instrs.iter() {
					instr_node_succ.push(i.clone());
					i.borrow_mut().pred.push(node.clone());
				}
				call_instrs.push(node.clone());
			}
			node.borrow_mut().succ = instr_node_succ;
			nodes.push(node);
		}
		for node in nodes.iter() {
			for succ in node.borrow().succ.iter() {
				succ.borrow_mut().in_deg += 1;
			}
		}
		for (index, instr) in nodes.iter_mut().enumerate().rev() {
			instr.borrow_mut().last_use +=
				last_uses.iter().filter(|x| *x.1 == index).count();
		}
		// construct hashmap,key is the id of the nodes that are call, values are the call instructions
		for (idx, instrs) in nodes.iter().enumerate().rev() {
			if instrs.borrow().instr.is_call() {
				call_related_map.insert(idx, call_related.pop().unwrap());
			}
		}
		Ok(Self {
			nodes,
			call_related: call_related_map,
			branch: last_branch,
			call_reads: ret_call_reads,
			call_writes: ret_call_writes,
			li_ret,
		})
	}
	pub fn assign_nodes(&mut self) {
		// 先备份一遍所有 node 的 indegs
		let indegs =
			self.nodes.iter().map(|x| x.borrow().in_deg).collect::<Vec<_>>();
		// 开始遍历
		let mut stack_ = Vec::new();
		for i in self.nodes.iter() {
			if i.borrow().succ.is_empty() {
				stack_.push(i.clone());
				// get latency
				let siz = i.borrow().instr.get_rtn_array()[4] as usize;
				i.borrow_mut().to_end = siz;
			}
		}
		while let Some(node) = stack_.pop() {
			for i in node.borrow().pred.iter() {
				let new_end = max(
					i.borrow().to_end,
					node.borrow().to_end + i.borrow().instr.get_rtn_array()[4] as usize,
				);
				i.borrow_mut().to_end = new_end;
				i.borrow_mut().in_deg -= 1;
				if i.borrow().in_deg == 0 {
					stack_.push(i.clone());
				}
			}
		}
		// 对每个点恢复 in_deg
		for (i, j) in self.nodes.iter().zip(indegs.iter()) {
			i.borrow_mut().in_deg = *j;
		}
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
