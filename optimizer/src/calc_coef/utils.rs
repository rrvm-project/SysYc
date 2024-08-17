use std::{
	cell::RefCell,
	collections::{HashMap, VecDeque},
	rc::Rc,
};

use llvm::{
	compute_two_value, ArithInstr, ArithOp, LlvmInstrTrait, LlvmTemp,
	LlvmTempManager, Value, VarType,
};
use rrvm::{cfg::BasicBlock, program::LlvmFunc};

#[derive(Debug, Clone)]
pub struct Entry {
	pub k_val: Vec<Value>,
	pub b_val: Value,
	pub _mod_val: Option<Value>, // 这个先不考虑
	pub params_len: usize,       // 除了 index 以外
}

pub fn is_constant_term(entry: &Entry) -> bool {
	entry.k_val.iter().all(|x| x.is_zero())
}

pub fn get_constant_term(
	value: &Value,
	entry_map: &HashMap<LlvmTemp, Entry>,
) -> Option<Value> {
	if let Value::Temp(t) = value {
		let entry = entry_map.get(t);
		if let Some(entry) = entry {
			if is_constant_term(entry) {
				Some(entry.b_val.clone())
			} else {
				None
			}
		} else {
			None
		}
	} else {
		Some(value.clone())
	}
}
pub fn get_entry(
	value: &Value,
	entry_map: &HashMap<LlvmTemp, Entry>,
	params_len: usize,
) -> Option<Entry> {
	if let Value::Temp(t) = value {
		entry_map.get(t).cloned()
	} else {
		Some(Entry {
			k_val: match value {
				Value::Int(_) => vec![Value::Int(0); params_len],
				Value::Float(_) => vec![Value::Float(0.0); params_len],
				_ => unreachable!(),
			},
			b_val: value.clone(),
			_mod_val: None,
			params_len,
		})
	}
}
pub fn get_typed_add(value: &LlvmTemp) -> ArithOp {
	match value.var_type {
		VarType::I32 => llvm::ArithOp::Add,
		VarType::F32 => llvm::ArithOp::Fadd,
		_ => unreachable!(),
	}
}
pub fn get_value_typed_add(value: &Value) -> ArithOp {
	match value {
		Value::Int(_) => llvm::ArithOp::Add,
		Value::Float(_) => llvm::ArithOp::Fadd,
		Value::Temp(t) => get_typed_add(t),
	}
}
pub fn get_typed_mul(value: &LlvmTemp) -> ArithOp {
	match value.var_type {
		VarType::I32 => llvm::ArithOp::Mul,
		VarType::F32 => llvm::ArithOp::Fmul,
		_ => unreachable!(),
	}
}
pub fn get_typed_zero(value: &LlvmTemp) -> Value {
	match value.var_type {
		VarType::I32 => Value::Int(0),
		VarType::F32 => Value::Float(0.0),
		_ => Value::Int(0),
	}
}
pub fn get_typed_one(value: &LlvmTemp) -> Value {
	match value.var_type {
		VarType::I32 => Value::Int(1),
		VarType::F32 => Value::Float(1.0),
		_ => Value::Int(1),
	}
}
pub fn calc_mul(
	entry: &Entry,
	value: &Value,
	op: ArithOp,
	target: &LlvmTemp,
	mgr: &mut LlvmTempManager,
	entry_map: &mut HashMap<LlvmTemp, Entry>,
	block_instrs: &mut Vec<Box<dyn LlvmInstrTrait>>,
) {
	let mut new_k = vec![];
	for val in entry.k_val.iter() {
		let (val1, instr1) = compute_two_value(val.clone(), value.clone(), op, mgr);
		new_k.push(val1);
		if let Some(instr1) = instr1 {
			block_instrs.push(Box::new(instr1));
		}
	}
	let (val2, instr2) =
		compute_two_value(entry.b_val.clone(), value.clone(), op, mgr);
	entry_map.insert(
		target.clone(),
		Entry {
			k_val: new_k,
			b_val: val2,
			_mod_val: None,
			params_len: entry.params_len,
		},
	);
	if let Some(instr2) = instr2 {
		block_instrs.push(Box::new(instr2));
	}
}
// 处理 call 返回值的情况
pub fn calc_mul_entries(
	entry1: &Entry,
	entry2: &[Entry],
	op: ArithOp,
	target: &LlvmTemp,
	mgr: &mut LlvmTempManager,
	entry_map: &mut HashMap<LlvmTemp, Entry>,
	block_instrs: &mut Vec<Box<dyn LlvmInstrTrait>>,
) {
	let mut prev_val: Option<Value> = None;
	let mut new_ks = vec![];
	for (idx, val) in entry1.k_val.iter().enumerate() {
		for entry in entry2.iter() {
			let (val1, instr1) =
				compute_two_value(val.clone(), entry.k_val[idx].clone(), op, mgr);
			if let Some(instr1) = instr1 {
				block_instrs.push(Box::new(instr1));
			}
			if let Some(val) = prev_val {
				let (val2, instr2) = compute_two_value(
					val.clone(),
					val1.clone(),
					get_value_typed_add(&val1),
					mgr,
				);
				if let Some(instr2) = instr2 {
					block_instrs.push(Box::new(instr2));
				}
				prev_val = Some(val2);
			} else {
				prev_val = Some(val1);
			}
		}
		new_ks.push(prev_val.unwrap());
		prev_val = None;
	}
	// 处理 b_val
	for val in entry1.k_val.iter() {
		for entry in entry2.iter() {
			let (val1, instr1) =
				compute_two_value(val.clone(), entry.b_val.clone(), op, mgr);
			if let Some(instr1) = instr1 {
				block_instrs.push(Box::new(instr1));
			}
			if let Some(val) = &prev_val {
				let (val2, instr2) = compute_two_value(
					val.clone(),
					val1.clone(),
					get_value_typed_add(&val1),
					mgr,
				);
				if let Some(instr2) = instr2 {
					block_instrs.push(Box::new(instr2));
				}
				prev_val = Some(val2);
			} else {
				prev_val = Some(val1);
			}
		}
		let (val_b, instr_b) = compute_two_value(
			entry1.b_val.clone(),
			prev_val.clone().unwrap().clone(),
			get_value_typed_add(&entry1.b_val),
			mgr,
		);
		if let Some(instr_b) = instr_b {
			block_instrs.push(Box::new(instr_b));
		}
		entry_map.insert(
			target.clone(),
			Entry {
				k_val: new_ks.clone(),
				b_val: val_b,
				_mod_val: None,
				params_len: entry1.params_len,
			},
		);
	}
}
pub fn calc_arith(
	arith_instr: &ArithInstr,
	entry_map: &mut HashMap<LlvmTemp, Entry>,
	block_instrs: &mut Vec<Box<dyn LlvmInstrTrait>>,
	mgr: &mut LlvmTempManager,
	params_len: usize,
) -> bool {
	use llvm::ArithOp::*;
	let lhs = arith_instr.lhs.clone();
	let rhs = arith_instr.rhs.clone();
	let target = arith_instr.target.clone();
	// 分类讨论 lhs 和 rhs 分别能否在 entry_map 中找到
	let lhs_entry = get_entry(&lhs, entry_map, params_len).unwrap();
	let rhs_entry = get_entry(&rhs, entry_map, params_len).unwrap();
	match arith_instr.op {
		Add | Sub | Fadd | Fsub | AddD => {
			let val_instr_vec = lhs_entry
				.k_val
				.iter()
				.zip(rhs_entry.k_val.iter())
				.map(|(lhs, rhs)| {
					compute_two_value(lhs.clone(), rhs.clone(), arith_instr.op, mgr)
				})
				.collect::<Vec<_>>();
			let (val1, instr1) = compute_two_value(
				lhs_entry.b_val.clone(),
				rhs_entry.b_val.clone(),
				arith_instr.op,
				mgr,
			);
			entry_map.insert(
				target,
				Entry {
					k_val: val_instr_vec.iter().map(|(val, _)| val.clone()).collect(),
					b_val: val1,
					_mod_val: None,
					params_len,
				},
			);
			for (_val, instr) in val_instr_vec {
				if let Some(instr) = instr {
					block_instrs.push(Box::new(instr));
				}
			}
			if let Some(instr1) = instr1 {
				block_instrs.push(Box::new(instr1));
			}
		}
		Ashr | Shl | Lshr => {
			if !is_constant_term(&rhs_entry) {
				return false;
			}
			let val_instr_vec = lhs_entry
				.k_val
				.iter()
				.map(|lhs| {
					compute_two_value(
						lhs.clone(),
						rhs_entry.b_val.clone(),
						arith_instr.op,
						mgr,
					)
				})
				.collect::<Vec<_>>();
			let (val1, instr1) = compute_two_value(
				lhs_entry.b_val.clone(),
				rhs_entry.b_val.clone(),
				arith_instr.op,
				mgr,
			);
			entry_map.insert(
				target,
				Entry {
					k_val: val_instr_vec.iter().map(|(val, _)| val.clone()).collect(),
					b_val: val1,
					_mod_val: None,
					params_len,
				},
			);
			for (_val, instr) in val_instr_vec {
				if let Some(instr) = instr {
					block_instrs.push(Box::new(instr));
				}
			}
			if let Some(instr1) = instr1 {
				block_instrs.push(Box::new(instr1));
			}
		}
		Fdiv | Div | Xor | And | Or | Rem => {
			if (!is_constant_term(&rhs_entry)) || (!is_constant_term(&lhs_entry)) {
				return false;
			}
			let (val0, instr0) = compute_two_value(
				lhs_entry.b_val.clone(),
				rhs_entry.b_val.clone(),
				arith_instr.op,
				mgr,
			);
			entry_map.insert(
				target.clone(),
				Entry {
					k_val: match target.var_type {
						VarType::I32 => vec![Value::Int(0); params_len],
						VarType::F32 => vec![Value::Float(0.0); params_len],
						_ => unreachable!(),
					},
					b_val: val0,
					_mod_val: None,
					params_len,
				},
			);
			if let Some(instr0) = instr0 {
				block_instrs.push(Box::new(instr0));
			}
		}
		Fmul | Mul => {
			// **这里认为乘法有交换律**
			let is_lhs_const = {
				if is_constant_term(&lhs_entry) || is_constant_term(&rhs_entry) {
					is_constant_term(&lhs_entry)
				} else {
					false
				}
			};
			if !is_lhs_const {
				calc_mul(
					&lhs_entry,
					&rhs_entry.b_val,
					arith_instr.op,
					&target,
					mgr,
					entry_map,
					block_instrs,
				);
			} else {
				calc_mul(
					&rhs_entry,
					&lhs_entry.b_val,
					arith_instr.op,
					&target,
					mgr,
					entry_map,
					block_instrs,
				);
			}
		}
	}
	true
}
pub fn calc_ret(
	retinstr: &llvm::RetInstr,
	entry_map: &HashMap<LlvmTemp, Entry>,
	block_instrs: &mut Vec<Box<dyn LlvmInstrTrait>>,
	mgr: &mut LlvmTempManager,
	addr: &LlvmTemp,
) -> bool {
	// 把 value 塞到 a 里面去
	// 注意我们是把 k_value 放在了上面
	let value = retinstr.value.clone();
	if let Some(val) = value {
		match val {
			Value::Temp(t) => {
				let entry = entry_map.get(&t);
				if let Some(entry) = entry {
					// store 进 a 里面去
					for (idx, val) in entry.k_val.iter().enumerate() {
						let gep = llvm::GEPInstr {
							target: mgr.new_temp(llvm::VarType::I32Ptr, false),
							var_type: llvm::VarType::I32Ptr,
							addr: Value::Temp(addr.clone()),
							offset: Value::Int(4 * idx as i32),
						};
						let store = llvm::StoreInstr {
							value: val.clone(),
							addr: Value::Temp(gep.target.clone()),
						};
						block_instrs.push(Box::new(gep));
						block_instrs.push(Box::new(store));
					}
					let gep2 = llvm::GEPInstr {
						target: mgr.new_temp(llvm::VarType::I32Ptr, false),
						var_type: llvm::VarType::I32Ptr,
						addr: Value::Temp(addr.clone()),
						offset: Value::Int(4 * entry.k_val.len() as i32),
					};
					let store2 = llvm::StoreInstr {
						value: entry.b_val.clone(),
						addr: Value::Temp(gep2.target.clone()),
					};
					let ret = llvm::RetInstr { value: None };
					block_instrs.push(Box::new(gep2));
					block_instrs.push(Box::new(store2));
					block_instrs.push(Box::new(ret));
				} else {
					panic!("ret instr value not in entry map");
				}
			}
			_ => {
				let gep_instr = llvm::GEPInstr {
					target: mgr.new_temp(llvm::VarType::I32Ptr, false),
					var_type: llvm::VarType::I32Ptr,
					addr: Value::Temp(addr.clone()),
					offset: Value::Int(4),
				};
				let store_instr = llvm::StoreInstr {
					value: val.clone(),
					addr: Value::Temp(gep_instr.target.clone()),
				};
				// 另一个 store 为0
				let store_instr2 = llvm::StoreInstr {
					value: match val.get_type() {
						llvm::VarType::I32 => Value::Int(0),
						llvm::VarType::F32 => Value::Float(0.0),
						_ => Value::Int(0),
					},
					addr: Value::Temp(addr.clone()),
				};
				let ret = llvm::RetInstr { value: None };
				block_instrs.push(Box::new(gep_instr));
				block_instrs.push(Box::new(store_instr));
				block_instrs.push(Box::new(store_instr2));
				block_instrs.push(Box::new(ret));
			}
		}
	} else {
		return false;
	}
	true
}
// TODO calc_call 把recurse_index filter 掉
pub fn calc_call(
	call_instr: &llvm::CallInstr,
	entry_map: &mut HashMap<LlvmTemp, Entry>,
	block_instrs: &mut Vec<Box<dyn LlvmInstrTrait>>,
	mgr: &mut LlvmTempManager,
	func_name: String,
	addr: &LlvmTemp,
	index: usize,
) -> bool {
	if call_instr.func.name != func_name {
		let params = call_instr.params.clone();
		let mut new_params = Vec::new();
		for (vartype, param) in params.iter() {
			let get_param_val = get_constant_term(param, entry_map);
			if let Some(param_val) = get_param_val {
				new_params.push((*vartype, param_val));
			} else {
				return false;
			}
		}
		let instr = llvm::CallInstr {
			target: call_instr.target.clone(),
			var_type: call_instr.var_type,
			func: call_instr.func.clone(),
			params: new_params,
		};
		block_instrs.push(Box::new(instr));
	} else {
		// 先把所有 f,g load 出来，组装成 entry 然后计算
		let mut kvals = vec![];
		for i in 0..call_instr.params.len() - 1 {
			let gep = llvm::GEPInstr {
				target: mgr.new_temp(llvm::VarType::I32Ptr, false),
				var_type: llvm::VarType::I32Ptr,
				addr: Value::Temp(addr.clone()),
				offset: Value::Int(4 * i as i32),
			};
			let load_target = mgr.new_temp(call_instr.var_type, false);
			let load = llvm::LoadInstr {
				target: load_target.clone(),
				var_type: call_instr.var_type,
				addr: Value::Temp(gep.target.clone()),
			};
			block_instrs.push(Box::new(gep));
			block_instrs.push(Box::new(load));
			kvals.push(Value::Temp(load_target));
		}
		let gep2 = llvm::GEPInstr {
			target: mgr.new_temp(llvm::VarType::I32Ptr, false),
			var_type: llvm::VarType::I32Ptr,
			addr: Value::Temp(addr.clone()),
			offset: Value::Int(4 * (call_instr.params.len() - 1) as i32),
		};
		let b_target = mgr.new_temp(call_instr.var_type, false);
		let load2 = llvm::LoadInstr {
			target: b_target.clone(),
			var_type: call_instr.var_type,
			addr: Value::Temp(gep2.target.clone()),
		};
		block_instrs.push(Box::new(gep2));
		block_instrs.push(Box::new(load2));
		let func_entry = Entry {
			k_val: kvals.clone(),
			b_val: Value::Temp(b_target),
			_mod_val: None,
			params_len: kvals.len(),
		};
		let params: Vec<_> = call_instr
			.params
			.iter()
			.enumerate()
			.filter(|(idx, _entry)| *idx != index)
			.map(|(_idx, (_ty, val))| {
				get_entry(val, entry_map, call_instr.params.len() - 1).unwrap()
			})
			.collect();
		calc_mul_entries(
			&func_entry,
			&params,
			get_typed_mul(&call_instr.target),
			&call_instr.target,
			mgr,
			entry_map,
			block_instrs,
		);
	}
	true
}
#[allow(clippy::ptr_arg)]
pub fn create_wrapper(
	data: &Vec<LlvmTemp>,
	index: &LlvmTemp,
	func_name: String,
	mgr: &mut LlvmTempManager,
	params_len: usize,
) -> Rc<RefCell<BasicBlock<Box<dyn LlvmInstrTrait>, LlvmTemp>>> {
	let mut instrs: Vec<Box<dyn LlvmInstrTrait>> = vec![];
	let alloc_target = mgr.new_temp(llvm::VarType::I32Ptr, false);
	let alloc_instr = llvm::AllocInstr {
		target: alloc_target.clone(),
		length: Value::Int(16 * ((params_len + 15) / 16) as i32),
		var_type: llvm::VarType::I32Ptr,
	};
	let call_instr = llvm::CallInstr {
		target: mgr.new_temp(llvm::VarType::I32, false),
		var_type: llvm::VarType::Void,
		func: utils::Label {
			name: format!("{}_calc_coef", func_name),
		},
		params: vec![
			(llvm::VarType::I32Ptr, Value::Temp(alloc_target.clone())),
			(index.clone().var_type, Value::Temp(index.clone())),
		],
	};
	instrs.push(Box::new(alloc_instr));
	instrs.push(Box::new(call_instr));
	// 从alloc_target 里面把系数全 load 出来，和 data 中对应项相乘然后求和
	let mut prev_val: Option<LlvmTemp> = None;
	for i in 0..params_len {
		let gep = llvm::GEPInstr {
			target: mgr.new_temp(llvm::VarType::I32Ptr, false),
			var_type: llvm::VarType::I32Ptr,
			addr: Value::Temp(alloc_target.clone()),
			offset: Value::Int(4 * i as i32),
		};
		let load_target = mgr.new_temp(data[0].var_type, false);
		let load = llvm::LoadInstr {
			target: load_target.clone(),
			var_type: data[i].var_type,
			addr: Value::Temp(gep.target.clone()),
		};
		let mul = llvm::ArithInstr {
			target: mgr.new_temp(data[i].var_type, false),
			op: get_typed_mul(&data[i]),
			var_type: data[i].var_type,
			lhs: Value::Temp(load_target.clone()),
			rhs: Value::Temp(data[i].clone()),
		};
		instrs.push(Box::new(gep));
		instrs.push(Box::new(load));
		instrs.push(Box::new(mul.clone()));
		if let Some(val) = prev_val {
			let add = llvm::ArithInstr {
				target: mgr.new_temp(data[i].var_type, false),
				op: get_typed_add(&data[i]),
				var_type: data[i].var_type,
				lhs: Value::Temp(val.clone()),
				rhs: Value::Temp(mul.target.clone()),
			};
			instrs.push(Box::new(add.clone()));
			prev_val = Some(add.target.clone());
		} else {
			prev_val = Some(mul.target.clone());
		}
	}
	// 加上b
	let gep2 = llvm::GEPInstr {
		target: mgr.new_temp(llvm::VarType::I32Ptr, false),
		var_type: llvm::VarType::I32Ptr,
		addr: Value::Temp(alloc_target.clone()),
		offset: Value::Int(4 * params_len as i32),
	};
	let load2_target = mgr.new_temp(data[0].var_type, false);
	let load2 = llvm::LoadInstr {
		target: load2_target.clone(),
		var_type: data[0].var_type,
		addr: Value::Temp(gep2.target.clone()),
	};
	let add2_target = mgr.new_temp(data[0].var_type, false);
	let add2 = llvm::ArithInstr {
		target: add2_target.clone(),
		op: get_typed_add(&data[0]),
		var_type: data[0].var_type,
		lhs: Value::Temp(prev_val.unwrap()),
		rhs: Value::Temp(load2_target.clone()),
	};
	instrs.push(Box::new(gep2));
	instrs.push(Box::new(load2));
	instrs.push(Box::new(add2));
	let ret_instr = llvm::RetInstr {
		value: Some(Value::Temp(add2_target)),
	};
	let node = BasicBlock::new_node(0, 1.0);
	node.borrow_mut().instrs = instrs;
	node.borrow_mut().jump_instr = Some(Box::new(ret_instr));
	node
}
pub fn topology_sort(func: &LlvmFunc) -> Vec<i32> {
	let mut indegs = func
		.cfg
		.blocks
		.iter()
		.map(|block| (block.borrow().id, block.borrow().prev.len()))
		.collect::<HashMap<_, _>>();
	let mut queue = VecDeque::new();
	let mut res = vec![];
	queue.push_back(func.cfg.get_entry().borrow().id);
	while let Some(node) = queue.pop_front() {
		res.push(node);
		for v in func
			.cfg
			.blocks
			.iter()
			.find(|block| block.borrow().id == node)
			.unwrap()
			.borrow()
			.succ
			.iter()
		{
			let v = v.borrow().id;
			let indeg = indegs.get_mut(&v).unwrap();
			*indeg -= 1;
			if *indeg == 0 {
				queue.push_back(v);
			}
		}
	}
	if res.len() != func.cfg.blocks.len() {
		vec![]
	} else {
		res
	}
}
