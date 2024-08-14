use std::{cell::RefCell, rc::Rc};

use llvm::{
	ArithInstr, ArithOp, CallInstr, CompInstr, CompKind, CompOp, JumpCondInstr,
	JumpInstr, LlvmInstr, LlvmInstrVariant, LlvmTemp, LlvmTempManager, PhiInstr,
	Value, VarType,
};

use rrvm::{prelude::LlvmBasicBlock, LlvmNode};
use utils::{math::increment, Label};

use crate::loops::loopinfo::LoopInfo;

pub fn make_parallel(
	info: LoopInfo,
	// pre_header: LlvmNode,
	// header: LlvmNode,
	// exit: LlvmNode,
	mgr: &mut LlvmTempManager,
	// cmp: &LlvmTemp,
	// step: Value,
	// begin: Value,
	// end: Value,
	bb_cnt: &mut i32,
	blocks: &mut Vec<LlvmNode>,
) -> (Vec<LlvmNode>, LlvmTemp, LlvmTemp) {
	// SLT only now!!
	let pre_header = info.preheader.clone();
	let header = info.header.clone();
	let exit = info.single_exit.clone();
	// let cmp_op = info.comp_op;
	let cmp = info.cmp.clone();
	let step = info.step.clone();
	let begin = info.begin.clone();
	let end = info.end.clone();

	let weight = header.borrow().weight;
	let hdr_012 =
		Rc::new(RefCell::new(LlvmBasicBlock::new(increment(bb_cnt), weight)));
	let hdr_3 =
		Rc::new(RefCell::new(LlvmBasicBlock::new(increment(bb_cnt), weight)));
	let hdr_all =
		Rc::new(RefCell::new(LlvmBasicBlock::new(increment(bb_cnt), weight)));

	pre_header.borrow_mut().succ = vec![hdr_012.clone(), hdr_3.clone()];
	hdr_012.borrow_mut().prev = vec![pre_header.clone()];
	hdr_3.borrow_mut().prev = vec![pre_header.clone()];
	hdr_012.borrow_mut().succ = vec![hdr_all.clone()];
	hdr_3.borrow_mut().succ = vec![hdr_all.clone()];
	hdr_all.borrow_mut().prev = vec![hdr_012.clone(), hdr_3.clone()];
	hdr_all.borrow_mut().succ = vec![header.clone()];
	header.borrow_mut().prev = vec![hdr_all.clone()];

	let tid = mgr.new_temp(llvm::VarType::I32, false);
	pre_header.borrow_mut().instrs.push(Box::new(CallInstr {
		target: tid.clone(),
		var_type: llvm::VarType::I32,
		func: Label {
			name: "__create_threads".to_string(),
		},
		params: vec![],
	}));

	let is_last = mgr.new_temp(llvm::VarType::I32, false);

	pre_header.borrow_mut().instrs.push(Box::new(CompInstr {
		target: is_last.clone(),
		var_type: llvm::VarType::I32,
		kind: CompKind::Icmp,
		op: CompOp::EQ,
		lhs: Value::Temp(tid.clone()),
		rhs: Value::Int(3),
	}));

	let old_jump =
		pre_header.borrow_mut().jump_instr.replace(Box::new(JumpCondInstr {
			var_type: llvm::VarType::I32,
			cond: Value::Temp(is_last),
			target_true: hdr_3.borrow().label(),
			target_false: hdr_012.borrow().label(),
		}));

	hdr_012.borrow_mut().jump_instr = Some(Box::new(JumpInstr {
		target: hdr_all.borrow().label(),
	}));

	hdr_3.borrow_mut().jump_instr = Some(Box::new(JumpInstr {
		target: hdr_all.borrow().label(),
	}));

	hdr_all.borrow_mut().jump_instr = old_jump;

	let diff = mgr.new_temp(llvm::VarType::I32, false);
	let div_step = mgr.new_temp(llvm::VarType::I32, false);
	let quarter = mgr.new_temp(llvm::VarType::I32, false);
	let quarter_mul_step = mgr.new_temp(llvm::VarType::I32, false);
	let times_tid = mgr.new_temp(llvm::VarType::I32, false);
	let add_quarter = mgr.new_temp(llvm::VarType::I32, false);

	pre_header.borrow_mut().instrs.push(Box::new(ArithInstr {
		target: diff.clone(),
		var_type: llvm::VarType::I32,
		op: ArithOp::Sub,
		lhs: end.clone(),
		rhs: begin.clone(),
	}));

	pre_header.borrow_mut().instrs.push(Box::new(ArithInstr {
		target: div_step.clone(),
		var_type: llvm::VarType::I32,
		op: ArithOp::Div,
		lhs: Value::Temp(diff.clone()),
		rhs: step.clone(),
	}));

	pre_header.borrow_mut().instrs.push(Box::new(ArithInstr {
		target: quarter.clone(),
		var_type: llvm::VarType::I32,
		op: ArithOp::Div,
		lhs: Value::Temp(div_step.clone()),
		rhs: Value::Int(4),
	}));

	pre_header.borrow_mut().instrs.push(Box::new(ArithInstr {
		target: quarter_mul_step.clone(),
		var_type: llvm::VarType::I32,
		op: ArithOp::Mul,
		lhs: Value::Temp(quarter),
		rhs: step.clone(),
	}));

	pre_header.borrow_mut().instrs.push(Box::new(ArithInstr {
		target: times_tid.clone(),
		var_type: llvm::VarType::I32,
		op: ArithOp::Mul,
		lhs: Value::Temp(quarter_mul_step.clone()),
		rhs: Value::Temp(tid.clone()),
	}));

	pre_header.borrow_mut().instrs.push(Box::new(ArithInstr {
		target: add_quarter.clone(),
		var_type: llvm::VarType::I32,
		op: ArithOp::Add,
		lhs: Value::Temp(times_tid.clone()),
		rhs: Value::Temp(quarter_mul_step),
	}));

	let end_thread_offset = mgr.new_temp(llvm::VarType::I32, false);

	hdr_all.borrow_mut().phi_instrs = vec![PhiInstr {
		target: end_thread_offset.clone(),
		var_type: llvm::VarType::I32,
		source: vec![
			(Value::Temp(add_quarter.clone()), hdr_012.borrow().label()),
			(end.clone(), hdr_3.borrow().label()),
		],
	}];

	let begin_thread = mgr.new_temp(llvm::VarType::I32, false);
	let end_thread = mgr.new_temp(llvm::VarType::I32, false);

	hdr_all.borrow_mut().instrs = vec![
		Box::new(ArithInstr {
			target: begin_thread.clone(),
			var_type: llvm::VarType::I32,
			op: ArithOp::Add,
			lhs: Value::Temp(times_tid.clone()),
			rhs: begin.clone(),
		}),
		Box::new(ArithInstr {
			target: end_thread.clone(),
			var_type: llvm::VarType::I32,
			op: ArithOp::Add,
			lhs: Value::Temp(end_thread_offset.clone()),
			rhs: begin,
		}),
	];

	let mut header_phi = std::mem::take(&mut header.borrow_mut().phi_instrs);

	// dbg!(header_phi.len());
	assert!(header_phi.len() == 1);

	header_phi.get_mut(0).unwrap().source.iter_mut().for_each(
		|(value, label)| {
			if *label == pre_header.borrow().label() {
				*label = hdr_all.borrow().label();
				*value = Value::Temp(begin_thread.clone());
			}
		},
	);
	let loop_var = header_phi[0].target.clone();
	header.borrow_mut().phi_instrs = header_phi;

	let new_loop_cmp = Box::new(CompInstr {
		target: cmp.clone(),
		var_type: llvm::VarType::I32,
		kind: CompKind::Icmp,
		op: CompOp::SLT,
		lhs: Value::Temp(loop_var.clone()),
		rhs: Value::Temp(end_thread.clone()),
	});

	let mut found = false;

	let mut new_instrs: Vec<LlvmInstr> = vec![];

	for instr in std::mem::take(&mut header.borrow_mut().instrs).into_iter() {
		match instr.get_variant() {
			LlvmInstrVariant::CompInstr(c) => {
				if c.target == cmp {
					match (c.op, &c.lhs, &c.rhs) {
						(CompOp::SLT, Value::Temp(lhs), _) if *lhs == loop_var => {
							new_instrs.push(new_loop_cmp);
							found = true;
							break;
						}
						(CompOp::SGT, _, Value::Temp(rhs)) if *rhs == loop_var => {
							new_instrs.push(new_loop_cmp);
							found = true;
							break;
						}
						_ => {
							new_instrs.push(instr);
						}
					}
				} else {
					new_instrs.push(instr);
				}
			}
			_ => {
				new_instrs.push(instr);
			}
		}
	}

	assert!(found);
	// TODO 检查跳转情况

	header.borrow_mut().instrs = new_instrs;

	exit.borrow_mut().instrs.insert(
		0,
		Box::new(CallInstr {
			target: mgr.new_temp(VarType::Void, false),
			var_type: VarType::Void,
			func: Label {
				name: "__join_threads".to_string(),
			},
			params: vec![(VarType::I32, Value::Temp(tid))],
		}),
	);

	blocks.push(hdr_012.clone());
	blocks.push(hdr_3.clone());
	blocks.push(hdr_all.clone());

	(
		vec![hdr_012, hdr_3, hdr_all],
		begin_thread.clone(),
		end_thread.clone(),
	)
}
