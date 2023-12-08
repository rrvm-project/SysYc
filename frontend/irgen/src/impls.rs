use std::collections::{HashMap, HashSet};

use ast::tree::*;

use llvm::{Value, VarType::*, *};
use rrvm::{
	cfg::{link_cfg, link_node, CFG},
	program::{LlvmProgram, RrvmProgram},
	LlvmCFG, LlvmNode,
};

use utils::errors::Result;
use value::FuncRetType;

use crate::{
	loop_state::LoopState,
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
			symbol_table: SymbolTable::default(),
			ret_type: FuncRetType::Void,
			states: Vec::new(),
			weights: Vec::new(),
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
		let out = CFG::new(self.total, *self.weights.last().unwrap());
		self.total += 1;
		out
	}
	pub fn fold_cfgs(&mut self, cfgs: Vec<LlvmCFG>) -> LlvmCFG {
		cfgs
			.into_iter()
			.reduce(|mut acc, v| {
				link_cfg(&acc, &v);
				acc.append(v);
				acc
			})
			.unwrap_or_else(|| self.new_cfg())
	}
	pub fn if_then_else(
		&mut self,
		mut cond: LlvmCFG,
		cond_val: Value,
		cfg1: LlvmCFG,
		diff1: Table,
		cfg2: LlvmCFG,
		diff2: Table,
	) -> LlvmCFG {
		let exit = self.new_cfg();
		let keys = diff1
			.keys()
			.chain(diff2.keys())
			.cloned()
			.filter(|v| self.symbol_table.contains(v))
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
		link_cfg(&cond, &cfg1);
		link_cfg(&cond, &cfg2);
		link_cfg(&cfg1, &exit);
		link_cfg(&cfg2, &exit);
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
	pub fn copy_symbols(
		&mut self,
		symbols: Vec<i32>,
	) -> (LlvmCFG, Table, HashMap<i32, Temp>) {
		let cfg = self.new_cfg();
		let mut table = Table::new();
		let mut need_phi = HashMap::new();
		let symbols: HashSet<_> =
			symbols.into_iter().filter(|v| self.symbol_table.contains(v)).collect();
		for id in symbols {
			let value = self.symbol_table.get(&id);
			table.insert(id, value.clone());
			let var_type = value.get_type();
			let temp = self.mgr.new_temp(var_type, false);
			need_phi.insert(id, temp.clone());
			self.symbol_table.set(id, temp.into());
		}
		(cfg, table, need_phi)
	}
	pub fn link_into(
		&mut self,
		target: LlvmNode,
		prev: Vec<(LlvmNode, Table)>,
		need_phi: Option<HashMap<i32, Temp>>,
	) {
		let phi_targets: Vec<_> = prev
			.iter()
			.flat_map(|(_, table)| table.iter().map(|(k, v)| (*k, v.get_type())))
			.collect::<HashSet<_>>()
			.into_iter()
			.filter(|(id, _)| self.symbol_table.contains(id))
			.map(|(id, var_type)| {
				let temp = need_phi
					.as_ref()
					.and_then(|v| v.get(&id))
					.cloned()
					.unwrap_or_else(|| self.mgr.new_temp(var_type, false));
				(id, temp)
			})
			.collect();

		prev.iter().for_each(|(node, _)| {
			node.borrow_mut().succ.clear();
			link_node(node, &target)
		});
		let init: Vec<_> =
			phi_targets.iter().map(|(id, _)| self.symbol_table.get(id)).collect();

		for ((id, temp), default) in phi_targets.into_iter().zip(init) {
			let source = prev
				.iter()
				.map(|(node, table)| {
					(
						table.get(&id).unwrap_or(&default).clone(),
						node.borrow().label(),
					)
				})
				.collect();
			target.borrow_mut().push_phi(PhiInstr {
				var_type: temp.var_type,
				target: temp.clone(),
				source,
			});
			self.symbol_table.set(id, temp.into());
		}
	}
	pub fn top_state(&mut self) -> &mut LoopState {
		self.states.last_mut().unwrap()
	}
	pub fn enter_loop(&mut self) {
		self.weights.push(*self.weights.last().unwrap() * 10.0);
		self.states.push(LoopState::new(self.symbol_table.size()));
	}
	pub fn enter_branch(&mut self) {
		self.weights.push(*self.weights.last().unwrap() * 0.5);
	}
}
