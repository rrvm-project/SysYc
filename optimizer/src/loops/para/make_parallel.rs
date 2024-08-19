use std::collections::HashMap;

use llvm::{
	ArithInstr, ArithOp, CallInstr, CompInstr, CompKind, CompOp, GEPInstr,
	LlvmInstr, LlvmInstrVariant, LlvmTemp, LlvmTempManager, PhiInstr, Value,
	VarType,
};

use utils::Label;

use crate::loops::{
	indvar::IndVar, indvar_type::IndVarType, loopinfo::LoopInfo,
};

fn add_arith_instr(
	instr: &mut Vec<LlvmInstr>,
	mgr: &mut LlvmTempManager,
	op: ArithOp,
	lhs: Value, // CAN BE PTR
	rhs: Value,
) -> LlvmTemp {
	let target = mgr.new_temp(VarType::I32, false);

	assert!(
		rhs.is_num()
			|| rhs.unwrap_temp().unwrap().var_type == VarType::I32
			|| rhs.unwrap_temp().unwrap().var_type == VarType::F32
	);

	let i: LlvmInstr = match lhs {
		Value::Int(_) => Box::new(ArithInstr {
			target: target.clone(),
			op,
			var_type: VarType::I32,
			lhs,
			rhs,
		}),
		Value::Temp(t) if t.var_type == VarType::I32 => Box::new(ArithInstr {
			target: target.clone(),
			op,
			var_type: VarType::I32,
			lhs: t.into(),
			rhs,
		}),

		Value::Temp(t) if t.var_type == VarType::I32Ptr => Box::new(GEPInstr {
			target: target.clone(),
			var_type: VarType::I32Ptr,
			addr: t.into(),
			offset: rhs,
		}),
		_ => unreachable!(),
	};

	instr.push(i);
	target
}

fn add_comp_instr(
	instr: &mut Vec<LlvmInstr>,
	mgr: &mut LlvmTempManager,
	op: CompOp,
	lhs: Value,
	rhs: Value,
) -> LlvmTemp {
	let target = mgr.new_temp(VarType::I32, false);

	let i = CompInstr {
		kind: CompKind::Icmp,
		target: target.clone(),
		op,
		var_type: VarType::I32,
		lhs,
		rhs,
	};

	instr.push(Box::new(i));
	target
}

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
	indvars: &mut HashMap<LlvmTemp, IndVar>,
) -> (Value, Value, LlvmTemp) {
	// SLT only now!!
	let pre_header = info.preheader.clone();
	let header = info.header.clone();
	let exit = info.single_exit.clone();
	let cmp_op = info.comp_op;
	let cmp = info.cmp.clone();
	let step = info.step.clone();
	let begin = info.begin.clone();
	let end = info.end.clone();

	let equal_op = match cmp_op {
		CompOp::SGT => false,
		CompOp::SGE => true,
		CompOp::SLT => false,
		CompOp::SLE => true,
		_ => unreachable!(),
	};

	let tid_old = mgr.new_temp(llvm::VarType::I32, false);
	pre_header.borrow_mut().instrs.push(Box::new(CallInstr {
		target: tid_old.clone(),
		var_type: llvm::VarType::I32,
		func: Label {
			name: "__create_threads".to_string(),
		},
		params: vec![],
	}));

	let tid = add_arith_instr(
		pre_header.borrow_mut().instrs.as_mut(),
		mgr,
		ArithOp::Sub,
		3.into(),
		tid_old.into(),
	);

	let is_last = add_comp_instr(
		pre_header.borrow_mut().instrs.as_mut(),
		mgr,
		CompOp::EQ,
		3.into(),
		tid.clone().into(),
	);
	let neg_is_last = add_arith_instr(
		pre_header.borrow_mut().instrs.as_mut(),
		mgr,
		ArithOp::Mul,
		is_last.into(),
		(-1).into(),
	);

	let diff = add_arith_instr(
		pre_header.borrow_mut().instrs.as_mut(),
		mgr,
		ArithOp::Sub,
		end.clone(),
		begin.clone(),
	);
	let diff_div_step = add_arith_instr(
		pre_header.borrow_mut().instrs.as_mut(),
		mgr,
		ArithOp::Div,
		diff.clone().into(),
		step.clone(),
	);

	let total_times = if equal_op {
		add_arith_instr(
			pre_header.borrow_mut().instrs.as_mut(),
			mgr,
			ArithOp::Add,
			diff_div_step.clone().into(),
			1.into(),
		)
	} else {
		let diff_div_step_mul_step = add_arith_instr(
			pre_header.borrow_mut().instrs.as_mut(),
			mgr,
			ArithOp::Mul,
			diff_div_step.clone().into(),
			step.clone(),
		);
		let need_to_add = add_comp_instr(
			pre_header.borrow_mut().instrs.as_mut(),
			mgr,
			CompOp::NE,
			diff_div_step_mul_step.into(),
			diff.clone().into(),
		);
		add_arith_instr(
			pre_header.borrow_mut().instrs.as_mut(),
			mgr,
			ArithOp::Add,
			diff_div_step.clone().into(),
			need_to_add.into(),
		)
	};

	let per_part = add_arith_instr(
		pre_header.borrow_mut().instrs.as_mut(),
		mgr,
		ArithOp::Div,
		total_times.clone().into(),
		4.into(),
	);
	let per_part_4 = add_arith_instr(
		pre_header.borrow_mut().instrs.as_mut(),
		mgr,
		ArithOp::Mul,
		per_part.clone().into(),
		4.into(),
	);
	let remain = add_arith_instr(
		pre_header.borrow_mut().instrs.as_mut(),
		mgr,
		ArithOp::Sub,
		total_times.clone().into(),
		per_part_4.into(),
	);

	let remain_to_add = add_arith_instr(
		pre_header.borrow_mut().instrs.as_mut(),
		mgr,
		ArithOp::And,
		remain.into(),
		neg_is_last.into(),
	);
	let this_length = add_arith_instr(
		pre_header.borrow_mut().instrs.as_mut(),
		mgr,
		ArithOp::Add,
		remain_to_add.into(),
		per_part.clone().into(),
	);

	let offset = add_arith_instr(
		pre_header.borrow_mut().instrs.as_mut(),
		mgr,
		ArithOp::Mul,
		tid.clone().into(),
		per_part.into(),
	);

	let header_phi = std::mem::take(&mut header.borrow_mut().phi_instrs);

	let new_index = mgr.new_temp(VarType::I32, false);
	let next_index = mgr.new_temp(VarType::I32, false);

	let rewrite_phi = |items: &mut Vec<(Value, Label)>, new_value: Value| {
		for item in items.iter_mut() {
			if item.1 == pre_header.borrow().label() {
				item.0 = new_value;
				break;
			}
		}
	};

	for mut item in header_phi {
		if let Some(indvar) = indvars.get(&item.target) {
			if matches!(indvar.get_type(), IndVarType::Ordinary) {
				let start = indvar.base.clone();
				let total_offset = add_arith_instr(
					pre_header.borrow_mut().instrs.as_mut(),
					mgr,
					ArithOp::Mul,
					offset.clone().into(),
					indvar.step.first().unwrap().clone(),
				);
				let thread_start = add_arith_instr(
					pre_header.borrow_mut().instrs.as_mut(),
					mgr,
					ArithOp::Add,
					start,
					total_offset.into(),
				);
				rewrite_phi(&mut item.source, thread_start.into());
				header.borrow_mut().phi_instrs.push(PhiInstr {
					target: item.target,
					var_type: item.var_type,
					source: item.source,
				});
			} else {
				unreachable!()
			}
		//TODO 有后端指令支持之后在这里搞zfp
		} else {
			unreachable!()
		}
	}

	let mut new_phi = header.borrow().phi_instrs.last().unwrap().clone();

	new_phi.target = new_index.clone();
	new_phi.var_type = VarType::I32;
	new_phi.source.iter_mut().for_each(|(value, label)| {
		if *label == pre_header.borrow().label() {
			*value = 0i32.into();
		} else {
			*value = next_index.clone().into();
		}
	});

	header.borrow_mut().phi_instrs.push(new_phi);

	let mut loop_control: Vec<LlvmInstr> = vec![
		Box::new(CompInstr {
			target: cmp.clone(),
			var_type: llvm::VarType::I32,
			kind: CompKind::Icmp,
			op: CompOp::SLT,
			lhs: new_index.clone().into(),
			rhs: this_length.clone().into(),
		}),
		Box::new(ArithInstr {
			target: next_index.clone(),
			var_type: llvm::VarType::I32,
			op: ArithOp::Add,
			lhs: new_index.clone().into(),
			rhs: 1.into(),
		}),
	];

	let mut found = false;

	let mut new_instrs: Vec<LlvmInstr> = vec![];

	for instr in std::mem::take(&mut header.borrow_mut().instrs).into_iter() {
		match instr.get_variant() {
			LlvmInstrVariant::CompInstr(c) => {
				if c.target == cmp {
					assert!(!loop_control.is_empty());
					found = true;
					new_instrs.extend(std::mem::take(&mut loop_control));
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

	(0i32.into(), this_length.clone().into(), new_index)
}
