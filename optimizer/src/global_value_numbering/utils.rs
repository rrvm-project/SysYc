use llvm::*;
use rand::rngs::StdRng;
use std::collections::HashMap;

use crate::{
	metadata::MetaData,
	number::{str2num, Number},
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum HashItem {
	Arith(ArithOp),
	Comp(CompOp),
	Convert(ConvertOp),
}

impl From<ArithOp> for HashItem {
	fn from(op: ArithOp) -> Self {
		HashItem::Arith(op)
	}
}

impl From<CompOp> for HashItem {
	fn from(op: CompOp) -> Self {
		HashItem::Comp(op)
	}
}

impl From<ConvertOp> for HashItem {
	fn from(op: ConvertOp) -> Self {
		HashItem::Convert(op)
	}
}

#[derive(Default, Clone)]
pub struct NodeInfo {
	pub num_mapper: HashMap<LlvmTemp, Number>,
	var_mapper: HashMap<Number, Value>,
	exp_mapper: HashMap<(HashItem, Number, Number), Number>,
	func_mapper: HashMap<(String, Vec<Number>), Number>,
}

impl NodeInfo {
	pub fn get_value(&self, number: &Number) -> Option<Value> {
		self.var_mapper.get(number).cloned()
	}
	pub fn get_number(&self, value: &Value) -> Number {
		match value {
			Value::Int(v) => Number::from(*v as u32),
			Value::Float(v) => Number::from(v.to_bits()),
			Value::Temp(v) => self.num_mapper.get(v).cloned().expect(&v.name),
		}
	}
	pub fn map_exp(
		&mut self,
		op: impl Into<HashItem> + Copy,
		lhs: Number,
		rhs: Number,
		rng: &mut StdRng,
	) -> Number {
		if let Some(number) =
			self.exp_mapper.get(&(op.into(), lhs.clone(), rhs.clone()))
		{
			number.clone()
		} else {
			let number = Number::new(rng);
			self.exp_mapper.insert((op.into(), lhs, rhs), number.clone());
			number
		}
	}
	pub fn set_value(&mut self, number: Number, value: Value) {
		self.var_mapper.insert(number, value);
	}
	pub fn set_number(&mut self, temp: LlvmTemp, number: Number) {
		self.num_mapper.insert(temp, number);
	}
	pub fn num2value(&self, number: &Number, var_type: VarType) -> Option<Value> {
		if let Some(x) = number.same_value() {
			Some(match var_type {
				VarType::I32 => (x as i32).into(),
				VarType::F32 => (f32::from_bits(x)).into(),
				_ => unreachable!(),
			})
		} else {
			self.get_value(number)
		}
	}
	pub fn map_value(&self, value: &Value) -> Value {
		self.num2value(&self.get_number(value), value.get_type()).unwrap()
	}
	pub fn map_func(
		&mut self,
		name: String,
		params: Vec<Number>,
		rng: &mut StdRng,
	) -> Number {
		self
			.func_mapper
			.entry((name, params))
			.or_insert_with(|| Number::new(rng))
			.clone()
	}
}

fn calc<T>(x: &Number, y: &Number, calculator: T) -> Number
where
	T: Fn(u32, u32) -> u32,
{
	Number {
		value: x
			.value
			.iter()
			.zip(y.value.iter())
			.map(|(x, y)| calculator(*x, *y))
			.collect(),
	}
}

pub fn work(
	mut instr: LlvmInstr,
	info: &mut NodeInfo,
	rng: &mut StdRng,
	flag: &mut bool,
	metadata: &mut MetaData,
) -> Option<LlvmInstr> {
	use LlvmInstrVariant::*;
	let mut mapper = HashMap::new();
	let mut insert = |oprand: &Value, number: &Number| {
		if let Some(temp) = oprand.unwrap_temp() {
			mapper.insert(temp, info.num2value(number, oprand.get_type()).unwrap());
		}
	};

	let (value, number) = match instr.get_variant() {
		ArithInstr(instr) => {
			use ArithOp::*;
			let lhs = info.get_number(&instr.lhs);
			let rhs = info.get_number(&instr.rhs);
			insert(&instr.lhs, &lhs);
			insert(&instr.rhs, &rhs);
			let number = match (instr.op, lhs.same_value(), rhs.same_value()) {
				(Add, _, _) => calc(&lhs, &rhs, |x, y| x.wrapping_add(y)),
				(Sub, _, _) => calc(&lhs, &rhs, |x, y| x.wrapping_sub(y)),
				(Mul, _, _) => calc(&lhs, &rhs, |x, y| x.wrapping_mul(y)),
				(Div, Some(x), Some(y)) => Number::from((x as i32 / y as i32) as u32),
				(Rem, Some(x), Some(y)) => Number::from((x as i32 % y as i32) as u32),
				(Shl, Some(x), Some(y)) => Number::from(x << y),
				(Lshr, Some(x), Some(y)) => Number::from(x >> y),
				(Ashr, Some(x), Some(y)) => Number::from((x as i32 >> y) as u32),
				(Xor, _, _) => calc(&lhs, &rhs, |x, y| x ^ y),
				(Fadd, Some(x), Some(y)) => {
					Number::from((f32::from_bits(x) + f32::from_bits(y)).to_bits())
				}
				(Fsub, Some(x), Some(y)) => {
					Number::from((f32::from_bits(x) - f32::from_bits(y)).to_bits())
				}
				(Fmul, Some(x), Some(y)) => {
					Number::from((f32::from_bits(x) * f32::from_bits(y)).to_bits())
				}
				(Fdiv, Some(x), Some(y)) => {
					Number::from((f32::from_bits(x) / f32::from_bits(y)).to_bits())
				}
				(op, _, _) => info.map_exp(op, lhs, rhs, rng),
			};
			(info.num2value(&number, instr.var_type), number)
		}
		CompInstr(instr) => {
			use CompOp::*;
			let lhs = info.get_number(&instr.lhs);
			let rhs = info.get_number(&instr.rhs);
			insert(&instr.lhs, &lhs);
			insert(&instr.rhs, &rhs);
			let number = match (instr.op, lhs.same_value(), rhs.same_value()) {
				(EQ, Some(x), Some(y)) => {
					Number::from(((x as i32) == (y as i32)) as u32)
				}
				(NE, Some(x), Some(y)) => {
					Number::from(((x as i32) != (y as i32)) as u32)
				}
				(SGT, Some(x), Some(y)) => {
					Number::from(((x as i32) > (y as i32)) as u32)
				}
				(SGE, Some(x), Some(y)) => {
					Number::from(((x as i32) >= (y as i32)) as u32)
				}
				(SLT, Some(x), Some(y)) => {
					Number::from(((x as i32) < (y as i32)) as u32)
				}
				(SLE, Some(x), Some(y)) => {
					Number::from(((x as i32) <= (y as i32)) as u32)
				}
				(OEQ, Some(x), Some(y)) => {
					Number::from((f32::from_bits(x) == f32::from_bits(y)) as u32)
				}
				(ONE, Some(x), Some(y)) => {
					Number::from((f32::from_bits(x) != f32::from_bits(y)) as u32)
				}
				(OGT, Some(x), Some(y)) => {
					Number::from((f32::from_bits(x) > f32::from_bits(y)) as u32)
				}
				(OGE, Some(x), Some(y)) => {
					Number::from((f32::from_bits(x) >= f32::from_bits(y)) as u32)
				}
				(OLT, Some(x), Some(y)) => {
					Number::from((f32::from_bits(x) < f32::from_bits(y)) as u32)
				}
				(OLE, Some(x), Some(y)) => {
					Number::from((f32::from_bits(x) <= f32::from_bits(y)) as u32)
				}
				(op, _, _) => info.map_exp(op, lhs, rhs, rng),
			};
			(info.num2value(&number, instr.var_type), number)
		}
		ConvertInstr(instr) => {
			let lhs = info.get_number(&instr.lhs);
			insert(&instr.lhs, &lhs);
			let number = match instr.op {
				ConvertOp::Int2Float => Number {
					value: lhs
						.value
						.iter()
						.map(|x| (*x as i32 as f32).to_bits())
						.collect(),
				},
				ConvertOp::Float2Int => Number {
					value: lhs
						.value
						.iter()
						.map(|x| f32::from_bits(*x) as i32 as u32)
						.collect(),
				},
			};
			(info.num2value(&number, instr.var_type), number)
		}
		GEPInstr(instr) => {
			let lhs = info.get_number(&instr.addr);
			let rhs = info.get_number(&instr.offset);
			insert(&instr.addr, &lhs);
			insert(&instr.offset, &rhs);
			let number = calc(&lhs, &rhs, |x, y| x.wrapping_add(y));
			(info.num2value(&number, instr.var_type), number)
		}
		AllocInstr(_) => (None, Number::new(rng)),
		CallInstr(instr) => {
			let mut params = Vec::new();
			for (_, param) in instr.params.iter() {
				let number = info.get_number(param);
				insert(param, &number);
				params.push(number);
			}
			if metadata.is_pure(&instr.func.name) {
				let number = info.map_func(instr.func.name.clone(), params, rng);
				(info.num2value(&number, instr.var_type), number)
			} else {
				(None, Number::new(rng))
			}
		}
		StoreInstr(instr) => {
			insert(&instr.value, &info.get_number(&instr.value));
			insert(&instr.addr, &info.get_number(&instr.addr));
			(None, Number::new(rng))
		}
		JumpInstr(_) => (None, Number::new(rng)),
		LoadInstr(instr) => {
			let temp = instr.addr.unwrap_temp().unwrap();
			let number = if temp.is_global {
				let num = str2num(temp.name.as_str());
				info.set_number(instr.addr.unwrap_temp().unwrap(), num.clone());
				num
			} else {
				insert(&instr.addr, &info.get_number(&instr.addr));
				Number::new(rng)
			};
			(info.num2value(&number, instr.var_type), number)
		}
		JumpCondInstr(instr) => {
			insert(&instr.cond, &info.get_number(&instr.cond));
			(None, Number::new(rng))
		}
		PhiInstr(_) => (None, Number::new(rng)),
		RetInstr(instr) => {
			if let Some(val) = instr.value.as_ref() {
				insert(val, &info.get_number(val));
			}
			(None, Number::new(rng))
		}
	};
	instr.map_temp(&mapper);
	match instr.get_write() {
		Some(target) => {
			info.set_number(target.clone(), number.clone());
			if value.is_none() {
				info.set_value(number, target.into());
				return Some(instr);
			}
			*flag = true;
			None
		}
		None => Some(instr),
	}
}
