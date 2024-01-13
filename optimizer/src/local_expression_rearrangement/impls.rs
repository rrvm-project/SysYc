use super::LocalExpressionRearrangement;

use crate::RrvmOptimizer;
use std::collections::{BTreeMap, HashMap};

use rrvm::{program::LlvmProgram, LlvmCFG};
use utils::errors::Result;

use llvm::{
	llvmop::ArithOp, llvmvar::VarType, ArithInstr, LlvmInstrTrait, Temp, Value,
};

#[derive(Debug, PartialEq)]
enum ArithType {
	Addicative,
	Multiplitive,
}

fn get_op_for_lhs(op: ArithOp) -> ArithOp {
	match op {
		ArithOp::Sub => ArithOp::Add,
		_ => op,
	}
}

fn get_neg(op: ArithOp) -> ArithOp {
	match op {
		ArithOp::Sub => ArithOp::Add,
		ArithOp::Add => ArithOp::Sub,
		_ => op,
	}
}

fn get_arith_type(arith_type: &ArithOp) -> ArithType {
	match arith_type {
		ArithOp::Add | ArithOp::Sub => ArithType::Addicative,
		ArithOp::Mul => ArithType::Multiplitive,
		_ => unreachable!(),
	}
}

fn add_addicative_tree(
	next_temp: u32,
	instrs_list: &mut Vec<Box<dyn LlvmInstrTrait>>,
	target: Temp,
	info: &Vec<(ArithOp, Value, i32)>,
	target_type: VarType,
) -> u32 {
	let mut const_part: i32 = 0;
	let mut count: BTreeMap<Temp, i32> = BTreeMap::new();
	let mut temp_number = next_temp;

	for (op, value, times) in info {
		match op {
			ArithOp::Add => match value {
				Value::Int(v) => {
					assert_eq!(*times, 1);
					const_part = const_part.wrapping_add(*v)
				}
				Value::Float(_) => unreachable!(),
				Value::Temp(t) => {
					if let Some(former) = count.insert(t.clone(), *times) {
						count.insert(t.clone(), *times + former);
					}
				}
			},
			ArithOp::Sub => match value {
				Value::Int(v) => {
					assert_eq!(*times, 1);
					const_part = const_part.wrapping_sub(*v)
				}
				Value::Float(_) => unreachable!(),
				Value::Temp(t) => {
					if let Some(former) = count.insert(t.clone(), -*times) {
						count.insert(t.clone(), -*times + former);
					}
				}
			},
			_ => unreachable!(),
		}
	}

	// println!("{:?}",  count);

	let mut remain: Vec<(ArithOp, Value)> = vec![];

	if const_part != 0 {
		remain.push((ArithOp::Add, const_part.into()));
	}

	let mut add_part: Vec<(ArithOp, Value)> = vec![];
	let mut sub_part: Vec<(ArithOp, Value)> = vec![];

	for (t, times) in count.into_iter() {
		if times == 1 {
			add_part.push((ArithOp::Add, Value::Temp(t)));
		} else if times == 2 {
			add_part.push((ArithOp::Add, Value::Temp(t.clone())));
			add_part.push((ArithOp::Add, Value::Temp(t)));
		} else if times > 2 {
			let new_temp = Temp {
				name: temp_number.to_string(),
				is_global: false,
				var_type: VarType::I32,
			};
			temp_number += 1;
			instrs_list.push(Box::new(ArithInstr {
				target: new_temp.clone(),
				op: ArithOp::Mul,
				var_type: VarType::I32,
				lhs: Value::Temp(t),
				rhs: times.into(),
			}));
			add_part.push((ArithOp::Add, Value::Temp(new_temp)));
		} else if times == -1 {
			sub_part.push((ArithOp::Sub, Value::Temp(t)));
		} else if times == -2 {
			sub_part.push((ArithOp::Sub, Value::Temp(t.clone())));
			sub_part.push((ArithOp::Sub, Value::Temp(t)));
		} else if times < -2 {
			let new_temp = Temp {
				name: temp_number.to_string(),
				is_global: false,
				var_type: VarType::I32,
			};
			temp_number += 1;
			instrs_list.push(Box::new(ArithInstr {
				target: new_temp.clone(),
				op: ArithOp::Mul,
				var_type: VarType::I32,
				lhs: Value::Temp(t),
				rhs: (-times).into(),
			}));
			sub_part.push((ArithOp::Sub, Value::Temp(new_temp)));
		}
	}

	remain.append(&mut add_part);
	remain.append(&mut sub_part);

	loop {
		if remain.is_empty() {
			instrs_list.push(Box::new(ArithInstr {
				target: target.clone(),
				op: ArithOp::Add,
				var_type: target_type,
				lhs: 0.into(),
				rhs: 0.into(),
			}));
			return temp_number;
		}
		if remain.len() == 1 {
			let (op, value) = remain.first().unwrap();
			instrs_list.push(Box::new(ArithInstr {
				target: target.clone(),
				op: *op,
				var_type: target_type,
				lhs: 0.into(),
				rhs: value.clone(),
			}));
			return temp_number;
		}
		if remain.len() == 2 {
			let (op0, value0) = remain.first().unwrap();
			let (op1, value1) = remain.get(1).unwrap();

			match (op0, op1) {
				(ArithOp::Add, ArithOp::Add) => {
					instrs_list.push(Box::new(ArithInstr {
						target: target.clone(),
						op: ArithOp::Add,
						var_type: target_type,
						lhs: value1.to_owned(),
						rhs: value0.to_owned(),
					}));
					return temp_number;
				}
				(ArithOp::Add, ArithOp::Sub) => {
					instrs_list.push(Box::new(ArithInstr {
						target: target.clone(),
						op: ArithOp::Sub,
						var_type: target_type,
						lhs: value0.to_owned(),
						rhs: value1.to_owned(),
					}));
					return temp_number;
				}
				(ArithOp::Sub, ArithOp::Add) => {
					instrs_list.push(Box::new(ArithInstr {
						target: target.clone(),
						op: ArithOp::Sub,
						var_type: target_type,
						lhs: value1.to_owned(),
						rhs: value0.to_owned(),
					}));
					return temp_number;
				}
				(ArithOp::Sub, ArithOp::Sub) => {}
				(_, _) => {
					unreachable!();
				}
			}
		}

		let mut new_remain = vec![];
		loop {
			if remain.len() >= 2 {
				let new_temp = Temp {
					name: temp_number.to_string(),
					is_global: false,
					var_type: VarType::I32,
				};
				temp_number += 1;
				let (op1, value1) = remain.pop().unwrap();
				let (op0, value0) = remain.pop().unwrap();
				match (op0, op1) {
					(ArithOp::Add, ArithOp::Add) => {
						instrs_list.push(Box::new(ArithInstr {
							target: new_temp.clone(),
							op: ArithOp::Add,
							var_type: target_type,
							lhs: value0.clone(),
							rhs: value1.clone(),
						}));
						new_remain.push((ArithOp::Add, new_temp.into()));
					}
					(ArithOp::Add, ArithOp::Sub) => {
						instrs_list.push(Box::new(ArithInstr {
							target: new_temp.clone(),
							op: ArithOp::Sub,
							var_type: target_type,
							lhs: value0.clone(),
							rhs: value1.clone(),
						}));
						new_remain.push((ArithOp::Add, new_temp.into()));
					}
					(ArithOp::Sub, ArithOp::Add) => {
						instrs_list.push(Box::new(ArithInstr {
							target: new_temp.clone(),
							op: ArithOp::Sub,
							var_type: target_type,
							lhs: value1.clone(),
							rhs: value0.clone(),
						}));
						new_remain.push((ArithOp::Add, new_temp.into()));
					}
					(ArithOp::Sub, ArithOp::Sub) => {
						instrs_list.push(Box::new(ArithInstr {
							target: new_temp.clone(),
							op: ArithOp::Add,
							var_type: target_type,
							lhs: value1.clone(),
							rhs: value0.clone(),
						}));
						new_remain.push((ArithOp::Sub, new_temp.into()));
					}
					(_, _) => {
						unreachable!();
					}
				}
			} else if remain.len() == 1 {
				new_remain.push(remain.pop().unwrap());
				break;
			} else {
				break;
			}
		}
		new_remain.reverse();
		assert_eq!(remain.len(), 0);
		remain = new_remain;
	}
}

fn add_multiplitive_tree(
	next_temp: u32,
	instrs_list: &mut Vec<Box<dyn LlvmInstrTrait>>,
	target: Temp,
	info: &Vec<(ArithOp, Value, i32)>,
	target_type: VarType,
) -> u32 {
	let mut const_part: i32 = 1;
	let mut temp_part = vec![];
	let mut temp_number = next_temp;

	let mut count: BTreeMap<Temp, i32> = BTreeMap::new();

	for item in info {
		match item.0 {
			ArithOp::Mul => match &item.1 {
				Value::Int(v) => const_part = const_part.wrapping_mul(*v),
				Value::Float(_) => unreachable!(),
				Value::Temp(t) => {
					if let Some(former) = count.get(t) {
						count.insert(t.clone(), former + item.2);
					} else {
						count.insert(t.clone(), item.2);
					}
				}
			},
			_ => unreachable!(),
		}
	}

	if const_part == 0 {
		// only for int!
		instrs_list.push(Box::new(ArithInstr {
			target,
			op: ArithOp::Add,
			var_type: target_type,
			lhs: Value::Int(0),
			rhs: Value::Int(0),
		}));
		return temp_number;
	}

	let mut stack: Vec<(Temp, i32)>;
	stack = count.into_iter().collect();

	while let Some((target, mut power)) = stack.pop() {
		if power < 1 {
			continue;
		}
		let mut next_power = 2;
		let mut powers = vec![target];

		while next_power <= power {
			let new_temp = Temp {
				name: temp_number.to_string(),
				is_global: false,
				var_type: VarType::I32,
			};
			temp_number += 1;
			instrs_list.push(Box::new(ArithInstr {
				target: new_temp.clone(),
				op: ArithOp::Mul,
				var_type: VarType::I32,
				lhs: Value::Temp(powers.last().unwrap().clone()),
				rhs: Value::Temp(powers.last().unwrap().clone()),
			}));
			next_power *= 2;
			powers.push(new_temp);
		}

		for item in powers {
			if power % 2 == 1 {
				temp_part.push(item);
			}
			power >>= 1;
		}
	}

	// temp_part.sort_by(|t1, t2| t1.name.cmp(&t2.name));

	loop {
		let mut new_temp_part = vec![];
		let current_length = temp_part.len();

		if current_length == 0 {
			instrs_list.push(Box::new(ArithInstr {
				target,
				op: ArithOp::Mul,
				var_type: target_type,
				lhs: 1.into(),
				rhs: Value::Int(const_part),
			}));
			return temp_number;
		}

		if current_length == 1 {
			instrs_list.push(Box::new(ArithInstr {
				target,
				op: ArithOp::Mul,
				var_type: target_type,
				lhs: temp_part.first().unwrap().to_owned().into(),
				rhs: Value::Int(const_part),
			}));
			return temp_number;
		}

		if current_length == 2 && const_part == 1 {
			instrs_list.push(Box::new(ArithInstr {
				target,
				op: ArithOp::Mul,
				var_type: target_type,
				lhs: temp_part.first().unwrap().to_owned().into(),
				rhs: temp_part.get(1).unwrap().to_owned().into(),
			}));
			return temp_number;
		}

		loop {
			if temp_part.len() >= 2 {
				let new_temp = Temp {
					name: temp_number.to_string(),
					is_global: false,
					var_type: VarType::I32,
				};
				temp_number += 1;
				let rhs = temp_part.pop().unwrap();
				let lhs = temp_part.pop().unwrap();
				instrs_list.push(Box::new(ArithInstr {
					target: new_temp.clone(),
					op: ArithOp::Mul,
					var_type: target_type,
					lhs: lhs.into(),
					rhs: rhs.into(),
				}));
				new_temp_part.push(new_temp);
			} else if temp_part.len() == 1 {
				new_temp_part.push(temp_part.pop().unwrap());
				break;
			} else {
				break;
			}
		}
		new_temp_part.reverse();
		assert_eq!(temp_part.len(), 0);
		temp_part = new_temp_part;
	}
}

fn squash(info: &mut Vec<(ArithOp, Value, i32)>) {
	let mut addicative_part: i32 = 0;
	let mut multi_part: i32 = 1;
	let mut variable_part = HashMap::new();

	while let Some((op, v, times)) = info.pop() {
		match v {
			Value::Temp(t) => {
				if let Some(former) = variable_part.insert((op, t.clone()), times) {
					variable_part.insert((op, t), times + former);
				}
			}
			Value::Int(v) => match op {
				ArithOp::Add => addicative_part = addicative_part.wrapping_add(v),
				ArithOp::Sub => addicative_part = addicative_part.wrapping_sub(v),
				ArithOp::Mul => multi_part = multi_part.wrapping_mul(v),
				_ => (),
			},
			_ => {
				unreachable!();
			}
		}
	}

	assert_eq!(info.len(), 0);

	if addicative_part != 0 {
		info.push((ArithOp::Add, addicative_part.into(), 1));
	}

	if multi_part != 1 {
		info.push((ArithOp::Mul, multi_part.into(), 1));
	}

	for (key, times) in variable_part.into_iter() {
		match key.0 {
			ArithOp::Add | ArithOp::Sub | ArithOp::Mul => {
				if times != 0 {
					info.push((key.0, Value::Temp(key.1), times));
				}
			}
			_ => {
				for _ in 0..times {
					info.push((key.0, Value::Temp(key.1.clone()), 1));
				}
			}
		}
	}
}

fn add_expression_tree(
	next_temp: u32,
	instrs_list: &mut Vec<Box<dyn LlvmInstrTrait>>,
	target: Temp,
	target_type: VarType,
	arith_type: ArithType,
	info: &Vec<(ArithOp, Value, i32)>,
) -> u32 {
	assert_eq!(target_type, llvm::VarType::I32);
	match arith_type {
		ArithType::Addicative => {
			add_addicative_tree(next_temp, instrs_list, target, info, target_type)
		}
		ArithType::Multiplitive => {
			add_multiplitive_tree(next_temp, instrs_list, target, info, target_type)
		}
	}
}

impl RrvmOptimizer for LocalExpressionRearrangement {
	//这是一个基本块内的优化
	/*
		 对于i32类型的加减法、乘法，
		 寻找能apply交换律的最大的边界，
		 如2 + a + c + 3 + b + 4
				 1 先把常量折叠成一个点
				 2 再排成 9 + a + b + c (不知道常量的temp，给一个全序，例如其自身的排序)
				 3 排成一个尽量平衡的树
	*/

	fn new() -> Self {
		Self {}
	}

	fn apply(self, program: &mut LlvmProgram) -> Result<bool> {
		fn solve(cfg: &mut LlvmCFG, next_temp: u32) -> (bool, u32) {
			cfg.analysis();
			// println!("next_temp:{:?}", next_temp);
			let mut temp_counter = next_temp;
			// let mut current_out = HashSet::new();
			type ArithMap = HashMap<String, (ArithType, Vec<(ArithOp, Value, i32)>)>;
			let mut current_i32_calculation: ArithMap = HashMap::new();
			for item in cfg.blocks.as_slice() {
				current_i32_calculation.clear();
				if item.borrow().instrs.len() > 20000 {
					continue;
				}
				let mut new_instr = vec![];
				for instr in &item.borrow_mut().instrs {
					// println!("{:#}", &instr.get_variant());
					let mut flag = true;

					if let llvm::LlvmInstrVariant::ArithInstr(arith) =
						&instr.get_variant()
					{
						if arith.var_type == VarType::I32 {
							match arith.op {
								ArithOp::Mul | ArithOp::Add | ArithOp::Sub => {
									let mut to_insert = vec![];

									let neg = arith.op == ArithOp::Sub;

									match &arith.lhs {
										Value::Int(_) => {
											to_insert.push((
												get_op_for_lhs(arith.op),
												arith.lhs.clone(),
												1,
											));
										}
										Value::Float(_) => unreachable!(),
										Value::Temp(t) => {
											if let Some((child_type, vec)) =
												current_i32_calculation.get(&t.name)
											{
												if get_arith_type(&arith.op) == *child_type {
													let mut cloned_vec = vec.clone();
													to_insert.append(&mut cloned_vec);
												} else {
													to_insert.push((
														get_op_for_lhs(arith.op),
														arith.lhs.clone(),
														1,
													));
												}
											} else {
												//atom!
												to_insert.push((
													get_op_for_lhs(arith.op),
													arith.lhs.clone(),
													1,
												));
											}
										}
									}

									match &arith.rhs {
										Value::Int(_) => {
											to_insert.push((arith.op, arith.rhs.clone(), 1));
										}
										Value::Float(_) => unreachable!(),
										Value::Temp(t) => {
											if let Some((child_type, vec)) =
												current_i32_calculation.get(&t.name)
											{
												if get_arith_type(&arith.op) == *child_type {
													let mut cloned_vec = vec.clone();
													if neg {
														for item in cloned_vec {
															to_insert.push((
																get_neg(item.0),
																item.1.clone(),
																item.2,
															));
														}
													} else {
														to_insert.append(&mut cloned_vec);
													}
												} else {
													to_insert.push((arith.op, arith.rhs.clone(), 1));
												}
											} else {
												//atom!
												to_insert.push((arith.op, arith.rhs.clone(), 1));
											}
										}
									}

									squash(&mut to_insert);

									// println!("{:#}", &instr);
									temp_counter = add_expression_tree(
										temp_counter,
										&mut new_instr,
										arith.target.clone(),
										arith.var_type,
										get_arith_type(&arith.op),
										&to_insert,
									);
									current_i32_calculation.insert(
										arith.target.name.clone(),
										(get_arith_type(&arith.op), to_insert),
									);
									flag = false;
								}
								_ => {}
							}
						}
					}

					if flag {
						new_instr.push(instr.clone_box());
					}
				}

				item.borrow_mut().instrs = new_instr;

				// println!("in  :{:?}\n\n", &current_i32_calculation);
			}
			(false, temp_counter)
		}

		Ok(program.funcs.iter_mut().fold(false, |_last, func| {
			let (valid, tmp_count) = solve(&mut func.cfg, program.next_temp);
			program.next_temp = tmp_count;
			valid
		}))
	}
}
