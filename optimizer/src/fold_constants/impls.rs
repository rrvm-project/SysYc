use std::collections::HashMap;

use super::FoldConstants;
use crate::{metadata::MetaData, RrvmOptimizer};
use llvm::{
	ArithInstr,
	ArithOp::*,
	LlvmInstrVariant, LlvmTemp,
	Value::{self, *},
	VarType::*,
};
use rrvm::{program::LlvmProgram, LlvmNode};
use utils::errors::Result;

impl RrvmOptimizer for FoldConstants {
	fn new() -> Self {
		Self {}
	}
	fn apply(
		self,
		program: &mut LlvmProgram,
		_metadata: &mut MetaData,
	) -> Result<bool> {
		fn solve(block: &LlvmNode) {
			let block = &mut block.borrow_mut();
			let instrs = std::mem::take(&mut block.instrs);
			let mut sets: HashMap<LlvmTemp, (Value, i32)> = HashMap::new();
			for instr in instrs.into_iter() {
				// if let LlvmInstrVariant::ArithInstr(v) = instr.get_variant() {
				match instr.get_variant() {
					LlvmInstrVariant::ArithInstr(v) => match v.op {
						Add | Sub => match (&v.lhs, &v.rhs) {
							(Int(x), Int(y)) => {
								let num = x + y * ((v.op == Add) as i32 * 2 - 1);
								sets.insert(v.target.clone(), (num.into(), 0));
								let instr = ArithInstr::new(v.target.clone(), num, Add, 0, I32);
								block.instrs.push(instr);
							}
							(Int(x), Temp(y)) => {
								let w = (v.op == Add) as i32 * 2 - 1;
								let (var, y) = sets.get(y).cloned().unwrap_or((y.into(), 0));
								if v.op == Add {
									sets.insert(v.target.clone(), (var.clone(), x + y));
								}
								let instr =
									ArithInstr::new(v.target.clone(), x + y * w, v.op, var, I32);
								block.instrs.push(instr);
							}
							(Temp(x), Int(y)) => {
								let w = (v.op == Add) as i32 * 2 - 1;
								let (var, x) = sets.get(x).cloned().unwrap_or((x.into(), 0));
								sets.insert(v.target.clone(), (var.clone(), x + y * w));
								let instr =
									ArithInstr::new(v.target.clone(), var, v.op, y + x * w, I32);
								block.instrs.push(instr);
							}
							_ => block.instrs.push(instr),
						},
						_ => block.instrs.push(instr),
					},
					LlvmInstrVariant::ConvertInstr(v) => match (&v.lhs, v.op) {
						(Int(t), llvm::ConvertOp::Int2Float) => {
							let f: f32 = *t as f32;
							let instr = ArithInstr::new(
								v.target.clone(),
								Value::Float(f),
								Fadd,
								Value::Float(0f32),
								F32,
							);
							block.instrs.push(instr);
						}
						(Float(t), llvm::ConvertOp::Float2Int) => {
							let i: i32 = *t as i32;
							let instr = ArithInstr::new(
								v.target.clone(),
								Value::Int(i),
								Add,
								Value::Int(0),
								I32,
							);
							block.instrs.push(instr);
						}
						_ => block.instrs.push(instr),
					},

					_ => block.instrs.push(instr),
				}
			}
		}
		program.analysis();
		for func in program.funcs.iter() {
			func.cfg.blocks.iter().for_each(solve);
		}
		Ok(false)
	}
}
