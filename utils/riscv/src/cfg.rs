use crate::basicblock::{BasicBlock, BlockType};
use llvm::llvminstr::{JumpCondInstr, JumpInstr, RetInstr};
use llvm::{func::LlvmFunc, label::Label, llvminstr::LlvmInstr};
use std::any::Any;
use std::collections::{BTreeMap, BTreeSet};
struct FuncCfg {
	label_id: BTreeMap<Label, i32>,
	blocks: Vec<BasicBlock>,
}
impl FuncCfg {
	pub fn switch_phi(func: &mut LlvmFunc, start: usize, end: usize) {
		let mut phi_index = Vec::new();
		for i in start..end {
			if func.body[i].is_phi() {
				phi_index.push(i);
			}
		}
		let mut cnt = if func.body[start].is_label().is_some() {
			start + 1
		} else {
			start
		};
		for i in phi_index {
			let t = func.body.remove(i);
			func.body.insert(cnt, t);
			cnt += 1;
		}
	}
	pub fn new(func: &mut LlvmFunc) -> Self {
		let mut func_cfg = FuncCfg {
			blocks: Vec::new(),
			label_id: BTreeMap::new(),
		};
		//start block creation
		//let mut index=0;
		let mut cur_id = 0;
		let mut prev_index = -1;
		let mut label: Option<Label> = None;
		//一个块一定以jmp,cjump,ret,其他块的label结尾，暂未发现其他情况
		for mut index in 0..func.body.len() as i32 {
			let i = &func.body[index as usize];
			if let Some(la) = i.is_label() {
				//check if continuous block,push continous block
				if prev_index != index - 1 {
					func_cfg.blocks.push(BasicBlock::new(
						label.clone(),
						cur_id,
						prev_index + 1,
						index,
					));
					FuncCfg::switch_phi(func, (prev_index + 1) as usize, index as usize);
					cur_id += 1;
					prev_index = index as i32;
				}
				label = Some(la);
				continue;
			}
			if i.is_seq() == false {
				func_cfg.blocks.push(BasicBlock::new(
					label.clone(),
					cur_id,
					prev_index + 1,
					index,
				));
				FuncCfg::switch_phi(func, (prev_index + 1) as usize, index as usize);
				cur_id += 1;
				if let Some(lab) = label {
					func_cfg.label_id.insert(lab, cur_id);
				}
				prev_index = index as i32;
				label = None;
			}
		}
		//construct graph
		for i in 0..func_cfg.blocks.len() {
			//check block type
			let index = func_cfg.blocks[i].range.1 as usize - 1;
			let box_any: Box<&dyn Any> = Box::new(&func.body[index]);
			if let Some(instr) = box_any.downcast_ref::<JumpInstr>() {
				//add pred and succ
				func_cfg.blocks[i].succ.push(func_cfg.label_id[&instr.target]);
				func_cfg.blocks[func_cfg.label_id[&instr.target] as usize]
					.pred
					.push(i as i32);
				continue;
			}
			if let Some(instr) = box_any.downcast_ref::<JumpCondInstr>() {
				func_cfg.blocks[i].succ.push(func_cfg.label_id[&instr.target_false]);
				func_cfg.blocks[i].succ.push(func_cfg.label_id[&instr.target_true]);
				func_cfg.blocks[func_cfg.label_id[&instr.target_false] as usize]
					.pred
					.push(i as i32);
				func_cfg.blocks[func_cfg.label_id[&instr.target_true] as usize]
					.pred
					.push(i as i32);
				continue;
			}
			if let Some(instr) = box_any.downcast_ref::<RetInstr>() {
			} else if i + 1 < func_cfg.blocks.len() {
				//continous block
				func_cfg.blocks[i].succ.push((i + 1) as i32);
				func_cfg.blocks[i + 1].pred.push(i as i32);
			}
		}
		return func_cfg;
	}
}

