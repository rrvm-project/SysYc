use std::collections::HashSet;

use ast::tree::*;

use llvm::{Value, VarType::*, *};
use rrvm::{
	cfg::{link_cfg, CFG},
	program::{LlvmProgram, RrvmProgram},
	LlvmCFG,
};

use utils::errors::Result;
use value::FuncRetType;

use crate::{
	symbol_table::{SymbolTable, Table},
	IRGenerator,
};

impl IRGenerator {
	pub fn new() -> Self {
		Self {
			program: RrvmProgram::new(),
			stack: Vec::new(),
			total: 0,
			mgr: TempManager::new(),
			symbol_table: SymbolTable::new(),
			ret_type: FuncRetType::Void,
		}
	}
	pub fn to_rrvm(mut self, mut program: Program) -> Result<LlvmProgram> {
		program.accept(&mut self)?;
		Ok(self.program)
	}
	pub fn type_conv(
		&mut self,
		value: Value,
		target: llvm::VarType,
		cfg: &mut LlvmCFG,
	) -> Value {
		use llvmop::ConvertOp::*;
		if target == value.get_type() {
			return value;
		}
		let (from_type, to_type, op) = match target {
			I32 => (F32, I32, Float2Int),
			F32 => (I32, F32, Int2Float),
			_ => unreachable!(),
		};
		match (target, &value) {
			(F32, Value::Int(v)) => Value::Float(*v as f32),
			(I32, Value::Float(v)) => Value::Int(*v as i32),
			(_, Value::Temp(temp)) => {
				let target = self.mgr.new_temp(to_type, false);
				let instr = Box::new(ConvertInstr {
					op,
					target: target.clone(),
					from_type,
					lhs: temp.clone().into(),
					to_type,
				});
				cfg.get_exit().borrow_mut().push(instr);
				target.into()
			}
			_ => unreachable!(),
		}
	}
	pub fn solve(
		&mut self,
		val: Option<Value>,
		addr: Option<Value>,
		cfg: &mut LlvmCFG,
	) -> Value {
		match val {
			Some(value) => value,
			None => {
				let var_type = addr.as_ref().unwrap().deref_type();
				let temp = self.mgr.new_temp(var_type, false);
				let instr = Box::new(LoadInstr {
					target: temp.clone(),
					var_type,
					addr: addr.unwrap(),
				});
				cfg.get_exit().borrow_mut().push(instr);
				temp.into()
			}
		}
	}
	pub fn new_cfg(&mut self) -> LlvmCFG {
		let out = CFG::new(self.total);
		self.total += 1;
		out
	}
	pub fn fold_cfgs(&mut self, cfgs: Vec<LlvmCFG>) -> LlvmCFG {
		cfgs
			.into_iter()
			.reduce(|mut acc, mut v| {
				link_cfg(&mut acc, &mut v);
				acc.append(v);
				acc
			})
			.unwrap_or_else(|| self.new_cfg())
	}
	pub fn if_then_else(
		&mut self,
		mut cond: LlvmCFG,
		cond_val: Value,
		mut cfg1: LlvmCFG,
		diff1: Table,
		mut cfg2: LlvmCFG,
		diff2: Table,
	) -> LlvmCFG {
		let mut exit = self.new_cfg();
		let keys = diff1
			.keys()
			.chain(diff2.keys())
			.cloned()
			.collect::<HashSet<_>>()
			.into_iter();
		fn get_val(id: i32, now: &Table, default: &SymbolTable) -> Value {
			now.get(&id).map_or_else(|| default.get(&id), |v| v.clone())
		}
		for key in keys {
			let val1 = get_val(key, &diff1, &self.symbol_table);
			let val2 = get_val(key, &diff2, &self.symbol_table);
			let var_type = val1.get_type();
			let temp = self.mgr.new_temp(var_type, false);
			let instr = PhiInstr {
				target: temp.clone(),
				var_type,
				source: vec![(val1, cfg1.exit_label()), (val2, cfg2.exit_label())],
			};
			exit.get_exit().borrow_mut().push_phi(instr);
			self.symbol_table.set(key, temp.into());
		}
		link_cfg(&mut cond, &mut cfg1);
		link_cfg(&mut cond, &mut cfg2);
		link_cfg(&mut cfg1, &mut exit);
		link_cfg(&mut cfg2, &mut exit);
		let instr = Box::new(JumpCondInstr {
			var_type: I32,
			cond: cond_val,
			target_true: cfg1.entry_label(),
			target_false: cfg2.entry_label(),
		});
		cond.get_exit().borrow_mut().set_jump(Some(instr));
		cond.append(cfg1);
		cond.append(cfg2);
		cond.append(exit);
		cond
	}
}
