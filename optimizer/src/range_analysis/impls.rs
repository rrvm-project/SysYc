use std::collections::{HashMap, HashSet, VecDeque};

use super::{
	analysis_graph::solve_graph, constrain::Constrain,
	constrain_graph::ConstrainGraph, range::Range,
	range_compare::comp_must_never, RangeAnalysis,
};
use crate::{
	range_analysis::{
		addictive_synonym::LlvmTempAddictiveSynonym,
		block_imply::{
			add_implication, flip_lnot, general_both, BlockImplyCondition,
		},
		tarjan::Tarjan,
	},
	RrvmOptimizer,
};
use llvm::{
	ArithInstr, ArithOp, CompOp, LlvmInstrTrait, LlvmInstrVariant, LlvmTemp,
	Value, VarType,
};
use rrvm::program::{LlvmFunc, LlvmProgram};
use utils::{errors::Result, from_label};

fn solve_phi_comp_reliance(
	input: &mut HashMap<LlvmTemp, (BlockImplyCondition, bool)>,
) {
	let mut reliance: HashMap<LlvmTemp, HashSet<LlvmTemp>> = HashMap::new();
	let mut reliants: HashMap<LlvmTemp, HashSet<LlvmTemp>> = HashMap::new();

	for item in input.keys() {
		reliance.insert(item.clone(), HashSet::new());
		reliants.insert(item.clone(), HashSet::new());
	}

	for (relied, (_, positive)) in input.iter() {
		for relying in input.keys() {
			if *positive {
				if input.get(relying).unwrap().0.positive.contains(relied) {
					reliance.get_mut(relying).unwrap().insert(relied.clone());
					reliants.get_mut(relied).unwrap().insert(relying.clone());
				}
			} else if input.get(relying).unwrap().0.negative.contains(relied) {
				reliance.get_mut(relying).unwrap().insert(relied.clone());
				reliants.get_mut(relied).unwrap().insert(relying.clone());
			}
		}
	}

	// dbg!(&reliance);
	// dbg!(&input);

	while let Some(sub_temp) =
		reliance.iter().find(|x| x.1.is_empty()).map(|(tmp, _)| tmp.clone())
	{
		if let Some((sub_cond, positive)) = input.remove(&sub_temp) {
			if let Some(reliant) = reliants.get(&sub_temp) {
				for item in reliant {
					if let Some((old_cond, _)) = input.get_mut(item) {
						old_cond.substution(&sub_temp, &sub_cond, positive)
					}
					if let Some(reliance) = reliance.get_mut(item) {
						reliance.remove(&sub_temp);
					}
				}
			}
			input.insert(sub_temp.clone(), (sub_cond, positive));
		}

		reliance.remove_entry(&sub_temp);
	}

	assert_eq!(reliance.len(), 0, "loop relinance in condition tmp!");

	// dbg!(&input);
}

fn process_function(func: &mut LlvmFunc) -> bool {
	func.cfg.analysis();

	let (block_implies_necessary, block_implies, comparisons) =
		extract_constrain(func);
	let (sccs, graph) = build_constrains_graph(
		func,
		block_implies_necessary,
		block_implies,
		comparisons,
	);
	let graph = solve_graph(sccs, graph);


	action(func, graph)
}

#[allow(clippy::type_complexity)]
fn extract_constrain(
	func: &mut LlvmFunc,
) -> (
	HashMap<i32, BlockImplyCondition>,
	HashMap<i32, BlockImplyCondition>,
	HashMap<LlvmTemp, (Value, Value, CompOp, i32)>,
) {
	let mut comparisons = HashMap::new();

	let mut lnot_n = 0;
	let mut lnot_pos_rev: HashMap<usize, LlvmTemp> = HashMap::new();
	let mut lnot_pos: HashMap<LlvmTemp, usize> = HashMap::new();
	let mut lnot_neg: HashMap<LlvmTemp, usize> = HashMap::new();

	let mut land: HashMap<LlvmTemp, (i32, LlvmTemp)> = HashMap::new();
	let mut lor: HashMap<LlvmTemp, (i32, LlvmTemp)> = HashMap::new();

	let mut block_condition: HashMap<
		i32,
		Vec<(i32, Option<LlvmTemp>, Option<LlvmTemp>)>,
	> = HashMap::new();

	let mut block_implies_workset =
		VecDeque::with_capacity(func.cfg.blocks.len());

	let mut id_to_block = HashMap::new();

	for block in func.cfg.blocks.iter() {
		block_implies_workset.push_back(block.borrow().id);
		id_to_block.insert(block.borrow().id, block.clone());

		for phi in block.borrow().phi_instrs.iter() {
			if phi.source.len() != 2 {
				continue;
			}

			for i in 0..=1 {
				if let Some(other_cond) = phi.source[1 - i].0.clone().into() {
					if phi.source[i].0 == llvm::Value::Int(0) {
						land.insert(
							phi.target.clone(),
							(from_label(&phi.source[1 - i].1), other_cond),
						);
					} else if phi.source[i].0 == llvm::Value::Int(1) {
						lor.insert(
							phi.target.clone(),
							(from_label(&phi.source[1 - i].1), other_cond),
						);
					}
				}
			}
		}

		// let v = block.borrow();
		//find all comparisons
		for instr in block.borrow().instrs.iter() {
			if let llvm::LlvmInstrVariant::CompInstr(i) = instr.get_variant() {
				// vec_comparison.push((i.target.clone(),i.lhs.clone(), i.rhs.clone(), i.op.clone(), block.borrow().id));

				if i.lhs == llvm::Value::Int(0) && i.op == llvm::CompOp::EQ {
					if let Some(tmp) = i.rhs.clone().into() {
						if let Some(lnot_id) = lnot_pos.get(&tmp) {
							lnot_neg.insert(i.target.clone(), *lnot_id);
						} else if let Some(lnot_id) = lnot_neg.get(&tmp) {
							lnot_pos.insert(i.target.clone(), *lnot_id);
						} else {
							lnot_n += 1;
							lnot_pos.insert(tmp.clone(), lnot_n);
							lnot_neg.insert(i.target.clone(), lnot_n);
							lnot_pos_rev.insert(lnot_n, tmp);
						}
					}
				}
				comparisons.insert(
					i.target.clone(),
					(i.lhs.clone(), i.rhs.clone(), i.op, block.borrow().id),
				);
			}
		}

		if let Some(instr) = &block.borrow().jump_instr {
			match instr.get_variant() {
				llvm::LlvmInstrVariant::JumpCondInstr(jc) => {
					block_condition
						.entry(from_label(&jc.target_true))
						.or_default()
						.push((block.borrow().id, jc.cond.clone().into(), None));
					block_condition
						.entry(from_label(&jc.target_false))
						.or_default()
						.push((block.borrow().id, None, jc.cond.clone().into()));
				}
				llvm::LlvmInstrVariant::JumpInstr(j) => {
					block_condition
						.entry(from_label(&j.get_label()))
						.or_default()
						.push((block.borrow().id, None, None));
				}
				_ => {}
			}
		}
	}

	let mut block_implies: HashMap<i32, BlockImplyCondition> = HashMap::new();

	while let Some(current) = block_implies_workset.pop_front() {
		let current_entry =
			block_implies.entry(current).or_insert_with(BlockImplyCondition::new);
		let old_size = current_entry.size();

		if let Some(conditions) = block_condition.get(&current) {
			let new_cond = general_both(conditions.iter().map(|(p, pos, neg)| {
				let prev_entry =
					block_implies.entry(*p).or_insert_with(BlockImplyCondition::new);
				add_implication(prev_entry, pos.clone(), neg.clone())
			}));

			if old_size < new_cond.size() {
				id_to_block
					.get(&current)
					.map(|block| {
						block
							.borrow()
							.succ
							.iter()
							.map(|block| block.borrow().id)
							.collect::<Vec<i32>>()
					})
					.map(|successor| {
						successor.into_iter().map(|i| block_implies_workset.push_back(i))
					});
			}

			block_implies.insert(current, new_cond);
		}
	}

	let mut substution: HashMap<LlvmTemp, (BlockImplyCondition, bool)> =
		HashMap::new();

	// only_for_positive
	for (land_target, (block, add_cond)) in &land {
		if let Some(old) = block_implies.get(block) {
			let new = add_implication(old, Some(add_cond.clone()), None);
			substution.insert(land_target.clone(), (new, true));
		} else {
			let empty = BlockImplyCondition::new();
			let new = add_implication(&empty, Some(add_cond.clone()), None);
			substution.insert(land_target.clone(), (new, true));
		}
	}

	// only_for_negative
	for (lor_target, (block, add_cond)) in &lor {
		if let Some(old) = block_implies.get(block) {
			let new = add_implication(old, None, Some(add_cond.clone()));
			substution.insert(lor_target.clone(), (new, false));
		} else {
			let empty = BlockImplyCondition::new();
			let new = add_implication(&empty, None, Some(add_cond.clone()));
			substution.insert(lor_target.clone(), (new, false));
		}
	}

	// dbg!(&substution);

	solve_phi_comp_reliance(&mut substution);

	for cond in block_implies.values_mut() {
		for (temp, (substution, pos)) in &substution {
			cond.substution(temp, substution, *pos);
		}
		flip_lnot(cond, &lnot_pos_rev, &lnot_pos, &lnot_neg);
	}

	let mut block_implies_necessary = HashMap::new();

	for block in func.cfg.blocks.iter() {
		let block = block.borrow();
		let mut imply_necessary = block_implies[&block.id].clone();
		imply_necessary.extract_necessary(
			block
				.get_prev_iter()
				.flat_map(|item| block_implies.get(&item.borrow().id).into_iter()),
		);
		block_implies_necessary.insert(block.id, imply_necessary);
	}

	(block_implies_necessary, block_implies, comparisons)
}

pub fn build_constrains_graph(
	func: &mut LlvmFunc,
	block_implies_necessary: HashMap<i32, BlockImplyCondition>,
	block_implies: HashMap<i32, BlockImplyCondition>,
	comparisons: HashMap<LlvmTemp, (Value, Value, CompOp, i32)>,
) -> (Vec<Vec<usize>>, ConstrainGraph) {
	let mut addicitive_synonym = LlvmTempAddictiveSynonym::new();

	for block in func.cfg.blocks.iter() {
		for instr in &block.borrow().instrs {
			if let llvm::LlvmInstrVariant::ArithInstr(instr) = instr.get_variant() {
				match instr.op {
					llvm::ArithOp::Add => match (&instr.lhs, &instr.rhs) {
						(Value::Int(i), Value::Temp(t)) => {
							addicitive_synonym.insert(t, &instr.target, Value::Int(*i))
						}
						(Value::Temp(t), Value::Int(i)) => {
							addicitive_synonym.insert(t, &instr.target, Value::Int(*i))
						}
						(Value::Float(f), Value::Temp(t)) => {
							addicitive_synonym.insert(t, &instr.target, Value::Float(*f))
						}
						(Value::Temp(t), Value::Float(f)) => {
							addicitive_synonym.insert(t, &instr.target, Value::Float(*f))
						}
						_ => {}
					},
					llvm::ArithOp::Sub => match (&instr.lhs, &instr.rhs) {
						(Value::Temp(t), Value::Int(i)) => {
							addicitive_synonym.insert(t, &instr.target, Value::Int(-*i))
						}
						(Value::Temp(t), Value::Float(f)) => {
							addicitive_synonym.insert(t, &instr.target, Value::Float(-*f))
						}
						_ => {}
					},
					// llvm::ArithOp::Fadd => todo!(), // support float?
					// llvm::ArithOp::Fsub => todo!(),
					_ => {}
				}
			}
		}
	}

	// dbg!(&addicitive_synonym);

	let mut graph = ConstrainGraph::new();


	for block in func.cfg.blocks.iter() {
		let mut processed = HashSet::new();
		let block = block.borrow();
		for phi in &block.phi_instrs {
			graph.handle_phi_instr(phi, block.id);
			processed.insert(phi.target.clone());
		}

		dbg!(&block.id, &block.live_in);

		for instr in &block.instrs {
			
			match instr.get_variant() {
				llvm::LlvmInstrVariant::ArithInstr(arith) => {
					graph.handle_arith_instr(arith, block.id)
				}
				llvm::LlvmInstrVariant::ConvertInstr(convert) => {
					graph.handle_convert_instr(convert, block.id)
				}
				_ => {}
			}
		}

		for tmp in block.live_in.iter() {
			let constrain = Constrain::build(
				tmp,
				&block_implies_necessary[&block.id],
				&block_implies[&block.id],
				&comparisons,
				&addicitive_synonym,
			);
			graph.handle_live_in(
				block.get_prev_iter().map(|b| b.borrow().id),
				tmp.clone(),
				constrain,
				block.id,
			);
		}
	}


	(Tarjan::new(graph.len()).work(&graph), graph)
}

pub fn action(func: &mut LlvmFunc, graph: ConstrainGraph) -> bool {
	let get_range = |value: &Value, basicblockid: i32| match value {
		Value::Int(v) => Range::fromi32(*v),
		Value::Float(v) => Range::fromf32(*v),
		Value::Temp(t) => {
			graph.look_up_tmp_node(t, basicblockid).map_or_else(Range::inf, |node| {
				graph.get_node_ref(node).range.clone().unwrap_or_else(Range::inf)
			})
		}
	};
	let mut changed = false;
	for block in func.cfg.blocks.iter_mut() {
		let id = block.borrow().id;
		for instr in &mut block.borrow_mut().instrs {
			if let LlvmInstrVariant::CompInstr(c) = instr.get_variant() {
				let op = &c.op;

				let new_instr = |t: bool| ArithInstr {
					target: c.target.clone(),
					op: ArithOp::Add,
					var_type: VarType::I32,
					lhs: 0.into(),
					rhs: if t { 1 } else { 0 }.into(),
				};
				if let Some(t) =
					comp_must_never(op, &get_range(&c.lhs, id), &get_range(&c.rhs, id))
				{
				println!(
					"{} {} {} {:?} {:?}",
					c.target,
					id,
					&op,
					&get_range(&c.lhs, id),
					&get_range(&c.rhs, id)
				);

				
					*instr = Box::new(new_instr(t));
					changed = true;
				}
			}
		}
	}
	changed
}

impl RrvmOptimizer for RangeAnalysis {
	fn new() -> Self {
		Self {}
	}

	fn apply(self, program: &mut LlvmProgram) -> Result<bool> {
		Ok(program.funcs.iter_mut().any(process_function))
	}
}
