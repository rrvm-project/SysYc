use rand::Rng;
use std::{collections::HashMap, hash::Hasher, vec};

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

fn try_rewrite(value: &Value, rewrite: &HashMap<Temp, Value>) -> Value {
	match value {
		Value::Temp(t) => {
			let mut result = Value::Temp(t.clone());
			loop {
				if let Value::Temp(ref t0) = result {
					if let Some(new_value) = rewrite.get(&t0) {
						result = new_value.clone();
					} else {
						break;
					}
				} else {
					break;
				}
			}
			result
		}
		_ => value.clone(),
	}
}

#[allow(clippy::borrowed_box)]
fn get_simple_lvn_value(
	instr: &Box<dyn LlvmInstrTrait>,
	rewrite: &HashMap<Temp, Value>,
) -> (Option<SimpleLvnValue>, Option<Temp>) {
	let mut dst = None;

	let value: Option<SimpleLvnValue> = match instr.get_variant() {
		llvm::LlvmInstrVariant::ArithInstr(i) => {
			dst = i.target.clone().into();
			SimpleLvnValue::Arith((
				i.op,
				i.var_type,
				try_rewrite(&i.lhs, rewrite),
				try_rewrite(&i.rhs, rewrite),
			))
			.into()
		}
		llvm::LlvmInstrVariant::CompInstr(i) => {
			dst = i.target.clone().into();
			SimpleLvnValue::Comp((
				i.op,
				i.var_type,
				try_rewrite(&i.lhs, rewrite),
				try_rewrite(&i.rhs, rewrite),
			))
			.into()
		}
		llvm::LlvmInstrVariant::ConvertInstr(i) => {
			dst = i.target.clone().into();
			SimpleLvnValue::Convert((
				i.from_type,
				i.to_type,
				try_rewrite(&i.lhs, rewrite),
			))
			.into()
		}
		llvm::LlvmInstrVariant::GEPInstr(i) => {
			dst = i.target.clone().into();
			SimpleLvnValue::Gep(
				try_rewrite(&i.addr, rewrite),
				try_rewrite(&i.offset, rewrite),
			)
			.into()
		}
		_ => None,
	};
	(value, dst)
}

fn get_random_vec(len: usize) -> Vec<i32> {
	let mut rng = rand::thread_rng();
	(0..len).map(|_| rng.gen()).collect()
}

fn get_value_vec(
	value: &Value,
	temp_to_vec: &mut HashMap<Temp, Vec<i32>>,
	vec_table: &mut HashMap<Vec<i32>, Value>,
) -> Vec<i32> {
	let length = 16;

	match value {
		Value::Int(v) => vec![*v; length],
		Value::Float(_) => unreachable!(),
		Value::Temp(t) => {
			if let Some(vec) = temp_to_vec.get(t) {
				vec.clone()
			} else {
				let random = get_random_vec(length);
				temp_to_vec.insert(t.clone(), random.clone());
				vec_table.insert(random.clone(), Value::Temp(t.clone()));
				random
			}
		}
	}
}
fn calculate_vecs(
	v1: Vec<i32>,
	v2: Vec<i32>,
	f: fn(i32, i32) -> i32,
) -> Vec<i32> {
	let min_len = v1.len().min(v2.len());
	v1.iter().zip(v2.iter()).take(min_len).map(|(&x, &y)| f(x, y)).collect()
}

#[allow(clippy::borrowed_box)]
fn get_vector_lvn_value(
	instr: &Box<dyn LlvmInstrTrait>,
	_rewrite: &HashMap<Temp, Value>,
	temp_to_vec: &mut HashMap<Temp, Vec<i32>>,
	vec_table: &mut HashMap<Vec<i32>, Value>,
) -> (Option<Vec<i32>>, Option<Temp>) {
	let mut dst = None;

	let value: Option<Vec<i32>> = match instr.get_variant() {
		llvm::LlvmInstrVariant::ArithInstr(i) => {
			dst = i.target.clone().into();
			if i.var_type == VarType::I32 {
				let vec_left = get_value_vec(&i.lhs, temp_to_vec, vec_table);
				let vec_right = get_value_vec(&i.rhs, temp_to_vec, vec_table);

				match i.op {
					ArithOp::Add => Some(calculate_vecs(vec_left, vec_right, |x, y| {
						x.wrapping_add(y)
					})),
					ArithOp::Sub => Some(calculate_vecs(vec_left, vec_right, |x, y| {
						x.wrapping_sub(y)
					})),
					ArithOp::Mul => Some(calculate_vecs(vec_left, vec_right, |x, y| {
						x.wrapping_mul(y)
					})),
					_ => None,
				}
			} else {
				None
			}
		}
		// TODO 添加对比较指令的支持？
		_ => None,
	};
	(value, dst)
}

fn check_all_equal(v: &[i32]) -> Option<i32> {
	if let Some((&first, rest)) = v.split_first() {
		if rest.iter().all(|&x| x == first) {
			Some(first)
		} else {
			None
		}
	} else {
		None
	}
}

pub fn solve(block: &LlvmNode, rewrite: &mut HashMap<Temp, Value>) {
	let mut table: HashMap<SimpleLvnValue, Value> = HashMap::new();

	let mut remain_instr = vec![];

	for instr in &block.borrow_mut().instrs {
		if let (Some(lvn_value), Some(target)) =
			get_simple_lvn_value(instr, rewrite)
		{
			if let Some(value) = table.get(&lvn_value) {
				rewrite.insert(target, try_rewrite(value, rewrite));
			} else {
				let value_try = try_const(&lvn_value, target.clone());
				let value_try = try_rewrite(&value_try, rewrite);
				table.insert(lvn_value, value_try.clone());
				if value_try.is_num() {
					rewrite.insert(target, value_try);
				} else {
					remain_instr.push(instr.clone_box());
				}
			}
		}
	}

	drop(table);

	let mut vec_table: HashMap<Vec<i32>, Value> = HashMap::new();
	let mut temp_to_vec: HashMap<Temp, Vec<i32>> = HashMap::new();

	for instr in remain_instr {
		if let (Some(lvn_value), Some(target)) =
			get_vector_lvn_value(&instr, rewrite, &mut temp_to_vec, &mut vec_table)
		{
			temp_to_vec.insert(target.clone(), lvn_value.clone());
			if let Some(value) = vec_table.get(&lvn_value) {
				rewrite.insert(target, try_rewrite(&value, rewrite));
			} else if let Some(const_value) = check_all_equal(&lvn_value) {
				rewrite.insert(target, Value::Int(const_value));
				vec_table.insert(lvn_value, Value::Int(const_value));
			} else {
				vec_table.insert(lvn_value, Value::Temp(target));
			}
		}
	}

	let mut new_rewirte = HashMap::new();

	for (key, value) in rewrite.clone() {
		new_rewirte.insert(key.clone(), try_rewrite(&value, rewrite));
	}

	*rewrite = new_rewirte;
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
