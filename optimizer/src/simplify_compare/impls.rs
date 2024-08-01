use super::{addictive_synonym::LlvmTempAddictiveSynonym, SimplifyCompare};
use crate::RrvmOptimizer;
use llvm::{CompInstr, LlvmInstr, Value};
use rrvm::program::{LlvmFunc, LlvmProgram};
use utils::errors::Result;

fn solve_function(func: &mut LlvmFunc) -> bool {
	let mut changed = false;

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
					// llvm::ArithOp::Fadd => todo!(), // support float? //TODO 依赖float结合律
					// llvm::ArithOp::Fsub => todo!(),
					_ => {}
				}
			}
		}
	}

	// dbg!(&addicitive_synonym);

	for block in func.cfg.blocks.iter_mut() {
		let mut new_instrs: Vec<LlvmInstr> = vec![];
		std::mem::take(&mut block.borrow_mut().instrs).into_iter().for_each(
			|instr| match instr.get_variant() {
				llvm::LlvmInstrVariant::CompInstr(comp) => {
					let target = instr.get_write().unwrap();
					let var_type = comp.var_type;
					let mut push_result = |result: bool| {
						new_instrs.push(Box::new(llvm::ArithInstr {
							target: target.clone(),
							op: llvm::ArithOp::Add,
							var_type: llvm::VarType::I32,
							lhs: (result as i32).into(),
							rhs: 0.into(),
						}))
					};

					match (comp.lhs.clone(), comp.rhs.clone(), comp.op, comp.kind) {
						(Value::Int(a), Value::Int(b), op, _) => {
							if let Some(result) = match op {
								llvm::CompOp::EQ => Some(a == b),
								llvm::CompOp::NE => Some(a != b),
								llvm::CompOp::SGT => Some(a > b),
								llvm::CompOp::SGE => Some(a >= b),
								llvm::CompOp::SLT => Some(a < b),
								llvm::CompOp::SLE => Some(a <= b),
								_ => None,
							} {
								push_result(result);
								changed = true;
							} else {
								new_instrs.push(instr);
							}
						}
						(Value::Float(a), Value::Float(b), op, _) => {
							if let Some(result) = match op {
								llvm::CompOp::OEQ => Some(a == b),
								llvm::CompOp::ONE => Some(a != b),
								llvm::CompOp::OGT => Some(a > b),
								llvm::CompOp::OGE => Some(a >= b),
								llvm::CompOp::OLT => Some(a < b),
								llvm::CompOp::OLE => Some(a <= b),
								_ => None,
							} {
								push_result(result);
								changed = true;
							} else {
								new_instrs.push(instr);
							}
						}

						(Value::Int(i), Value::Temp(t), op, kind) => {
							if let Some((new_t, Value::Int(offset))) =
								addicitive_synonym.get(&t)
							{
								if let Some(new_i) = i.checked_sub(offset) {
									new_instrs.push(Box::new(CompInstr {
										kind,
										target,
										op,
										var_type,
										lhs: new_i.into(),
										rhs: Value::Temp(new_t),
									}));
									changed = offset != 0;
								} else {
									new_instrs.push(instr);
								}
							} else {
								new_instrs.push(instr);
							}
						}

						(Value::Temp(t), Value::Int(i), op, kind) => {
							if let Some((new_t, Value::Int(offset))) =
								addicitive_synonym.get(&t)
							{
								if let Some(new_i) = i.checked_sub(offset) {
									new_instrs.push(Box::new(CompInstr {
										kind,
										target,
										op,
										var_type,
										lhs: Value::Temp(new_t),
										rhs: new_i.into(),
									}));
									changed = offset != 0;
								} else {
									new_instrs.push(instr);
								}
							} else {
								new_instrs.push(instr);
							}
						}

						(Value::Float(f), Value::Temp(t), op, kind) => {
							if let Some((new_t, Value::Float(offset))) =
								addicitive_synonym.get(&t)
							{
								new_instrs.push(Box::new(CompInstr {
									kind,
									target,
									op,
									var_type,
									lhs: (f - offset).into(),
									rhs: Value::Temp(new_t),
								}));
								changed = offset != 0f32;
							} else {
								new_instrs.push(instr);
							}
						}

						(Value::Temp(t), Value::Float(f), op, kind) => {
							if let Some((new_t, Value::Float(offset))) =
								addicitive_synonym.get(&t)
							{
								new_instrs.push(Box::new(CompInstr {
									kind,
									target,
									op,
									var_type,
									lhs: Value::Temp(new_t),
									rhs: (f - offset).into(),
								}));
								changed = true;
							} else {
								new_instrs.push(instr);
							}
						}

						_ => {
							new_instrs.push(instr);
						}
					};
				}
				_ => new_instrs.push(instr),
			},
		);

		block.borrow_mut().instrs = new_instrs;
	}

	changed
}

impl RrvmOptimizer for SimplifyCompare {
	fn new() -> Self {
		Self {}
	}

	fn apply(self, program: &mut LlvmProgram) -> Result<bool> {
		Ok(program.funcs.iter_mut().fold(false, |x, func| x | solve_function(func)))
	}
}
