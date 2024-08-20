use super::{compute_graph::GraphOp, ArithSimplify};
use crate::{
	arith::compute_graph::{remove_addicative_common, GraphValue, Single},
	RrvmOptimizer,
};
use llvm::{
	ArithInstr, CompInstr, LlvmInstr, LlvmInstrVariant, LlvmTemp,
	LlvmTempManager, Value, VarType,
};
use rrvm::{program::LlvmProgram, LlvmNode};
use std::{
	collections::{BTreeMap, HashMap, HashSet, VecDeque},
	usize,
};
use utils::{errors::Result, SysycError::OptimizerError};

fn extract_const(value: GraphValue) -> (bool, i32, GraphValue) {
	match value {
		GraphValue::NonTrival((GraphOp::Mul, v)) => {
			let mut new_v = vec![];
			let mut const_part = 1i32;
			for item in v {
				if let Some(i) = item.as_number() {
					const_part *= i;
				} else {
					new_v.push(item);
				}
			}

			if new_v.is_empty() {
				(true, 1i32, GraphValue::Single(Single::Int(const_part)))
			} else if const_part > 0 {
				(
					true,
					const_part,
					GraphValue::NonTrival((GraphOp::Mul, new_v)),
				)
			} else if const_part < 0 {
				(
					false,
					-const_part,
					GraphValue::NonTrival((GraphOp::Mul, new_v)),
				)
			} else {
				(true, 1i32, GraphValue::Single(Single::Int(0)))
			}
		}
		_ => (true, 1i32, value),
	}
}

fn add_up_value(
	values: &mut Vec<Value>,
	tmp_mgr: &mut LlvmTempManager,
	output: &mut Vec<LlvmInstr>,
	op: GraphOp,
) {
	while values.len() >= 2 {
		let mut new = std::mem::take(values).into_iter();
		while let Some(first) = new.next() {
			if let Some(second) = new.next() {
				let target = tmp_mgr.new_temp(VarType::I32, false);
				output.push(Box::new(ArithInstr {
					target: target.clone(),
					op: match op {
						GraphOp::Mul => llvm::ArithOp::Mul,
						GraphOp::Plus => llvm::ArithOp::Add,
					},
					var_type: VarType::I32,
					lhs: first,
					rhs: second,
				}));

				values.push(Value::Temp(target))
			} else {
				values.push(first);
			}
		}
	}
}

fn time_const_for_value(
	values: Vec<(i32, Value)>,
	tmp_mgr: &mut LlvmTempManager,
	output: &mut Vec<LlvmInstr>,
) -> Vec<Value> {
	values
		.into_iter()
		.map(|(time, value)| {
			if time == 1 {
				value
			} else {
				let target = tmp_mgr.new_temp(VarType::I32, false);
				output.push(Box::new(ArithInstr {
					target: target.clone(),
					op: llvm::ArithOp::Mul,
					var_type: VarType::I32,
					lhs: value,
					rhs: time.into(),
				}));
				Value::Temp(target)
			}
		})
		.collect()
}

fn add_const_rewirte(
	v: Vec<GraphValue>,
	tmp_mgr: &mut LlvmTempManager,
	output: &mut Vec<LlvmInstr>,
) -> Vec<GraphValue> {
	if v.len() < 2 {
		return v;
	}
	let mut parts: BTreeMap<i32, (Vec<GraphValue>, Vec<GraphValue>)> =
		BTreeMap::new();

	fn init_part() -> (Vec<GraphValue>, Vec<GraphValue>) {
		(vec![], vec![])
	}

	for (pos, times, value) in v.into_iter().map(extract_const) {
		if pos {
			parts.entry(times).or_insert_with(init_part).0.push(value);
		} else {
			parts.entry(times).or_insert_with(init_part).1.push(value);
		}
	}

	let mut solved_pos_parts: Vec<(i32, Value)> = vec![];
	let mut solved_neg_parts: Vec<(i32, Value)> = vec![];

	for (times, (pos_part, neg_part)) in parts {
		assert!(times >= 1);
		let mut pos_values: Vec<Value> = pos_part
			.into_iter()
			.map(|graph_value| build_providing_value(graph_value, tmp_mgr, output))
			.collect();

		add_up_value(&mut pos_values, tmp_mgr, output, GraphOp::Plus);

		assert!(pos_values.len() <= 1);

		let mut neg_values: Vec<Value> = neg_part
			.into_iter()
			.map(|graph_value| build_providing_value(graph_value, tmp_mgr, output))
			.collect();

		add_up_value(&mut neg_values, tmp_mgr, output, GraphOp::Plus);
		assert!(neg_values.len() <= 1);

		match (pos_values.pop(), neg_values.pop()) {
			(None, None) => {}
			(None, Some(neg)) => solved_neg_parts.push((times, neg)),
			(Some(pos), None) => solved_pos_parts.push((times, pos)),
			(Some(pos), Some(neg)) => {
				let target = tmp_mgr.new_temp(VarType::I32, false);
				output.push(Box::new(ArithInstr {
					target: target.clone(),
					op: llvm::ArithOp::Sub,
					var_type: VarType::I32,
					lhs: pos,
					rhs: neg,
				}));
				solved_pos_parts.push((times, Value::Temp(target)))
			}
		}
	}

	let mut pos_values = time_const_for_value(solved_pos_parts, tmp_mgr, output);
	let mut neg_values = time_const_for_value(solved_neg_parts, tmp_mgr, output);

	add_up_value(&mut neg_values, tmp_mgr, output, GraphOp::Plus);

	if let Some(neg) = neg_values.pop() {
		let pos = pos_values.pop().unwrap_or(0.into());
		let target = tmp_mgr.new_temp(VarType::I32, false);
		output.push(Box::new(ArithInstr {
			target: target.clone(),
			op: llvm::ArithOp::Sub,
			var_type: VarType::I32,
			lhs: pos,
			rhs: neg,
		}));
		pos_values.push(Value::Temp(target));
	}

	assert!(neg_values.is_empty());

	add_up_value(&mut pos_values, tmp_mgr, output, GraphOp::Plus);
	assert!(pos_values.len() <= 1);

	if let Some(result) = pos_values.pop() {
		vec![GraphValue::from_value(result).unwrap()]
	} else {
		vec![GraphValue::Single(Single::Int(0))]
	}
}

fn build_providing_value(
	value: GraphValue,
	tmp_mgr: &mut LlvmTempManager,
	output: &mut Vec<LlvmInstr>,
) -> Value {
	match value {
		GraphValue::Single(Single::Int(i)) => Value::Int(i),
		GraphValue::Single(Single::Temp(t)) => Value::Temp(t),
		GraphValue::NonTrival((op, v)) => {
			let mut v = match op {
				GraphOp::Plus => add_const_rewirte(v, tmp_mgr, output),
				GraphOp::Mul => v,
			};
			// let mut values: Vec<Value> = v
			// 	.into_iter()
			// 	.map(|graph_value| build_providing_value(graph_value, tmp_mgr, output))
			// 	.collect();
			let mut values = vec![];
			v.sort();
			let mut v = v.into_iter().peekable();

			while let Some(current_value) = v.next() {
				let mut cnt = 0;
				while let Some(next) = v.peek() {
					if *next == current_value {
						v.next();
						cnt += 1;
					} else {
						break;
					}
				}
				let build_value = build_providing_value(current_value, tmp_mgr, output);

				for _ in 0..cnt {
					values.push(build_value.clone());
				}
				values.push(build_value);
			}

			add_up_value(&mut values, tmp_mgr, output, op);

			if values.len() == 1 {
				values.pop().unwrap()
			} else {
				Value::Int(0)
			}
		}
	}
}

fn build_instr(
	value: GraphValue,
	tmp_mgr: &mut LlvmTempManager,
	output: &mut Vec<LlvmInstr>,
	target: LlvmTemp,
) {
	match build_providing_value(value, tmp_mgr, output) {
		Value::Int(i) => {
			output.push(Box::new(ArithInstr {
				target,
				op: llvm::ArithOp::Add,

				var_type: VarType::I32,
				lhs: i.into(),
				rhs: 0.into(),
			}));
		}
		Value::Float(_) => unreachable!(),
		Value::Temp(t) => {
			if output.last().is_some_and(|last| {
				matches!(last.get_variant(), LlvmInstrVariant::ArithInstr(_))
					&& last.get_write() == Some(t.clone())
			}) {
				output.last_mut().unwrap().set_target(target);
			} else {
				output.push(Box::new(ArithInstr {
					target,
					op: llvm::ArithOp::Add,
					var_type: VarType::I32,
					lhs: Value::Temp(t),
					rhs: 0.into(),
				}));
			}
		}
	}
}

fn sovle_bb(
	bb: &mut LlvmNode,
	llvm_temp_manager: &mut LlvmTempManager,
) -> Result<bool, (String, String)> {
	let mut in_degrees: HashMap<LlvmTemp, usize> = HashMap::new();
	let mut covered_degrees: HashMap<LlvmTemp, usize> = HashMap::new();
	let mut graph_values: HashMap<LlvmTemp, GraphValue> = HashMap::new();
	let mut killed_temp: HashSet<LlvmTemp> = HashSet::new();

	fn add_one(temp: &LlvmTemp, map: &mut HashMap<LlvmTemp, usize>) -> usize {
		if let Some(old_value) = map.get_mut(temp) {
			*old_value += 1;
			*old_value
		} else {
			map.insert(temp.clone(), 1);
			1
		}
	}

	fn set_zero(map: &mut HashMap<LlvmTemp, usize>) {
		for (_, value) in map.iter_mut() {
			*value = 0;
		}
	}

	for item in bb.borrow().live_out.iter() {
		add_one(item, &mut in_degrees);
	}

	for instr in bb.borrow().instrs.iter().chain(bb.borrow().jump_instr.iter()) {
		for uses in instr.get_read() {
			add_one(&uses, &mut in_degrees);
		}
	}

	let mut uses: HashMap<LlvmTemp, Vec<LlvmTemp>> = HashMap::new();

	// (lhs, rhs, who uses this division)
	let mut pending_division: HashMap<LlvmTemp, (GraphValue, GraphValue)> =
		HashMap::new();

	for instr in bb.borrow().instrs.iter() {
		if let LlvmInstrVariant::ArithInstr(arith) = instr.get_variant() {
			if arith.target.var_type != VarType::I32 {
				continue;
			}

			if let (Some(left), Some(right)) = (
				GraphValue::from_value(arith.lhs.clone()),
				GraphValue::from_value(arith.rhs.clone()),
			) {
				let value_result = match arith.op {
					llvm::ArithOp::Add => left.add(&right),
					llvm::ArithOp::Sub => left.sub(&right),
					llvm::ArithOp::Div => {
						pending_division.insert(arith.target.clone(), (left, right));
						None
					}
					llvm::ArithOp::Mul => left.mul(&right),
					_ => {
						continue;
					}
				};
				if value_result.is_none() {
					continue;
				}
				graph_values.insert(arith.target.clone(), value_result.unwrap());
				uses.insert(
					arith.target.clone(),
					arith
						.lhs
						.unwrap_temp()
						.into_iter()
						.chain(arith.rhs.unwrap_temp().into_iter())
						.collect(),
				);
			}
		}
	}

	for target in
		bb.borrow().instrs.iter().rev().filter_map(|instr| instr.get_write())
	{
		// dbg!(&target);
		set_zero(&mut covered_degrees);
		let mut work_queue = VecDeque::new();

		for item in uses.get(&target).iter().flat_map(|v| v.iter()) {
			work_queue.push_back(item.clone());
		}

		if let Some((target, mut current_value)) =
			graph_values.remove_entry(&target)
		{
			while let Some(visiting) = work_queue.pop_front() {
				let visit_count = add_one(&visiting, &mut covered_degrees);
				// eprintln!("{:?} {}", &visiting, visit_count);

				if in_degrees
					.get(&visiting)
					.is_some_and(|&total_in_degree| total_in_degree == visit_count)
				{
					if let Some((sub_tmp, sub_value)) =
						graph_values.remove_entry(&visiting)
					{
						current_value = if let Some(success_substitute) =
							current_value.substitude_checked(&visiting, &sub_value)
						{
							for item in uses.get(&visiting).iter().flat_map(|v| v.iter()) {
								work_queue.push_back(item.clone());
							}
							killed_temp.insert(sub_tmp);
							success_substitute
						} else {
							graph_values.insert(sub_tmp, sub_value);
							current_value
						}
					}
				}
			}

			graph_values.insert(target, current_value);
		}
	}

	let mut changed = HashSet::new();
	// dbg!(&graph_values);

	for (div_tmp, (mut lhs, mut rhs)) in pending_division.into_iter() {
		if let Some(lhs_tmp) = lhs.as_tmp() {
			if let Some(lhs_value) = graph_values.get(lhs_tmp) {
				let value = lhs_value.clone();
				lhs = value;
			}
		}
		if let Some(rhs_tmp) = rhs.as_tmp() {
			if let Some(rhs_value) = graph_values.get(rhs_tmp) {
				let value = rhs_value.clone();
				rhs = value;
			}
		}

		if let Some(result) = lhs.div(&rhs) {
			// dbg!(&lhs, &rhs, &result);
			graph_values.insert(div_tmp.clone(), result);
			changed.insert(div_tmp);
		}
	}

	for instr in &bb.borrow().instrs {
		if let LlvmInstrVariant::ArithInstr(arith) = instr.get_variant() {
			if let Some((current, mut graphvalue)) =
				graph_values.remove_entry(&arith.target)
			{
				let mut related_changed_value = HashSet::new();
				graphvalue.contains_temp(&changed, &mut related_changed_value);
				let mut add_this = false;
				for related_changed in related_changed_value {
					if let Some(sub_value) = graph_values.get(&related_changed) {
						graphvalue = if let Some(new_value) =
							graphvalue.substitude_checked(&related_changed, sub_value)
						{
							add_this = true;
							new_value
						} else {
							graphvalue
						}
					}
				}
				graph_values.insert(current, graphvalue);
				if add_this {
					changed.insert(arith.target.clone());
				}
			}
		}
	}

	// dbg!(&graph_values);
	// dbg!(&killed_temp);

	let mut graph_value_back: HashMap<LlvmTemp, GraphValue> = HashMap::new();

	let mut new_instr = vec![];

	let old_size = bb.borrow().instrs.len();

	for instr in std::mem::take(&mut bb.borrow_mut().instrs) {
		if let LlvmInstrVariant::ArithInstr(arith) = instr.get_variant() {
			if killed_temp.contains(&arith.target) {
				continue;
			} else if let Some((target, value)) =
				graph_values.remove_entry(&arith.target)
			{
				graph_value_back.insert(target.clone(), value.clone());
				build_instr(value, llvm_temp_manager, &mut new_instr, target);
			} else {
				new_instr.push(instr);
			}
		} else {
			new_instr.push(instr);
		}
	}

	let new_size = new_instr.len();

	if old_size < new_size {
		let mut label = bb.borrow().label().to_string();
		label.push(' ');
		label.push_str("become longer after optimization");
		return Err((label, "arith simplify".to_string()));
	} else {
		bb.borrow_mut().instrs = new_instr;
	};

	fn get_graph_value(
		map: &HashMap<LlvmTemp, GraphValue>,
		value: &Value,
	) -> Option<GraphValue> {
		match value {
			Value::Int(i) => Some(GraphValue::Single(Single::Int(*i))),
			Value::Temp(t) if t.var_type == VarType::I32 => map
				.get(t)
				.cloned()
				.or_else(|| Some(GraphValue::Single(Single::Temp(t.clone())))),
			_ => None,
		}
	}

	let mut new_instr = vec![];
	for instr in std::mem::take(&mut bb.borrow_mut().instrs) {
		if let LlvmInstrVariant::CompInstr(arith_instr) = instr.get_variant() {
			let op = arith_instr.op;
			let kind = arith_instr.kind;
			let var_type = arith_instr.var_type;

			let mut done = false;

			if var_type == VarType::I32 {
				if let (Some(lhs), Some(rhs)) = (
					get_graph_value(&graph_value_back, &arith_instr.lhs),
					get_graph_value(&graph_value_back, &arith_instr.rhs),
				) {
					if let Some((lhs_new, rhs_new)) = remove_addicative_common(&lhs, &rhs)
					{
						done = true;
						let new_lhs =
							build_providing_value(lhs_new, llvm_temp_manager, &mut new_instr);
						let new_rhs =
							build_providing_value(rhs_new, llvm_temp_manager, &mut new_instr);
						new_instr.push(Box::new(CompInstr {
							kind,
							target: arith_instr.target.clone(),
							op,
							var_type,
							lhs: new_lhs,
							rhs: new_rhs,
						}));
					}
				}
			}
			if !done {
				new_instr.push(instr);
			}
		} else {
			new_instr.push(instr);
		}
	}

	bb.borrow_mut().instrs = new_instr;

	Ok(false)
}

impl RrvmOptimizer for ArithSimplify {
	fn new() -> Self {
		Self {}
	}

	fn apply(
		self,
		program: &mut LlvmProgram,
		_meta: &mut crate::metadata::MetaData,
	) -> Result<bool> {
		let mut changed = false;
		program.analysis();

		for func in program.funcs.iter_mut() {
			for bb in &mut func.cfg.blocks {
				changed |=
					sovle_bb(bb, &mut program.temp_mgr).map_err(|(bb, pass)| {
						let mut func = func.name.clone();
						func.push(':');
						func.push_str(&bb);
						OptimizerError(pass, func)
					})?;
			}
		}

		Ok(changed)
	}
}
