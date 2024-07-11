use llvm::*;
use rand::{rngs::StdRng, SeedableRng};
use std::{
	collections::{hash_map::DefaultHasher, HashMap},
	hash::{Hash, Hasher},
};

use super::number::Number;

// use this function to solve global variable
pub fn str2num(input: &str) -> Number {
	let mut hasher = DefaultHasher::new();
	input.hash(&mut hasher);
	let hash_value = hasher.finish();
	let mut rng = StdRng::seed_from_u64(hash_value);
	Number::new(&mut rng)
}

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
	num_mapper: HashMap<LlvmTemp, Number>,
	var_mapper: HashMap<Number, Value>,
	exp_mapper: HashMap<(HashItem, Number, Number), Number>,
}

impl NodeInfo {
	pub fn get_value(&self, number: &Number) -> Option<Value> {
		self.var_mapper.get(number).cloned()
	}
	pub fn get_number(&self, value: &Value) -> Number {
		match value {
			Value::Int(v) => Number::from(*v as u32),
			Value::Float(v) => Number::from(v.to_bits()),
			Value::Temp(v) => self.num_mapper.get(v).cloned().unwrap(),
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
				VarType::F32 => (x as f32).into(),
				_ => unreachable!(),
			})
		} else {
			self.get_value(number)
		}
	}
	pub fn map_value(&self, value: &Value) -> Value {
		self.num2value(&self.get_number(value), value.get_type()).unwrap()
	}
}

fn calc_number<T>(x: &Number, y: &Number, calculator: T) -> Number
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
) -> Option<LlvmInstr> {
	use LlvmInstrVariant::*;
	let mut mapper = HashMap::new();
	let mut insert = |oprand: &Value, number: &Number| {
		if let Some(temp) = oprand.unwrap_temp() {
			mapper.insert(temp, info.num2value(number, oprand.get_type()).unwrap());
		}
	};
	// eprintln!("{}", instr);
	let (value, number) = match instr.get_variant() {
		ArithInstr(instr) => {
			let lhs = info.get_number(&instr.lhs);
			let rhs = info.get_number(&instr.rhs);
			insert(&instr.lhs, &lhs);
			insert(&instr.rhs, &rhs);
			let number = match instr.op {
				ArithOp::Add => calc_number(&lhs, &rhs, |x, y| x.wrapping_add(y)),
				ArithOp::AddD => calc_number(&lhs, &rhs, |x, y| x.wrapping_add(y)),
				ArithOp::Sub => calc_number(&lhs, &rhs, |x, y| x.wrapping_sub(y)),
				ArithOp::Mul => calc_number(&lhs, &rhs, |x, y| x.wrapping_mul(y)),
				ArithOp::Div => info.map_exp(ArithOp::Div, lhs, rhs, rng),
				ArithOp::Rem => info.map_exp(ArithOp::Rem, lhs, rhs, rng),
				ArithOp::Shl => info.map_exp(ArithOp::Shl, lhs, rhs, rng),
				ArithOp::Lshr => info.map_exp(ArithOp::Lshr, lhs, rhs, rng),
				ArithOp::Ashr => info.map_exp(ArithOp::Ashr, lhs, rhs, rng),
				ArithOp::And => calc_number(&lhs, &rhs, |x, y| x & y),
				ArithOp::Or => calc_number(&lhs, &rhs, |x, y| x | y),
				ArithOp::Xor => calc_number(&lhs, &rhs, |x, y| x ^ y),
				ArithOp::Fadd => calc_number(&lhs, &rhs, |x, y| {
					(f32::from_bits(x) + f32::from_bits(y)).to_bits()
				}),
				ArithOp::Fsub => calc_number(&lhs, &rhs, |x, y| {
					(f32::from_bits(x) - f32::from_bits(y)).to_bits()
				}),
				ArithOp::Fmul => calc_number(&lhs, &rhs, |x, y| {
					(f32::from_bits(x) * f32::from_bits(y)).to_bits()
				}),
				ArithOp::Fdiv => info.map_exp(ArithOp::Fdiv, lhs, rhs, rng),
			};
			(info.num2value(&number, instr.var_type), number)
		}
		CompInstr(instr) => {
			let lhs = info.get_number(&instr.lhs);
			let rhs = info.get_number(&instr.rhs);
			insert(&instr.lhs, &lhs);
			insert(&instr.rhs, &rhs);
			let number = match instr.op {
				CompOp::EQ => info.map_exp(CompOp::EQ, lhs, rhs, rng),
				CompOp::NE => info.map_exp(CompOp::NE, lhs, rhs, rng),
				CompOp::SGT => info.map_exp(CompOp::SGT, lhs, rhs, rng),
				CompOp::SGE => info.map_exp(CompOp::SGE, lhs, rhs, rng),
				CompOp::SLT => info.map_exp(CompOp::SLT, lhs, rhs, rng),
				CompOp::SLE => info.map_exp(CompOp::SLE, lhs, rhs, rng),
				CompOp::OEQ => info.map_exp(CompOp::OEQ, lhs, rhs, rng),
				CompOp::ONE => info.map_exp(CompOp::ONE, lhs, rhs, rng),
				CompOp::OGT => info.map_exp(CompOp::OGT, lhs, rhs, rng),
				CompOp::OGE => info.map_exp(CompOp::OGE, lhs, rhs, rng),
				CompOp::OLT => info.map_exp(CompOp::OLT, lhs, rhs, rng),
				CompOp::OLE => info.map_exp(CompOp::OLE, lhs, rhs, rng),
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
			let number = calc_number(&lhs, &rhs, |x, y| x.wrapping_add(y));
			(info.num2value(&number, instr.var_type), number)
		}
		AllocInstr(_) => (None, Number::new(rng)),
		CallInstr(insrt) => {
			for (_, param) in insrt.params.iter() {
				insert(param, &info.get_number(param));
			}
			(None, Number::new(rng))
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
				str2num(temp.name.as_str())
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
	// eprintln!("{}", instr);
	instr.map_temp(&mapper);
	// eprintln!("-> {:?}", value);
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
