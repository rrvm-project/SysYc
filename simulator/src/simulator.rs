use std::{collections::HashMap, mem::transmute, vec};

use llvm::{
	ArithInstr, CloneLlvmInstr, CompInstr, LlvmInstrTrait, Value, VarType,
};
use rrvm::program::*;

use crate::inout::inout;

#[derive(Debug, Clone, Copy)]
pub enum StackValue {
	Int(i32),
	Float(f32),
	Ptr(usize),
}

impl Default for StackValue {
	fn default() -> Self {
		StackValue::Int(0)
	}
}

impl From<i32> for StackValue {
	fn from(value: i32) -> Self {
		StackValue::Int(value)
	}
}

impl From<f32> for StackValue {
	fn from(value: f32) -> Self {
		StackValue::Float(value)
	}
}

impl StackValue {
	pub fn as_i32(&self) -> i32 {
		match self {
			StackValue::Int(v) => *v,
			_ => unreachable!(),
		}
	}

	pub fn as_f32(&self) -> f32 {
		match self {
			StackValue::Float(v) => *v,
			_ => unreachable!(),
		}
	}

	pub fn as_usize(&self) -> usize {
		match self {
			StackValue::Ptr(v) => *v,
			_ => unreachable!(),
		}
	}
}

#[derive(Debug)]
pub struct FuncStackFrame {
	pub ra: Option<usize>,
	pub name: String,
	pub fp: usize,
	pub last_label: String,
	pub current_label: String,
	pub temp: HashMap<String, StackValue>,
	pub return_to: String,
}

pub struct MiddleSimulator {
	pub input: String,
	pub input_position: usize,
	pub output: Vec<String>,
	pub step_count: usize,
	pub instr_list: Vec<Box<dyn LlvmInstrTrait>>,
	pub label_map: HashMap<String, HashMap<String, usize>>,
	pub return_scratch: Option<StackValue>,
	pub pc: usize,
	pub memory_stack: Vec<StackValue>,
	pub calling_stack: Vec<FuncStackFrame>,
	pub calling_params: HashMap<String, Vec<llvm::Value>>,
	pub global_vars: HashMap<String, usize>,
}

impl MiddleSimulator {
	pub fn new(input: String) -> Self {
		MiddleSimulator {
			input,
			input_position: 0,
			output: vec![],
			step_count: 0,
			instr_list: vec![],
			label_map: HashMap::new(),
			return_scratch: None,
			pc: 0,
			memory_stack: vec![],
			calling_stack: vec![],
			calling_params: HashMap::new(),
			global_vars: HashMap::new(),
		}
	}

	fn init(&mut self, program: &LlvmProgram) {
		self.instr_list = vec![];
		for item in &program.funcs {
			self.calling_params.insert(item.name.clone(), item.params.clone());
			let mut labels = HashMap::new();
			for block in &item.cfg.blocks {
				let label = block.borrow().label();
				let current_addr = self.instr_list.len();
				labels.insert(label.name.clone(), current_addr);

				for item in &block.borrow().phi_instrs {
					self.instr_list.push(item.clone_box());
				}
				for item in &block.borrow().instrs {
					self.instr_list.push(item.clone_box());
				}
				for item in &block.borrow().jump_instr {
					self.instr_list.push(item.clone_box());
				}
			}

			self.label_map.insert(item.name.clone(), labels);
		}

		for item in &program.global_vars {
			self.global_vars.insert(item.ident.clone(), self.memory_stack.len());

			for value_item in &item.data {
				if item.is_float {
					match value_item {
						utils::ValueItem::Word(v) => self.memory_stack.push({
							//按照bit强转 float, 不得以而为之
							let f: f32 = f32::from_bits(*v);
							StackValue::Float(f)
						}),
						utils::ValueItem::Zero(n) => {
							let len = n / 4;
							let mut zeros = vec![StackValue::Float(0f32); len];
							self.memory_stack.append(&mut zeros);
						}
					}
				} else {
					match value_item {
						utils::ValueItem::Word(v) => self.memory_stack.push(unsafe {
							//按照bit强转 i32, 不得以而为之
							let i: i32 = transmute(*v);
							StackValue::Int(i)
						}),
						utils::ValueItem::Zero(n) => {
							let len = n / 4;
							let mut zeros = vec![StackValue::Int(0); len];
							self.memory_stack.append(&mut zeros);
						}
					}
				}
			}
		}

		// println!("{:?}", &self.global_vars);
		// println!("{:?}", &self.memory_stack);
	}
	pub fn run_program(&mut self, program: &LlvmProgram) {
		self.init(program);

		self.pc =
			*self.label_map.get("main").unwrap().get("entry").to_owned().unwrap();

		self.calling_stack.push(FuncStackFrame {
			ra: None,
			name: "main".into(),
			fp: self.memory_stack.len(),
			current_label: "entry".into(),
			last_label: "".into(),
			temp: HashMap::new(),
			return_to: "".to_string(),
		});

		loop {
			let instr = self.instr_list.get(self.pc).unwrap();
			// println!("{:?} {:#}",self.pc, &instr);

			let mut next = self.pc + 1;
			let frame = self.calling_stack.last_mut().unwrap();
			// println!("last {:?} now {:?}", frame.last_label, frame.current_label);
			// println!("{:?}", frame.temp);
			let labels = self.label_map.get(&frame.name).unwrap();
			let mut return_info: Option<(String, StackValue)> = None;

			match instr.get_variant() {
				llvm::LlvmInstrVariant::ArithInstr(instr) => {
					do_arith_instr(instr, frame, &self.global_vars)
				}
				llvm::LlvmInstrVariant::CompInstr(instr) => {
					do_comp_instr(instr, frame, &self.global_vars)
				}
				llvm::LlvmInstrVariant::ConvertInstr(instr) => {
					let lhs = get_stack(&instr.lhs, frame, &self.global_vars);
					let value = match instr.op {
						llvm::ConvertOp::Int2Float => {
							StackValue::Float(lhs.as_i32() as f32)
						}
						llvm::ConvertOp::Float2Int => StackValue::Int(lhs.as_f32() as i32),
					};
					frame.temp.insert(instr.target.name.clone(), value);
				}
				llvm::LlvmInstrVariant::JumpInstr(instr) => {
					frame.last_label = frame.current_label.clone();
					frame.current_label = instr.target.name.clone();

					next = *labels.get(&instr.target.name).unwrap();
				}
				llvm::LlvmInstrVariant::JumpCondInstr(instr) => {
					let value = get_stack(&instr.cond, frame, &self.global_vars);

					let jump = match instr.var_type {
						llvm::VarType::I32 => value.as_i32() != 0,
						llvm::VarType::F32 => value.as_f32() != 0f32,
						_ => todo!(),
					};

					if jump {
						frame.last_label = frame.current_label.clone();
						frame.current_label = instr.target_true.name.clone();
						next = *labels.get(&instr.target_true.name).unwrap();
					} else {
						frame.last_label = frame.current_label.clone();
						frame.current_label = instr.target_false.name.clone();
						next = *labels.get(&instr.target_false.name).unwrap();
					}
				}
				llvm::LlvmInstrVariant::PhiInstr(instr) => {
					for (value, label) in &instr.source {
						if label.name == frame.last_label {
							frame.temp.insert(
								instr.target.name.clone(),
								get_stack(value, frame, &self.global_vars),
							);
							break;
						}
					}
				}
				llvm::LlvmInstrVariant::RetInstr(instr) => {
					if let Some(value) = &instr.value {
						self.return_scratch =
							Some(get_stack(value, frame, &self.global_vars));
					} else {
						self.return_scratch = None;
					}

					// println!("{:?}", self.memory_stack);

					self.memory_stack.resize(frame.fp, StackValue::Int(0));

					if let Some(ra) = &frame.ra {
						next = *ra;
					} else {
						return;
					}

					// self.calling_stack.get(self.calling_stack.len() -2 ).unwrap().temp.insert(frame.return_to.clone(), self.return_scratch.unwrap_or_default());
					return_info = Some((
						frame.return_to.clone(),
						self.return_scratch.unwrap_or_default(),
					));
				}
				llvm::LlvmInstrVariant::AllocInstr(instr) => {
					let len = get_stack(&instr.length, frame, &self.global_vars);
					// TODO 随机化?
					let mut to_append = match instr.var_type {
						llvm::VarType::I32Ptr | llvm::VarType::I32 => {
							vec![StackValue::Int(0); len.as_i32() as usize / 4]
						}
						llvm::VarType::F32Ptr | llvm::VarType::F32 => {
							vec![StackValue::Float(0f32); len.as_i32() as usize / 4]
						}
						_ => unreachable!(),
					};
					frame.temp.insert(
						instr.target.name.clone(),
						StackValue::Ptr(self.memory_stack.len()),
					);
					self.memory_stack.append(&mut to_append);
				}
				llvm::LlvmInstrVariant::StoreInstr(instr) => {
					let addr = get_stack(&instr.addr, frame, &self.global_vars);
					let value = get_stack(&instr.value, frame, &self.global_vars);
					*self.memory_stack.get_mut(addr.as_usize()).unwrap() = value;
				}
				llvm::LlvmInstrVariant::LoadInstr(instr) => {
					let addr = get_stack(&instr.addr, frame, &self.global_vars);

					let value = match instr.var_type {
						VarType::I32 | VarType::F32 => match addr {
							StackValue::Ptr(v) => *self.memory_stack.get(v).unwrap(),
							_ => unreachable!(),
						},
						VarType::I32Ptr | VarType::F32Ptr => addr,
						_ => {
							unreachable!()
						}
					};

					frame.temp.insert(instr.target.name.clone(), value);
				}
				llvm::LlvmInstrVariant::GEPInstr(instr) => {
					let addr = get_stack(&instr.addr, frame, &self.global_vars);
					let offset = get_stack(&instr.offset, frame, &self.global_vars);
					frame.temp.insert(
						instr.target.name.clone(),
						StackValue::Ptr(
							(addr.as_usize() as i32 + offset.as_i32() / 4) as usize,
						),
					);
				}
				llvm::LlvmInstrVariant::CallInstr(instr) => {
					let mut value_list = vec![];
					let mut argument_name_list = vec![];
					for (_type, value) in &instr.params {
						value_list.push(get_stack(value, frame, &self.global_vars));
					}

					let (lib, lib_value) = inout(
						&instr.func.name,
						&value_list,
						&mut self.output,
						&self.input,
						&mut self.input_position,
					);

					if lib {
						if let Some(lib_value) = lib_value {
							frame.temp.insert(instr.target.name.clone(), lib_value);
						}
					} else {
						for item in self.calling_params.get(&instr.func.name).unwrap() {
							match item {
								Value::Temp(t) => argument_name_list.push(t.name.clone()),
								_ => unreachable!(),
							}
						}

						assert_eq!(argument_name_list.len(), value_list.len());

						let arguments: HashMap<String, StackValue> =
							argument_name_list.into_iter().zip(value_list).collect();

						// dbg!(&arguments);

						self.calling_stack.push(FuncStackFrame {
							ra: Some(self.pc + 1),
							name: instr.func.name.clone(),
							fp: self.memory_stack.len(),
							current_label: "entry".into(),
							last_label: "".into(),
							temp: arguments,
							return_to: instr.target.name.clone(),
						});

						next = *self
							.label_map
							.get(&instr.func.name)
							.unwrap()
							.get("entry")
							.unwrap();
					}
				}
			}
			self.pc = next;

			if let Some((name, value)) = return_info {
				self.calling_stack.pop();
				self.calling_stack.last_mut().unwrap().temp.insert(name, value);
			}
		}
	}
}

fn get_stack(
	value: &llvm::Value,
	frame: &FuncStackFrame,
	global_var: &HashMap<String, usize>,
) -> StackValue {
	match value {
		llvm::Value::Int(v) => StackValue::Int(*v),
		llvm::Value::Float(v) => StackValue::Float(*v),
		llvm::Value::Temp(t) => {
			if t.is_global {
				if let Some(addr) = global_var.get(&t.name) {
					match t.var_type {
						llvm::VarType::I32 | llvm::VarType::I32Ptr => {
							StackValue::Ptr(*addr)
						}
						llvm::VarType::F32 | llvm::VarType::F32Ptr => {
							StackValue::Ptr(*addr)
						}
						_ => unreachable!(),
					}
				} else {
					unreachable!();
				}
			} else if let Some(v) = frame.temp.get(&t.name) {
				*v
			} else {
				dbg!(&t, &frame);
				unreachable!();
			}
		}
	}
}

fn do_arith_instr(
	instr: &ArithInstr,
	frame: &mut FuncStackFrame,
	global_var: &HashMap<String, usize>,
) {
	let lhs = get_stack(&instr.lhs, frame, global_var);
	let rhs = get_stack(&instr.rhs, frame, global_var);

	let value: StackValue = match instr.op {
		llvm::ArithOp::Add => lhs.as_i32().wrapping_add(rhs.as_i32()).into(),
		llvm::ArithOp::Sub => lhs.as_i32().wrapping_sub(rhs.as_i32()).into(),
		llvm::ArithOp::Div => lhs.as_i32().wrapping_div(rhs.as_i32()).into(),
		llvm::ArithOp::Mul => lhs.as_i32().wrapping_mul(rhs.as_i32()).into(),
		llvm::ArithOp::Rem => lhs.as_i32().wrapping_rem(rhs.as_i32()).into(),
		llvm::ArithOp::Fadd => (lhs.as_f32() + rhs.as_f32()).into(),
		llvm::ArithOp::Fsub => (lhs.as_f32() - rhs.as_f32()).into(),
		llvm::ArithOp::Fdiv => (lhs.as_f32() / rhs.as_f32()).into(),
		llvm::ArithOp::Fmul => (lhs.as_f32() * rhs.as_f32()).into(),
		llvm::ArithOp::Shl => todo!(),
		llvm::ArithOp::Lshr => todo!(),
		llvm::ArithOp::Ashr => todo!(),
		llvm::ArithOp::And => todo!(),
		llvm::ArithOp::Or => todo!(),
		llvm::ArithOp::Xor => todo!(),
		llvm::ArithOp::AddD => todo!(),
	};

	let target = &instr.target;
	frame.temp.insert(target.name.clone(), value);
}

fn do_comp_instr(
	instr: &CompInstr,
	frame: &mut FuncStackFrame,
	global_var: &HashMap<String, usize>,
) {
	let lhs = get_stack(&instr.lhs, frame, global_var);
	let rhs = get_stack(&instr.rhs, frame, global_var);

	let value: bool = match instr.var_type {
		llvm::VarType::I32 => match instr.op {
			llvm::CompOp::EQ => lhs.as_i32() == rhs.as_i32(),
			llvm::CompOp::NE => lhs.as_i32() != rhs.as_i32(),
			llvm::CompOp::SGT => lhs.as_i32() > rhs.as_i32(),
			llvm::CompOp::SGE => lhs.as_i32() >= rhs.as_i32(),
			llvm::CompOp::SLT => lhs.as_i32() < rhs.as_i32(),
			llvm::CompOp::SLE => lhs.as_i32() <= rhs.as_i32(),
			llvm::CompOp::OEQ => todo!(),
			llvm::CompOp::ONE => todo!(),
			llvm::CompOp::OGT => todo!(),
			llvm::CompOp::OGE => todo!(),
			llvm::CompOp::OLT => todo!(),
			llvm::CompOp::OLE => todo!(),
		},
		llvm::VarType::F32 => match instr.op {
			llvm::CompOp::EQ => lhs.as_f32() == rhs.as_f32(),
			llvm::CompOp::NE => lhs.as_f32() != rhs.as_f32(),
			llvm::CompOp::SGT => lhs.as_f32() > rhs.as_f32(),
			llvm::CompOp::SGE => lhs.as_f32() >= rhs.as_f32(),
			llvm::CompOp::SLT => lhs.as_f32() < rhs.as_f32(),
			llvm::CompOp::SLE => lhs.as_f32() <= rhs.as_f32(),
			llvm::CompOp::OEQ => todo!(),
			llvm::CompOp::ONE => todo!(),
			llvm::CompOp::OGT => todo!(),
			llvm::CompOp::OGE => todo!(),
			llvm::CompOp::OLT => todo!(),
			llvm::CompOp::OLE => todo!(),
		},
		_ => unreachable!(),
	};

	let target = &instr.target;
	frame.temp.insert(target.name.clone(), (value as i32).into());
}
