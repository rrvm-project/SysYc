use crate::{context::IRPassContext, irpass::IRPass};
use llvm::{
	cfg::CFG,
	llvmop::{is_commutative, ArithOp, Value},
	ArithInstr, LlvmInstr, LlvmProgram, Temp,
};

use std::collections::{HashMap, HashSet};

pub struct Svn {
	last_value_number: usize,
}

impl Svn {
	pub fn new() -> Self {
		Svn {
			last_value_number: 0,
		}
	}
}

impl Default for Svn {
	fn default() -> Self {
		Self::new()
	}
}

impl IRPass for Svn {
	fn pass(&mut self, program: &mut LlvmProgram, context: &mut IRPassContext) {
		for item in &mut program.funcs {
			let cfg = &mut item.cfg;
			println!("\n\nfor function {}", item.label);
			self.traverse_cfg(cfg, context);
		}
	}
}
#[derive(PartialEq, Eq, Hash, Debug)]
enum LvnValueItem {
	LLValue(Value),
	Exp((String, Vec<usize>)),
}
impl LvnValueItem {
	pub fn get_value(self) -> Option<Value> {
		match self {
			LvnValueItem::LLValue(v) => Some(v),
			LvnValueItem::Exp(_) => None,
		}
	}
}

#[derive(Debug)]
#[allow(dead_code)]
struct BasicBlockLvnData {
	pub id: usize,
	pub value_to_number: HashMap<LvnValueItem, usize>,
	pub number_repr_value: HashMap<usize, Value>,
}

impl BasicBlockLvnData {
	fn new(id: usize) -> Self {
		BasicBlockLvnData {
			id,
			value_to_number: HashMap::new(),
			number_repr_value: HashMap::new(),
		}
	}
}

pub fn identity_arithinstr_detect(instr: &ArithInstr) -> Option<Value> {
	let (op, lhs, rhs) = (&instr.op, &instr.lhs, &instr.rhs);
	match op {
		//TODO add more here
		//Note that xor is logical, not bitwise!
		ArithOp::Add => {
			if lhs == &Value::Int(0) {
				Some(rhs.clone())
			} else if rhs == &Value::Int(0) {
				Some(lhs.clone())
			} else {
				None
			}
		}
		ArithOp::Mul => {
			if lhs == &Value::Int(1) {
				Some(rhs.clone())
			} else if rhs == &Value::Int(1) {
				Some(lhs.clone())
			} else {
				None
			}
		}
		ArithOp::Fadd => {
			if lhs == &Value::Float(0.0) {
				Some(rhs.clone())
			} else if rhs == &Value::Float(0.0) {
				Some(lhs.clone())
			} else {
				None
			}
		}
		ArithOp::Fmul => {
			if lhs == &Value::Float(1.0) {
				Some(rhs.clone())
			} else if rhs == &Value::Float(1.0) {
				Some(lhs.clone())
			} else {
				None
			}
		}
		_ => None,
	}
}

impl Svn {
	fn find_value_number(
		&self,
		value_item: &LvnValueItem,
		table: &mut [BasicBlockLvnData],
	) -> Option<usize> {
		assert!(!table.is_empty(), "lvn 表不能为空！");
		for item in table.as_ref().iter() {
			if let Some(number) = item.value_to_number.get(value_item) {
				return Some(*number);
			}
		}
		None
	}

	fn look_up_lvn_value_item(
		&mut self,
		value_item: LvnValueItem,
		table: &mut [BasicBlockLvnData],
	) -> usize {
		if let Some(number) = self.find_value_number(&value_item, table) {
			return number;
		}

		self.last_value_number += 1;

		if let LvnValueItem::LLValue(value) = &value_item {
			table
				.last_mut()
				.unwrap()
				.number_repr_value
				.insert(self.last_value_number, value.clone());
		}

		table
			.last_mut()
			.unwrap()
			.value_to_number
			.insert(value_item, self.last_value_number);

		self.last_value_number
	}

	fn link_value_number_to_value(
		&self,
		table: &mut [BasicBlockLvnData],
		value: Value,
		number: usize,
	) -> Option<usize> {
		table
			.last_mut()
			.unwrap()
			.value_to_number
			.insert(LvnValueItem::LLValue(value), number)
	}

	fn get_value_from_value_number(
		&self,
		table: &[BasicBlockLvnData],
		number: usize,
	) -> Option<Value> {
		for item in table.as_ref().iter() {
			if let Some(value) = item.number_repr_value.get(&number) {
				return Some(value.clone());
			}
		}
		None
	}

	fn link_tmp_to_value_number(
		&self,
		table: &mut [BasicBlockLvnData],
		tmp: Temp,
		number: usize,
	) {
		let lvn_value_item = LvnValueItem::LLValue(Value::Temp(tmp));

		if self.get_value_from_value_number(table, number).is_none() {
			table
				.last_mut()
				.unwrap()
				.number_repr_value
				.insert(number, lvn_value_item.get_value().unwrap());
		}
	}

	fn look_up_value_for_value_number(
		&mut self,
		value: &Value,
		table: &mut [BasicBlockLvnData],
	) -> usize {
		self.look_up_lvn_value_item(LvnValueItem::LLValue(value.clone()), table)
	}

	fn look_up_expression_for_value_number(
		&mut self,
		op: String,
		value_numbers: Vec<usize>,
		table: &mut [BasicBlockLvnData],
	) -> usize {
		self.look_up_lvn_value_item(
			LvnValueItem::Exp((op.clone(), value_numbers)),
			table,
		)
	}

	fn traverse_cfg(&mut self, cfg: &mut CFG, ctx: &mut IRPassContext) {
		let mut work_list = vec![cfg.entry];
		let mut visited = HashSet::<usize>::new();

		let mut id;
		let mut lvn_data: Vec<BasicBlockLvnData> = vec![];

		while !work_list.is_empty() {
			id = work_list.pop().unwrap();
			self.svn(id, cfg, &mut work_list, &mut lvn_data, &mut visited, ctx);
		}
	}

	fn svn(
		&mut self,
		id: usize,
		cfg: &mut CFG,
		work_list: &mut Vec<usize>,
		lvn_data: &mut Vec<BasicBlockLvnData>,
		visited: &mut HashSet<usize>,
		ctx: &mut IRPassContext,
	) {
		lvn_data.push(BasicBlockLvnData::new(id));

		visited.insert(id);
		self.lvn(id, cfg, lvn_data, ctx);

		let mut to_visit_next = vec![];

		for succ in &cfg.basic_blocks.get(&id).unwrap().succ {
			if cfg.basic_blocks.get(succ).unwrap().pred.len() == 1 {
				to_visit_next.push(*succ);
			} else if !visited.contains(succ) {
				work_list.push(*succ);
				println!("pushed into worklist {}", *succ);
			}
		}

		for item in to_visit_next {
			self.svn(item, cfg, work_list, lvn_data, visited, ctx);
		}

		lvn_data.pop();
	}

	#[allow(dead_code)]
	fn lvn(
		&mut self,
		id: usize,
		cfg: &mut CFG,
		lvn_data: &mut [BasicBlockLvnData],
		_: &mut IRPassContext,
	) {
		// dbg!(&lvn_data);
		if let Some(mut basicblock) = cfg.basic_blocks.remove(&id) {
			let instrs = std::mem::take(&mut basicblock.instrs);
			let mut new_instrs: Vec<Box<dyn LlvmInstr>> = Vec::new();

			for item in instrs {
				let mut do_nothing = false;
				match item.get_variant() {
					llvm::LlvmInstrVariant::ArithInstr(instr) => {
						if let Some(value_on_right) = identity_arithinstr_detect(instr) {
							println!("Marked as assign: {:#}", instr);
							let value_number_for_right =
								self.look_up_value_for_value_number(&value_on_right, lvn_data);
							println!("right value number {}", value_number_for_right);
							if let Some(old) = self.link_value_number_to_value(
								lvn_data,
								Value::Temp(instr.target.clone()),
								value_number_for_right,
							) {
								panic!("ssa violated on {:?} in {:#} with original value number {} and new one {}", instr.target, instr, old, value_number_for_right);
							}
							let value_to_assign = if let Some(value) = self
								.get_value_from_value_number(lvn_data, value_number_for_right)
							{
								value
							} else {
								value_on_right
							};
							match instr.target.var_type {
								llvm::llvmvar::VarType::I32 => {
									new_instrs.push(Box::new(llvm::ArithInstr {
										target: instr.target.clone(),
										op: ArithOp::Add,
										var_type: llvm::llvmvar::VarType::I32,
										lhs: value_to_assign,
										rhs: Value::Int(0),
									}))
								}
								llvm::llvmvar::VarType::F32 => {
									new_instrs.push(Box::new(llvm::ArithInstr {
										target: instr.target.clone(),
										op: ArithOp::Fadd,
										var_type: llvm::llvmvar::VarType::F32,
										lhs: value_to_assign,
										rhs: Value::Float(0.0),
									}))
								}
								_ => do_nothing = true,
							}
						} else {
							let op_name = instr.op.to_string();
							println!("Marked as normal: {:#}", instr);
							let (value_number_l, value_number_r) = (
								self.look_up_value_for_value_number(&instr.lhs, lvn_data),
								self.look_up_value_for_value_number(&instr.rhs, lvn_data),
							);
							let mut value_numbers = vec![value_number_l, value_number_r];
							if is_commutative(&instr.op) {
								value_numbers.sort();
							}
							let value_number_for_expression = self
								.look_up_expression_for_value_number(
									op_name,
									value_numbers,
									lvn_data,
								);

							if let Some(value_to_assign) = self.get_value_from_value_number(
								lvn_data,
								value_number_for_expression,
							) {
								match instr.target.var_type {
									llvm::llvmvar::VarType::I32 => {
										new_instrs.push(Box::new(llvm::ArithInstr {
											target: instr.target.clone(),
											op: ArithOp::Add,
											var_type: llvm::llvmvar::VarType::I32,
											lhs: value_to_assign,
											rhs: Value::Int(0),
										}))
									}
									llvm::llvmvar::VarType::F32 => {
										new_instrs.push(Box::new(llvm::ArithInstr {
											target: instr.target.clone(),
											op: ArithOp::Fadd,
											var_type: llvm::llvmvar::VarType::F32,
											lhs: value_to_assign,
											rhs: Value::Float(0.0),
										}))
									}
									_ => do_nothing = true,
								}
								self.link_value_number_to_value(
									lvn_data,
									Value::Temp(instr.target.clone()),
									value_number_for_expression,
								);
							} else {
								println!(
									"value_number {} not found ",
									value_number_for_expression
								);
								self.link_value_number_to_value(
									lvn_data,
									Value::Temp(instr.target.clone()),
									value_number_for_expression,
								);
								self.link_tmp_to_value_number(
									lvn_data,
									instr.target.clone(),
									value_number_for_expression,
								);
								println!("{:?}", &lvn_data);
								new_instrs.push(Box::new(llvm::ArithInstr {
									target: instr.target.clone(),
									op: instr.op,
									var_type: instr.var_type,
									lhs: self
										.get_value_from_value_number(lvn_data, value_number_l)
										.unwrap(), // previous lookuping for the value numbers asserts this
									rhs: self
										.get_value_from_value_number(lvn_data, value_number_r)
										.unwrap(),
								}));
							}
						}
					}

					// llvm::LlvmInstrVariant::GEPInstr(instr) => {
					// 	// let (target, addr, offset) = (&instr.target, &instr.addr, &instr.offset);

					// 	// let (value_number_l, value_number_r) = (self.look_up_value_for_value_number(&addr, lvn_data), self.look_up_value_for_value_number(&offset, lvn_data));

					// 	do_nothing = true;
					// }
					// llvm::LlvmInstrVariant::LabelInstr(_) => todo!(),
					// llvm::LlvmInstrVariant::CompInstr(_) => todo!(),
					// llvm::LlvmInstrVariant::ConvertInstr(_) => todo!(),
					// llvm::LlvmInstrVariant::JumpInstr(_) => todo!(),
					// llvm::LlvmInstrVariant::JumpCondInstr(_) => todo!(),
					// llvm::LlvmInstrVariant::PhiInstr(_) => todo!(),
					// llvm::LlvmInstrVariant::RetInstr(_) => todo!(),
					// llvm::LlvmInstrVariant::AllocInstr(_) => todo!(),
					// llvm::LlvmInstrVariant::StoreInstr(_) => todo!(),
					// llvm::LlvmInstrVariant::LoadInstr(_) => todo!(),
					// llvm::LlvmInstrVariant::GEPInstr(_) => todo!(),
					// llvm::LlvmInstrVariant::CallInstr(_) => todo!(),
					_ => {
						do_nothing = true;
					}
				}
				if do_nothing {
					new_instrs.push(item);
				}
			}

			// instrs.push(Box::new(llvm::ArithInstr{ target: Temp::new(114515, llvm::llvmvar::VarType::F32), op: ArithOp::Fadd, var_type: llvm::llvmvar::VarType::F32, lhs: Value::Float(0.3), rhs: Value::Float(7.66) }));

			basicblock.instrs = new_instrs;
			cfg.basic_blocks.insert(id, basicblock);
		}
	}
}
