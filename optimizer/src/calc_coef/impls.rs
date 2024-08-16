use super::CalcCoef;
use crate::{MetaData, RrvmOptimizer};
use core::panic;
use llvm::{
	compute_two_value,
	ArithOp::{And, Or, Rem, Xor},
	LlvmInstr, LlvmInstrTrait,
	LlvmInstrVariant::{
		AllocInstr, ArithInstr, CallInstr, CompInstr, ConvertInstr, GEPInstr,
		JumpCondInstr, LoadInstr, PhiInstr, StoreInstr,
	},
	LlvmTemp, LlvmTempManager, RetInstr,
	Value::{self, Temp},
	VarType,
};
use rrvm::{
	cfg::{BasicBlock, CFG},
	func,
	program::{LlvmFunc, LlvmProgram},
};
use std::{
	cell::RefCell,
	collections::{HashMap, HashSet, VecDeque},
	io::{self, Write},
	mem,
	rc::Rc,
	vec,
};
use utils::{errors::Result, Label};
impl RrvmOptimizer for CalcCoef {
	fn new() -> Self {
		Self {}
	}
	fn apply(
		self,
		program: &mut LlvmProgram,
		metadata: &mut MetaData,
	) -> Result<bool> {
		let old_len = program.funcs.len();
		let new_funcs: Vec<_> = mem::take(&mut program.funcs)
			.into_iter()
			.flat_map(|func| {
				if let Some((index, recurse_index)) = can_calc(&func) {
					calc_coef(&func, index, &mut program.temp_mgr, recurse_index)
				} else {
					vec![func]
				}
			})
			.collect();
		program.funcs = new_funcs;
		Ok(old_len != program.funcs.len())
	}
}
fn can_calc(func: &LlvmFunc) -> Option<(LlvmTemp, Box<dyn LlvmInstrTrait>)> {
	if func.params.len() == 2 {
		// check if recursive call and has (every) branch based on the index parameter (此处加强了条件)
		let mut param: Option<llvm::LlvmTemp> = None;
		let mut has_recursion = false;
		for block in func.cfg.blocks.iter() {
			for instr in block.borrow().instrs.iter() {
				if let CallInstr(callinstr) = instr.get_variant() {
					if callinstr.func.name == func.name {
						has_recursion = true;
						break;
					}
				}
			}
		}
		// 判断是否纯函数，检查有无语句是 store/gep/alloca 对于 load 全局变量这种就可以忽略
		for block in func.cfg.blocks.iter() {
			for instr in block.borrow().instrs.iter() {
				if let GEPInstr(_) | AllocInstr(_) | StoreInstr(_) | LoadInstr(_) =
					instr.get_variant()
				{
					// TODO 这里先不考虑全局变量，之后再加
					return None;
				} else if let ArithInstr(instr) = instr.get_variant() {
					if Rem == instr.op
						|| And == instr.op
						|| Or == instr.op
						|| Xor == instr.op
					{
						return None;
					}
				}
			}
		}
		if has_recursion {
			let mut jmp_conds = HashSet::new();
			for block in func.cfg.blocks.iter().rev() {
				// 找 branch 指令，找最后一次写他的指令 如果是参数是 param 或者是 param 和别人比较得到的结果就行
				let block = &block.borrow();
				let mut jmp_instr = {
					if let Some(instr) = block.jump_instr.clone() {
						vec![instr]
					} else {
						vec![]
					}
				};
				for i in block.instrs.iter().chain(jmp_instr.iter()).rev() {
					if let JumpCondInstr(jmp_cond) = i.get_variant() {
						if let Value::Temp(t) = &jmp_cond.cond {
							if !func.params.contains(&jmp_cond.cond) {
								jmp_conds.insert(t.clone());
							}
						}
					}
					if let Some(t) = i.get_write() {
						if jmp_conds.contains(&t) {
							jmp_conds.remove(&t);
							// 进行检查
							if let CompInstr(compinstr) = i.get_variant() {
								if let Value::Temp(t) = &compinstr.lhs {
									io::stderr().flush().unwrap();
									if func.params.contains(&compinstr.lhs) {
										if let Some(ref p) = param {
											if *p != *t {
												return None;
											}
										} else {
											param = Some(t.clone());
										}
									}
								} else if let Value::Temp(t) = &compinstr.rhs {
									if func.params.contains(&compinstr.rhs) {
										if let Some(ref p) = param {
											if *p != *t {
												return None;
											}
										} else {
											param = Some(t.clone());
										}
									}
								}
							}
						}
					}
				}
			}
			if let Some(p) = param {
				// 检查满不满足那个在所有递归 call 函数中不变的条件，1. 有一样的变量，2. 该变量的写是和 index 有关而且和另一个参数为常数
				// 找到所有 call 指令
				let mut call_instrs = vec![];
				for block in func.cfg.blocks.iter() {
					for instr in block.borrow().instrs.iter() {
						if let CallInstr(callinstr) = instr.get_variant() {
							if callinstr.func.name == func.name {
								call_instrs.push(callinstr.clone());
							}
						}
					}
				}
				let params =
					call_instrs.iter().map(|x| x.params.clone()).collect::<Vec<_>>();
				// 每对 param 要不都是常数 要不有一个相同的元素
				let mut element = HashSet::new();
				for param in params.iter() {
					let param1 = param[0].1.clone();
					let param2 = param[1].1.clone();
					let mut param_set = HashSet::new();
					if let Value::Temp(t) = param1 {
						param_set.insert(t);
					}
					if let Value::Temp(t) = param2 {
						param_set.insert(t);
					}
					if param_set.is_empty() {
						continue;
					}
					if element.is_empty() {
						element = param_set;
					} else if element.len() == 1 {
						if !param_set.contains(&element.iter().next().unwrap()) {
							return None;
						}
					} else {
						element =
							element.intersection(&param_set).map(|x| x.clone()).collect();
					}
				}
				if element.is_empty() {
					return None;
				}
				let tmp = element.iter().next().unwrap();
				let mut my_instr = None;
				// 检查写的地方 read 是否含有 p
				for block in func.cfg.blocks.iter() {
					for instr in block.borrow().instrs.iter() {
						let write = instr.get_write();
						if let Some(write_tmp) = write {
							if *tmp == write_tmp {
								my_instr = Some(instr.clone());
								let read = instr.get_read();
								if !read.iter().any(|x| *x == p.clone()) {
									return None;
								}
								break;
							}
						}
					}
				}
				return Some((p, my_instr.unwrap().clone()));
			}
		}
	}
	None
}
fn map_instr(
	instr: &Box<dyn LlvmInstrTrait>,
	entry_map: &mut HashMap<LlvmTemp, Entry>,
	block_instrs: &mut Vec<Box<dyn LlvmInstrTrait>>,
	mgr: &mut LlvmTempManager,
	block_phi_instrs: &mut Vec<llvm::PhiInstr>,
	data: &Value,
	addr: &LlvmTemp,
	func_name: String,
	recurse_index: &Box<dyn LlvmInstrTrait>,
) -> bool {
	match instr.get_variant() {
		ArithInstr(arith_instr) => {
			let lhs = arith_instr.lhs.clone();
			let rhs = arith_instr.rhs.clone();
			let target = arith_instr.target.clone();
			// 分类讨论 lhs 和 rhs 分别能否在 entry_map 中找到
			let get_lhs = {
				if let Value::Temp(t) = &lhs {
					entry_map.get(t)
				} else {
					None
				}
			};
			let get_rhs = {
				if let Value::Temp(t) = &rhs {
					entry_map.get(t)
				} else {
					None
				}
			};
			if get_lhs.is_none() && get_rhs.is_none() {
				// 直接调用 compute_two_value
				let (value, instr) = compute_two_value(lhs, rhs, arith_instr.op, mgr);
				if let Some(instr) = instr {
					block_instrs.push(Box::new(instr));
				}
				entry_map.insert(
					target,
					Entry {
						k_val: Value::Int(0),
						b_val: value,
						mod_val: None,
					},
				);
			} else if let Some(lhs_entry) = get_lhs {
				if let Some(rhs_entry) = get_rhs {
					if let llvm::ArithOp::Ashr
					| llvm::ArithOp::Div
					| llvm::ArithOp::Fdiv
					| llvm::ArithOp::Lshr
					| llvm::ArithOp::Shl = arith_instr.op
					{
						match get_rhs.unwrap().k_val {
							Value::Int(0) => {}
							Value::Float(0.0) => {}
							_ => {
								return false;
							}
						}
						let (val0, instr0) = compute_two_value(
							lhs,
							get_rhs.unwrap().b_val.clone(),
							arith_instr.op,
							mgr,
						);
						entry_map.insert(
							target,
							Entry {
								k_val: Value::Int(0),
								b_val: val0,
								mod_val: None,
							},
						);
						if let Some(instr) = instr0 {
							block_instrs.push(Box::new(instr));
						}
					} else if let llvm::ArithOp::Fmul | llvm::ArithOp::Mul =
						arith_instr.op
					{
						let mut is_lhs_const = false;
						match get_rhs.unwrap().k_val {
							Value::Int(0) => {}
							Value::Float(0.0) => {}
							_ => {
								match get_lhs.unwrap().k_val {
									Value::Int(0) => {}
									Value::Float(0.0) => {}
									_ => {
										return false;
									}
								}
								is_lhs_const = true;
							}
						}
						if !is_lhs_const {
							let (val0, instr0) = compute_two_value(
								lhs_entry.k_val.clone(),
								rhs_entry.b_val.clone(),
								arith_instr.op,
								mgr,
							);
							let (val1, instr1) = compute_two_value(
								lhs_entry.b_val.clone(),
								rhs_entry.b_val.clone(),
								arith_instr.op,
								mgr,
							);
							entry_map.insert(
								target,
								Entry {
									k_val: val0,
									b_val: val1,
									mod_val: None,
								},
							);
							if let Some(instr0) = instr0 {
								block_instrs.push(Box::new(instr0));
							}
							if let Some(instr1) = instr1 {
								block_instrs.push(Box::new(instr1));
							}
						} else {
							let (val0, instr0) = compute_two_value(
								lhs_entry.b_val.clone(),
								rhs_entry.k_val.clone(),
								arith_instr.op,
								mgr,
							);
							let (val1, instr1) = compute_two_value(
								lhs_entry.b_val.clone(),
								rhs_entry.b_val.clone(),
								arith_instr.op,
								mgr,
							);
							entry_map.insert(
								target,
								Entry {
									k_val: val0,
									b_val: val1,
									mod_val: None,
								},
							);
							if let Some(instr0) = instr0 {
								block_instrs.push(Box::new(instr0));
							}
							if let Some(instr1) = instr1 {
								block_instrs.push(Box::new(instr1));
							}
						}
					} else {
						let (val0, instr0) = compute_two_value(
							lhs_entry.k_val.clone(),
							rhs_entry.k_val.clone(),
							arith_instr.op,
							mgr,
						);
						let (val1, instr1) = compute_two_value(
							lhs_entry.b_val.clone(),
							rhs_entry.b_val.clone(),
							arith_instr.op,
							mgr,
						);
						entry_map.insert(
							target,
							Entry {
								k_val: val0,
								b_val: val1,
								mod_val: None,
							},
						);
						if let Some(instr0) = instr0 {
							block_instrs.push(Box::new(instr0));
						}
						if let Some(instr1) = instr1 {
							block_instrs.push(Box::new(instr1));
						}
					}
				} else {
					// lhs 是 entry_map 中的
					let (val0, instr0) = compute_two_value(
						lhs_entry.k_val.clone(),
						rhs.clone(),
						arith_instr.op,
						mgr,
					);
					let (val1, instr1) = compute_two_value(
						lhs_entry.b_val.clone(),
						rhs,
						arith_instr.op,
						mgr,
					);
					entry_map.insert(
						target,
						Entry {
							k_val: val0,
							b_val: val1,
							mod_val: None,
						},
					);
					if let Some(instr0) = instr0 {
						block_instrs.push(Box::new(instr0));
					}
					if let Some(instr1) = instr1 {
						block_instrs.push(Box::new(instr1));
					}
				}
			} else {
				// rhs 是 entry_map 中的
				// 先判断是否是可以直接终止计算的特殊情况
				if let llvm::ArithOp::Ashr
				| llvm::ArithOp::Div
				| llvm::ArithOp::Fdiv
				| llvm::ArithOp::Fmul
				| llvm::ArithOp::Lshr
				| llvm::ArithOp::Mul
				| llvm::ArithOp::Shl = arith_instr.op
				{
					match get_rhs.unwrap().k_val {
						Value::Int(0) => {}
						Value::Float(0.0) => {}
						_ => {
							return false;
						}
					}
					let (val0, instr0) = compute_two_value(
						lhs,
						get_rhs.unwrap().b_val.clone(),
						arith_instr.op,
						mgr,
					);
					entry_map.insert(
						target,
						Entry {
							k_val: Value::Int(0),
							b_val: val0,
							mod_val: None,
						},
					);
					if let Some(instr) = instr0 {
						block_instrs.push(Box::new(instr));
					}
				} else {
					let (val0, instr0) = compute_two_value(
						lhs.clone(),
						get_rhs.unwrap().k_val.clone(),
						arith_instr.op,
						mgr,
					);
					let (val1, instr1) = compute_two_value(
						lhs,
						get_rhs.unwrap().b_val.clone(),
						arith_instr.op,
						mgr,
					);
					entry_map.insert(
						target,
						Entry {
							k_val: val0,
							b_val: val1,
							mod_val: None,
						},
					);
					if let Some(instr0) = instr0 {
						block_instrs.push(Box::new(instr0));
					}
					if let Some(instr1) = instr1 {
						block_instrs.push(Box::new(instr1));
					}
				}
			}
		}
		CompInstr(comp_instr) => {
			// 要求 lhs rhs 的 data 系数为 0
			let lhs = comp_instr.lhs.clone();
			let rhs = comp_instr.rhs.clone();
			let target = comp_instr.target.clone();
			let get_lhs_val = {
				if let Value::Temp(t) = lhs {
					let entry = entry_map.get(&t);
					if let Some(entry) = entry {
						if let Value::Int(0) | Value::Float(0.0) = entry.k_val {
							Some(entry.b_val.clone())
						} else {
							None
						}
					} else {
						None
					}
				} else {
					Some(lhs.clone())
				}
			};
			let get_rhs_val = {
				if let Value::Temp(t) = rhs {
					let entry = entry_map.get(&t);
					if let Some(entry) = entry {
						if let Value::Int(0) | Value::Float(0.0) = entry.k_val {
							Some(entry.b_val.clone())
						} else {
							None
						}
					} else {
						None
					}
				} else {
					Some(rhs.clone())
				}
			};
			if let Some(lhs_val) = get_lhs_val {
				if let Some(rhs_val) = get_rhs_val {
					let my_target = mgr.new_temp(llvm::VarType::I32, false);
					let instr = llvm::CompInstr {
						target: my_target.clone(),
						lhs: lhs_val.clone(),
						rhs: rhs_val,
						op: comp_instr.op,
						var_type: lhs_val.get_type(),
						kind: comp_instr.kind,
					};
					block_instrs.push(Box::new(instr));
					entry_map.insert(
						target,
						Entry {
							k_val: Value::Int(0),
							b_val: Value::Temp(my_target),
							mod_val: None,
						},
					);
				} else {
					return false;
				}
			} else {
				return false;
			}
		}
		ConvertInstr(convert_instr) => {
			// 也是要求 lhs 和 data 无关
			let target = convert_instr.target.clone();
			let lhs = convert_instr.lhs.clone();
			let get_lhs_val = {
				if let Value::Temp(t) = lhs {
					let entry = entry_map.get(&t);
					if let Some(entry) = entry {
						if let Value::Int(0) | Value::Float(0.0) = entry.k_val {
							Some(entry.b_val.clone())
						} else {
							None
						}
					} else {
						panic!("convert instr lhs not in entry map");
					}
				} else {
					Some(lhs.clone())
				}
			};
			if let Some(lhs_val) = get_lhs_val {
				let my_target = mgr.new_temp(convert_instr.var_type, false);
				let instr = llvm::ConvertInstr {
					target: my_target.clone(),
					op: convert_instr.op,
					lhs: lhs_val,
					var_type: convert_instr.var_type,
				};
				block_instrs.push(Box::new(instr));
				entry_map.insert(
					target,
					Entry {
						k_val: Value::Int(0),
						b_val: Value::Temp(my_target),
						mod_val: None,
					},
				);
			} else {
				return false;
			}
		}
		JumpCondInstr(jump_cond_instr) => {
			// 同上要求 cond 和 data 无关
			let cond = jump_cond_instr.cond.clone();
			let get_cond_val = {
				if let Value::Temp(t) = cond {
					let entry = entry_map.get(&t);
					if let Some(entry) = entry {
						if let Value::Int(0) | Value::Float(0.0) = entry.k_val {
							Some(entry.b_val.clone())
						} else {
							None
						}
					} else {
						panic!("jump cond instr cond not in entry map");
					}
				} else {
					Some(cond.clone())
				}
			};
			if let Some(cond_val) = get_cond_val {
				let instr = llvm::JumpCondInstr {
					cond: cond_val.clone(),
					target_true: jump_cond_instr.target_true.clone(),
					target_false: jump_cond_instr.target_false.clone(),
					var_type: cond_val.get_type(),
				};
				block_instrs.push(Box::new(instr));
			}
		}
		PhiInstr(phi_instr) => {
			// 想一下怎么处理有 phi 的情况
			// 处理有 phi 的情况，搞成俩 phi
			let target = phi_instr.target.clone();
			let mut new_sources_k: Vec<_> = phi_instr
				.source
				.iter()
				.map(|(val, label)| {
					let get_val = {
						if let Value::Temp(t) = val {
							let entry = entry_map.get(&t);
							if let Some(entry) = entry {
								entry.k_val.clone()
							} else {
								panic!("phi instr val not in entry map");
							}
						} else {
							Value::Int(0)
						}
					};
					(get_val, label.clone())
				})
				.collect();
			let mut new_sources_b: Vec<_> = phi_instr
				.source
				.iter()
				.map(|(val, label)| {
					let get_val = {
						if let Value::Temp(t) = val {
							let entry = entry_map.get(&t);
							if let Some(entry) = entry {
								entry.b_val.clone()
							} else {
								panic!("phi instr val not in entry map");
							}
						} else {
							val.clone()
						}
					};
					(get_val, label.clone())
				})
				.collect();
			let k_target = mgr.new_temp(phi_instr.var_type, false);
			let b_target = mgr.new_temp(phi_instr.var_type, false);
			let instr1 = llvm::PhiInstr {
				target: k_target.clone(),
				source: new_sources_k.clone(),
				var_type: phi_instr.var_type,
			};
			let instr2 = llvm::PhiInstr {
				target: b_target.clone(),
				source: new_sources_b.clone(),
				var_type: phi_instr.var_type,
			};
			block_instrs.push(Box::new(instr1.clone()));
			block_instrs.push(Box::new(instr2.clone()));
			block_phi_instrs.push(instr1);
			block_phi_instrs.push(instr2);
			entry_map.insert(
				target,
				Entry {
					k_val: Value::Temp(k_target),
					b_val: Value::Temp(b_target),
					mod_val: None,
				},
			);
		}
		CallInstr(call_instr) => {
			// 检查是否是 call 的自身，如果不是的话，params 中都不能与 data 有关
			if call_instr.func.name != func_name {
				let params = call_instr.params.clone();
				let mut new_params = Vec::new();
				for param in params.iter() {
					let get_param_val = {
						if let Value::Temp(t) = &param.1 {
							let entry = entry_map.get(t);
							if let Some(entry) = entry {
								if let Value::Int(0) | Value::Float(0.0) = entry.k_val {
									Some(entry.b_val.clone())
								} else {
									None
								}
							} else {
								panic!("call instr param not in entry map");
							}
						} else {
							Some(param.1.clone())
						}
					};
					if let Some(param_val) = get_param_val {
						new_params.push((param.0, param_val));
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
				// 我们把 call 指令转成从 a 里面把 value load 出来再给到 call 的 dst 里面
				// eprintln!("before calling self {}",call_instr);
				// for i in block_instrs.iter() {
				// 	eprintln!("{}",i);
				// }
				// eprintln!("entry_map {:?}",entry_map);
				// io::stderr().flush().unwrap();
				let dst = call_instr.target.clone();
				let kdst = mgr.new_temp(data.get_type(), false);
				let bdst = mgr.new_temp(data.get_type(), false);
				let b_addr = mgr.new_temp(
					match bdst.var_type {
						llvm::VarType::I32 => llvm::VarType::I32Ptr,
						llvm::VarType::F32 => llvm::VarType::F32Ptr,
						_ => llvm::VarType::I32Ptr,
					},
					false,
				);
				let load1 = llvm::LoadInstr {
					target: kdst.clone(),
					var_type: data.get_type(),
					addr: Value::Temp(addr.clone()),
				};
				let gep_instr = llvm::GEPInstr {
					target: b_addr.clone(),
					var_type: b_addr.var_type,
					addr: Value::Temp(addr.clone()),
					offset: Value::Int(4),
				};
				let load2 = llvm::LoadInstr {
					target: bdst.clone(),
					var_type: data.get_type(),
					addr: Value::Temp(gep_instr.target.clone()),
				};
				// get param that is not recurse index and get its entry
				let my_recurse_index = recurse_index.get_write().unwrap();
				let my_data = call_instr
					.params
					.iter()
					.find(|x| x.1 != llvm::Value::Temp(my_recurse_index.clone()))
					.unwrap()
					.1
					.clone();
				if let Value::Temp(tmp) = my_data {
					let entry = entry_map.get(&tmp).unwrap();
					let (val0, instr1) = compute_two_value(
						llvm::Value::Temp(kdst.clone()),
						entry.k_val.clone(),
						match &kdst.var_type {
							llvm::VarType::I32 => llvm::ArithOp::Mul,
							llvm::VarType::F32 => llvm::ArithOp::Fmul,
							_ => llvm::ArithOp::Mul,
						},
						mgr,
					);
					let (val2, instr2) = compute_two_value(
						llvm::Value::Temp(kdst.clone()),
						entry.b_val.clone(),
						match &kdst.var_type {
							llvm::VarType::I32 => llvm::ArithOp::Mul,
							llvm::VarType::F32 => llvm::ArithOp::Fmul,
							_ => llvm::ArithOp::Mul,
						},
						mgr,
					);
					let (val3, instr3) = compute_two_value(
						llvm::Value::Temp(bdst.clone()),
						val2,
						match &bdst.var_type {
							llvm::VarType::I32 => llvm::ArithOp::Add,
							llvm::VarType::F32 => llvm::ArithOp::Fadd,
							_ => llvm::ArithOp::Add,
						},
						mgr,
					);
					block_instrs.push(Box::new(load1));
					block_instrs.push(Box::new(gep_instr));
					block_instrs.push(Box::new(load2));
					if let Some(instr1) = instr1 {
						block_instrs.push(Box::new(instr1));
					}
					if let Some(instr2) = instr2 {
						block_instrs.push(Box::new(instr2));
					}
					if let Some(instr3) = instr3 {
						block_instrs.push(Box::new(instr3));
					}
					entry_map.insert(
						dst,
						Entry {
							k_val: val0,
							b_val: val3,
							mod_val: None,
						},
					);
				} else {
					// fb+g
					let (val0, instr1) = compute_two_value(
						llvm::Value::Temp(kdst.clone()),
						my_data.clone(),
						match &kdst.var_type {
							llvm::VarType::I32 => llvm::ArithOp::Mul,
							llvm::VarType::F32 => llvm::ArithOp::Fmul,
							_ => llvm::ArithOp::Mul,
						},
						mgr,
					);
					let (val2, instr2) = compute_two_value(
						llvm::Value::Temp(bdst.clone()),
						val0,
						match &bdst.var_type {
							llvm::VarType::I32 => llvm::ArithOp::Add,
							llvm::VarType::F32 => llvm::ArithOp::Fadd,
							_ => llvm::ArithOp::Add,
						},
						mgr,
					);
					block_instrs.push(Box::new(load1));
					block_instrs.push(Box::new(gep_instr));
					block_instrs.push(Box::new(load2));
					if let Some(instr1) = instr1 {
						block_instrs.push(Box::new(instr1));
					}
					if let Some(instr2) = instr2 {
						block_instrs.push(Box::new(instr2));
					}
					entry_map.insert(
						dst,
						Entry {
							k_val: match &val2 {
								Value::Int(i) => Value::Int(0),
								Value::Float(f) => Value::Float(0.0),
								Value::Temp(t) => {
									if t.var_type == llvm::VarType::I32 {
										Value::Int(0)
									} else {
										Value::Float(0.0)
									}
								}
							},
							b_val: val2,
							mod_val: None,
						},
					);
				}
				// eprintln!("after calling self {}",call_instr);
				// for i in block_instrs.iter() {
				// 	eprintln!("{}",i);
				// }
				// eprintln!("entry_map {:?}",entry_map);
				// eprintln!("----------------------");
				// io::stderr().flush().unwrap();
			}
		}
		llvm::LlvmInstrVariant::RetInstr(retinstr) => {
			// 把 value 塞到 a 里面去
			// 注意我们是把 k_value 放在了上面
			let value = retinstr.value.clone();
			if let Some(val) = value {
				match val {
					Value::Temp(t) => {
						let entry = entry_map.get(&t);
						if let Some(entry) = entry {
							// store 进 a 里面去
							let gep1 = llvm::GEPInstr {
								target: mgr.new_temp(llvm::VarType::I32Ptr, false),
								var_type: llvm::VarType::I32Ptr,
								addr: Value::Temp(addr.clone()),
								offset: Value::Int(0),
							};
							let store1 = llvm::StoreInstr {
								value: entry.k_val.clone(),
								addr: Value::Temp(gep1.target.clone()),
							};
							let gep2 = llvm::GEPInstr {
								target: mgr.new_temp(llvm::VarType::I32Ptr, false),
								var_type: llvm::VarType::I32Ptr,
								addr: Value::Temp(addr.clone()),
								offset: Value::Int(4),
							};
							let store2 = llvm::StoreInstr {
								value: entry.b_val.clone(),
								addr: Value::Temp(gep2.target.clone()),
							};
							let ret = llvm::RetInstr { value: None };
							block_instrs.push(Box::new(gep1));
							block_instrs.push(Box::new(store1));
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
							value: val,
							addr: Value::Temp(gep_instr.target.clone()),
						};
						// 另一个 store 为0
						let store_instr2 = llvm::StoreInstr {
							value: Value::Int(0),
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
		}
		llvm::LlvmInstrVariant::JumpInstr(instr) => {
			block_instrs.push(Box::new(instr.clone()));
		}
		_ => {
			unreachable!("instr not supported");
		}
	}
	true
}
#[derive(Debug, Clone)]
struct Entry {
	k_val: Value,
	b_val: Value,
	mod_val: Option<Value>, // 这个先不考虑
}

fn map_coef_instrs(
	func: &LlvmFunc,
	index: LlvmTemp,
	addr: LlvmTemp,
	mgr: &mut LlvmTempManager,
	special_nodes: HashSet<i32>,
	recurse_index: Box<dyn LlvmInstrTrait>,
	my_index: LlvmTemp,
) -> Option<Vec<Rc<RefCell<BasicBlock<Box<dyn LlvmInstrTrait>, LlvmTemp>>>>> {
	let mut entry_map = HashMap::new();
	let mut data =
		func.params.iter().find(|x| **x != Value::Temp(index.clone())).unwrap();
	entry_map.insert(
		index.clone(),
		Entry {
			k_val: Value::Int(0),
			b_val: Value::Temp(my_index.clone()),
			mod_val: None,
		},
	);
	if let Value::Temp(t) = data {
		if VarType::I32 == t.var_type {
			entry_map.insert(
				t.clone(),
				Entry {
					k_val: Value::Int(1),
					b_val: Value::Int(0),
					mod_val: None,
				},
			);
		} else {
			entry_map.insert(
				t.clone(),
				Entry {
					k_val: Value::Float(1.0),
					b_val: Value::Float(0.0),
					mod_val: None,
				},
			);
		}
	}
	let mut new_instrs = vec![];
	let mut res_vec = vec![];
	let res = map_instr(
		&recurse_index,
		&mut entry_map,
		&mut res_vec,
		mgr,
		&mut vec![],
		data,
		&addr,
		func.name.clone(),
		&recurse_index,
	);
	let calc_recurse_instr = res_vec[0].clone();
	let my_recurse_index = calc_recurse_instr.get_write().unwrap();
	if !res {
		return None;
	}
	let mut call_instr = llvm::CallInstr {
		target: mgr.new_temp(llvm::VarType::I32, false),
		var_type: llvm::VarType::Void,
		func: Label::new(format!("{}_calc_coef", func.name)),
		params: vec![
			(llvm::VarType::I32Ptr, Value::Temp(addr.clone())),
			(llvm::VarType::I32, Value::Temp(my_recurse_index.clone())),
		],
	};
	let mut phi_instrs = vec![];
	let mut jmp_instrs = vec![];
	// 先把 data 和 index 放进entry_map 因为自有 Value 所以不用搞 instrs
	for (idx, block) in func.cfg.blocks.iter().enumerate() {
		let mut block_instrs: Vec<Box<dyn LlvmInstrTrait>> = vec![];
		let mut block_phi_instrs: Vec<llvm::PhiInstr> = vec![];
		if special_nodes.contains(&block.borrow().id) {
			block_instrs.push(calc_recurse_instr.clone());
			block_instrs.push(Box::new(call_instr.clone()));
		}
		let mut has_jmp = block.borrow().jump_instr.is_some();
		let jmp_vec = {
			if let Some(instr) = block.borrow().jump_instr.clone() {
				vec![instr]
			} else {
				vec![]
			}
		};
		for instr in block.borrow().instrs.iter().chain(jmp_vec.iter()) {
			let res = map_instr(
				instr,
				&mut entry_map,
				&mut block_instrs,
				mgr,
				&mut block_phi_instrs,
				data,
				&addr,
				func.name.clone(),
				&recurse_index,
			);
			if !res {
				return None;
			}
		}
		if has_jmp {
			let jmp_instr = block_instrs.pop().unwrap();
			jmp_instrs.push(Some(jmp_instr));
		} else {
			jmp_instrs.push(None);
		}
		phi_instrs.push(block_phi_instrs);
		new_instrs.push(block_instrs);
	}
	// assemble blocks with phi_instrs and new_instrs
	let mut new_blocks = vec![];
	for block in func
		.cfg
		.blocks
		.iter()
		.zip(new_instrs.iter())
		.zip(phi_instrs.iter())
		.zip(jmp_instrs.iter())
	{
		let (((block, instrs), phi_instrs), jmp_instr) = block;
		let mut new_block = block.clone();
		new_block.borrow_mut().instrs = instrs.clone();
		new_block.borrow_mut().phi_instrs = phi_instrs.clone();
		new_block.borrow_mut().jump_instr = jmp_instr.clone();
		new_blocks.push(new_block);
	}
	Some(new_blocks)
}
fn calc_coef(
	func: &LlvmFunc,
	index: LlvmTemp,
	mgr: &mut LlvmTempManager,
	recurse_index: Box<dyn LlvmInstrTrait>,
) -> Vec<LlvmFunc> {
	let data_val =
		func.params.iter().find(|x| **x != Value::Temp(index.clone())).unwrap();
	let data = {
		if let Value::Temp(t) = data_val {
			Some(t.clone())
		} else {
			None
		}
	};
	//  多源 bfs
	// 找到所有特殊点，即是有递归调用自身的点
	let mut special_node_ids = HashSet::new();
	let mut special_map = HashMap::new();
	for block in func.cfg.blocks.iter() {
		for instr in block.borrow().instrs.iter() {
			if let CallInstr(callinstr) = instr.get_variant() {
				if callinstr.func.name == func.name {
					special_node_ids.insert(block.borrow().id);
					special_map.insert(block.borrow().id, block.clone());
				}
			}
		}
	}
	// bfs 算特可达点
	loop {
		// calculate special reachables
		let mut special_reachables = HashSet::new();
		let mut special_reachable_map = HashMap::new();
		let mut queue = VecDeque::new();
		for node in special_node_ids.iter() {
			queue.push_back(special_map.get(node).unwrap().clone());
		}
		while let Some(node) = queue.pop_front() {
			if special_reachables.contains(&node.borrow().id) {
				continue;
			}
			special_reachables.insert(node.borrow().id);
			special_reachable_map.insert(node.borrow().id, node.clone());
			for succ in node.borrow().succ.iter() {
				queue.push_back(succ.clone());
			}
		}
		// calculate special nodes
		let mut new_special_nodes = HashSet::new();
		let mut new_special_map = HashMap::new();
		for node in special_node_ids.iter() {
			if !special_map
				.get(node)
				.unwrap()
				.borrow()
				.prev
				.iter()
				.any(|v| special_reachables.contains(&v.borrow().id))
			{
				new_special_nodes.insert(*node);
				new_special_map.insert(*node, special_map.get(node).unwrap().clone());
			} else if !special_map
				.get(node)
				.unwrap()
				.borrow()
				.prev
				.iter()
				.all(|v| special_reachables.contains(&v.borrow().id))
			{
				let borrowed_node = special_map.get(node).unwrap().borrow();
				let filtered_prevs = borrowed_node
					.prev
					.iter()
					.filter(|v| !special_reachables.contains(&v.borrow().id));
				new_special_nodes.extend(filtered_prevs.clone().map(|v| v.borrow().id));
				new_special_map
					.extend(filtered_prevs.map(|v| (v.borrow().id, v.clone())));
			}
		}
		let mut is_changed = special_node_ids.len() != new_special_nodes.len();
		for (val1, val2) in special_node_ids.iter().zip(new_special_nodes.iter()) {
			if *val1 != *val2 {
				is_changed = true;
				break;
			}
		}
		if !is_changed {
			break;
		} else {
			special_map = new_special_map;
			special_node_ids = new_special_nodes;
		}
	}

	// 对于每一个变量, todo 改成 load 和 gep 间隔
	let mut instrs: Vec<Box<dyn LlvmInstrTrait>> = vec![];
	let alloc_target = mgr.new_temp(llvm::VarType::I32Ptr, false);
	let alloc_instr = llvm::AllocInstr {
		target: alloc_target.clone(),
		length: Value::Int(16),
		var_type: llvm::VarType::I32Ptr,
	};
	let call_instr = llvm::CallInstr {
		target: mgr.new_temp(llvm::VarType::I32, false),
		var_type: llvm::VarType::Void,
		func: utils::Label {
			name: format!("{}_calc_coef", func.name),
		},
		params: vec![
			(llvm::VarType::I32, Value::Temp(index.clone())),
			(llvm::VarType::I32Ptr, Value::Temp(alloc_target.clone())),
		],
	};
	let f_tmp = mgr.new_temp(data.clone().unwrap().var_type, false);
	let load_f = llvm::LoadInstr {
		target: f_tmp.clone(),
		var_type: data.clone().unwrap().var_type,
		addr: Value::Temp(alloc_target.clone()),
	};
	let gep_dst = mgr.new_temp(llvm::VarType::I32Ptr, false);
	let gep_ptr = llvm::GEPInstr {
		target: gep_dst.clone(),
		var_type: llvm::VarType::I32Ptr,
		addr: Value::Temp(alloc_target.clone()),
		offset: Value::Int(4),
	};
	let g_tmp = mgr.new_temp(data.clone().unwrap().var_type, false);
	let load_g = llvm::LoadInstr {
		target: g_tmp.clone(),
		var_type: data.clone().unwrap().var_type,
		addr: Value::Temp(gep_dst),
	};
	let mul_dst = mgr.new_temp(data.clone().unwrap().var_type, false);
	let mul_instr = llvm::ArithInstr {
		target: mul_dst.clone(),
		var_type: data.clone().unwrap().var_type,
		lhs: Value::Temp(f_tmp),
		rhs: Value::Temp(data.clone().unwrap()),
		op: llvm::ArithOp::Mul,
	};
	let add_dst = mgr.new_temp(data.clone().unwrap().var_type, false);
	let add_instr = llvm::ArithInstr {
		target: add_dst.clone(),
		var_type: data.clone().unwrap().var_type,
		lhs: Value::Temp(g_tmp),
		rhs: Value::Temp(mul_dst),
		op: llvm::ArithOp::Add,
	};
	let ret_instr = llvm::RetInstr {
		value: Some(Value::Temp(add_dst)),
	};
	instrs.push(Box::new(alloc_instr));
	instrs.push(Box::new(call_instr));
	instrs.push(Box::new(load_f));
	instrs.push(Box::new(gep_ptr));
	instrs.push(Box::new(load_g));
	instrs.push(Box::new(mul_instr));
	instrs.push(Box::new(add_instr));

	let node = BasicBlock::new_node(0, 1.0);
	node.borrow_mut().instrs = instrs;
	node.borrow_mut().jump_instr = Some(Box::new(ret_instr));
	let mut wrapper_func = LlvmFunc {
		total: mgr.total as i32,
		spills: 0,
		cfg: CFG { blocks: vec![node] },
		name: func.name.clone(),
		ret_type: func.ret_type,
		params: func.params.clone(),
	};
	let mut addr =
		mgr.new_temp_with_name("addr".to_string(), llvm::VarType::I32Ptr);
	let mut my_index =
		mgr.new_temp_with_name("index".to_string(), index.var_type);
	let mut new_blocks = map_coef_instrs(
		func,
		index,
		addr.clone(),
		mgr,
		special_node_ids,
		recurse_index,
		my_index.clone(),
	);
	if new_blocks.is_none() {
		let new_func = LlvmFunc {
			total: func.total,
			spills: func.spills,
			cfg: CFG {
				blocks: func.cfg.blocks.clone(),
			},
			name: func.name.clone(),
			ret_type: func.ret_type,
			params: func.params.clone(),
		};
		return vec![new_func];
	}
	let mut calc_func = LlvmFunc {
		total: mgr.total as i32,
		spills: 0,
		cfg: rrvm::cfg::CFG {
			blocks: new_blocks.unwrap(),
		},
		name: format!("{}_calc_coef", func.name),
		ret_type: llvm::VarType::Void,
		params: vec![Value::Temp(addr.clone()), Value::Temp(my_index)],
	};
	vec![wrapper_func, calc_func]
}
