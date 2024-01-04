use std::{collections::HashMap, hash::Hasher};

use llvm::{
	llvmop::{ArithOp, CompOp},
	llvmvar::VarType,
	LlvmInstrTrait, Temp, Value,
};

use std::hash::Hash;

use super::calc::{arith_binaryop, comp_binaryop};
use rrvm::LlvmNode;

enum SimpleLvnValue {
	// LiteralInt(i32),
	// LiteralFloat(f32),
	Arith((ArithOp, VarType, Value, Value)),
	Comp((CompOp, VarType, Value, Value)),
	Convert((VarType, VarType, Value)),
	Gep(Value, Value),
}
impl Eq for SimpleLvnValue {}

impl PartialEq for SimpleLvnValue {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			// (Self::LiteralInt(l0), Self::LiteralInt(r0)) => l0 == r0,
			// (Self::LiteralFloat(l0), Self::LiteralFloat(r0)) => l0 == r0,
			(Self::Arith(l0), Self::Arith(r0)) => l0 == r0,
			(Self::Comp(l0), Self::Comp(r0)) => l0 == r0,
			(Self::Convert(l0), Self::Convert(r0)) => l0 == r0,
			(Self::Gep(l0, l1), Self::Gep(r0, r1)) => l0 == r0 && l1 == r1,
			_ => false,
		}
	}
}

impl Hash for SimpleLvnValue {
	fn hash<H: Hasher>(&self, state: &mut H) {
		core::mem::discriminant(self).hash(state);
		match self {
			// SimpleLvnValue::LiteralInt(i) => {
			// 	i.hash(state);
			// }
			// SimpleLvnValue::LiteralFloat(f) => {
			// 	let mut value = *f;
			// 	if value.is_nan() || value.is_infinite() {
			// 		value = 1926.0817f32;
			// 	}
			// 	value.to_bits().hash(state);
			// }
			SimpleLvnValue::Arith((op, var_type, val1, val2)) => {
				op.hash(state);
				var_type.hash(state);
				val1.hash(state);
				val2.hash(state);
			}
			SimpleLvnValue::Comp((op, var_type, val1, val2)) => {
				op.hash(state);
				var_type.hash(state);
				val1.hash(state);
				val2.hash(state);
			}
			SimpleLvnValue::Convert((var_type1, var_type2, val1)) => {
				var_type1.hash(state);
				var_type2.hash(state);
				val1.hash(state);
			}
			SimpleLvnValue::Gep(val1, val2) => {
				val1.hash(state);
				val2.hash(state);
			}
		}
	}
}

fn try_const(value: &SimpleLvnValue, backup_temp: Temp) -> Value {
	match value {
		SimpleLvnValue::Arith((op, _vartype, lhs, rhs)) => {
			if let Some(value) = arith_binaryop(lhs, *op, rhs) {
				match value {
					Value::Int(v) => v.into(),
					Value::Float(v) => v.into(),
					_ => unreachable!(),
				}
			} else {
				Value::Temp(backup_temp)
			}
		}
		SimpleLvnValue::Comp((op, _vartype, lhs, rhs)) => {
			if let Some(value) = comp_binaryop(lhs, *op, rhs) {
				match value {
					Value::Int(v) => v.into(),
					_ => unreachable!(),
				}
			} else {
				Value::Temp(backup_temp)
			}
		}
		_ => Value::Temp(backup_temp),
	}
}

#[allow(clippy::borrowed_box)]
fn get_simple_lvn_value(
	instr: &Box<dyn LlvmInstrTrait>,
	rewrite: &HashMap<Temp, Value>,
) -> (Option<SimpleLvnValue>, Option<Temp>) {
	fn try_rewrite(value: Value, rewrite: &HashMap<Temp, Value>) -> Value {
		match value {
			Value::Temp(t) => {
				if let Some(v) = rewrite.get(&t) {
					v.clone()
				} else {
					Value::Temp(t)
				}
			}
			_ => value,
		}
	}

	let mut dst = None;

	let value: Option<SimpleLvnValue> = match instr.get_variant() {
		llvm::LlvmInstrVariant::ArithInstr(i) => {
			dst = i.target.clone().into();
			SimpleLvnValue::Arith((
				i.op,
				i.var_type,
				try_rewrite(i.lhs.clone(), rewrite),
				try_rewrite(i.rhs.clone(), rewrite),
			))
			.into()
		}
		llvm::LlvmInstrVariant::CompInstr(i) => {
			dst = i.target.clone().into();
			SimpleLvnValue::Comp((
				i.op,
				i.var_type,
				try_rewrite(i.lhs.clone(), rewrite),
				try_rewrite(i.rhs.clone(), rewrite),
			))
			.into()
		}
		llvm::LlvmInstrVariant::ConvertInstr(i) => {
			dst = i.target.clone().into();
			SimpleLvnValue::Convert((
				i.from_type,
				i.to_type,
				try_rewrite(i.lhs.clone(), rewrite),
			))
			.into()
		}
		llvm::LlvmInstrVariant::GEPInstr(i) => {
			dst = i.target.clone().into();
			SimpleLvnValue::Gep(
				try_rewrite(i.addr.clone(), rewrite),
				try_rewrite(i.offset.clone(), rewrite),
			)
			.into()
		}
		_ => None,
	};
	(value, dst)
}

pub fn solve(block: &LlvmNode, rewrite: &mut HashMap<Temp, Value>) {
	let mut table: HashMap<SimpleLvnValue, Value> = HashMap::new();
	for instr in &block.borrow_mut().instrs {
		if let (Some(lvn_value), Some(target)) =
			get_simple_lvn_value(instr, rewrite)
		{
			if let Some(value) = table.get(&lvn_value) {
				rewrite.insert(target, value.clone());
			} else {
				let value_try = try_const(&lvn_value, target.clone());
				table.insert(lvn_value, value_try.clone());
				if value_try.is_num() {
					rewrite.insert(target, value_try);
				}
			}
		}
	}
}

pub fn rewrite_block(block: &mut LlvmNode, map: &mut HashMap<Temp, Value>) {
	let mut new_vec = vec![];
	for instr in &mut block.borrow_mut().phi_instrs {
		if instr.replaceable(map) {
			continue;
		}
		instr.map_temp(map);
		new_vec.push(instr.to_owned());
	}
	block.borrow_mut().phi_instrs = new_vec;

	let mut new_vec = vec![];
	for instr in &mut block.borrow_mut().instrs {
		if instr.replaceable(map) {
			continue;
		}
		instr.map_temp(map);
		new_vec.push(instr.clone_box());
	}
	block.borrow_mut().instrs = new_vec;

	if let Some(instr) = &mut block.borrow_mut().jump_instr {
		instr.map_temp(map);
	}
}
