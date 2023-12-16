use utils::{Label, LabelManager};

use crate::{
	func::LlvmFunc,
	llvminstr::*,
	llvmop::*,
	llvmvar::VarType,
	temp::{Temp, TempManager},
	utils_llvm::ptr2type,
};

pub struct LlvmFuncEmitter {
	pub ret_type: VarType,
	label: Label,
	params: Vec<Temp>,
	temp_mgr: TempManager,
	label_mgr: LabelManager,
	break_label: Vec<Label>,
	continue_label: Vec<Label>,
	func_body: Vec<Box<dyn LlvmInstr>>,
}

impl LlvmFuncEmitter {
	pub fn new(name: String, ret_type: VarType, params: Vec<Temp>) -> Self {
		LlvmFuncEmitter {
			label: Label::new(format!("Function<{}>", name)),
			ret_type,
			params,
			temp_mgr: TempManager::new(),
			label_mgr: LabelManager::new(),
			break_label: Vec::new(),
			continue_label: Vec::new(),
			func_body: Vec::new(),
		}
	}

	pub fn fresh_label(&mut self) -> Label {
		self.label_mgr.new_label()
	}

	pub fn openloop(&mut self, break_label: Label, continue_label: Label) {
		self.break_label.push(break_label);
		self.continue_label.push(continue_label);
	}

	pub fn closeloop(&mut self) {
		self.break_label.pop();
		self.continue_label.pop();
	}

	pub fn get_break_label(&self) -> Label {
		self.break_label.last().unwrap().clone()
	}

	pub fn get_continue_label(&self) -> Label {
		self.continue_label.last().unwrap().clone()
	}

	pub fn visit_label(&mut self, label: Label) {
		self.func_body.push(Box::new(LabelInstr { label }))
	}

	pub fn visit_arith_instr(
		&mut self,
		mut lhs: Value,
		op: ArithOp,
		mut rhs: Value,
	) -> Temp {
		if lhs.get_type() == VarType::F32 && rhs.get_type() == VarType::I32 {
			rhs = match rhs {
				Value::Int(v) => Value::Float(v as f32),
				Value::Temp(t) => {
					let new_temp = self.temp_mgr.new_temp(VarType::F32);
					let convert = ConvertInstr {
						target: new_temp.clone(),
						op: ConvertOp::Int2Float,
						from_type: VarType::I32,
						lhs: Value::Temp(t),
						to_type: VarType::F32,
					};
					self.func_body.push(Box::new(convert));
					Value::Temp(new_temp)
				}
				_ => unreachable!(),
			}
		}
		if lhs.get_type() == VarType::I32 && rhs.get_type() == VarType::F32 {
			lhs = match lhs {
				Value::Int(v) => Value::Float(v as f32),
				Value::Temp(t) => {
					let new_temp = self.temp_mgr.new_temp(VarType::F32);
					let convert = ConvertInstr {
						target: new_temp.clone(),
						op: ConvertOp::Int2Float,
						from_type: VarType::I32,
						lhs: Value::Temp(t),
						to_type: VarType::F32,
					};
					self.func_body.push(Box::new(convert));
					Value::Temp(new_temp)
				}
				_ => unreachable!(),
			}
		}
		let target = self.temp_mgr.new_temp(op.oprand_type());
		let instr = ArithInstr {
			target: target.clone(),
			var_type: op.oprand_type(),
			lhs,
			op,
			rhs,
		};
		self.func_body.push(Box::new(instr));
		target
	}

	pub fn visit_comp_instr(
		&mut self,
		mut lhs: Value,
		op: CompOp,
		mut rhs: Value,
	) -> Temp {
		if lhs.get_type() == VarType::F32 && rhs.get_type() == VarType::I32 {
			rhs = match rhs {
				Value::Int(v) => Value::Float(v as f32),
				Value::Temp(t) => {
					let new_temp = self.temp_mgr.new_temp(VarType::F32);
					let convert = ConvertInstr {
						target: new_temp.clone(),
						op: ConvertOp::Int2Float,
						from_type: VarType::I32,
						lhs: Value::Temp(t),
						to_type: VarType::F32,
					};
					self.func_body.push(Box::new(convert));
					Value::Temp(new_temp)
				}
				_ => unreachable!(),
			}
		}
		if lhs.get_type() == VarType::I32 && rhs.get_type() == VarType::F32 {
			lhs = match lhs {
				Value::Int(v) => Value::Float(v as f32),
				Value::Temp(t) => {
					let new_temp = self.temp_mgr.new_temp(VarType::F32);
					let convert = ConvertInstr {
						target: new_temp.clone(),
						op: ConvertOp::Int2Float,
						from_type: VarType::I32,
						lhs: Value::Temp(t),
						to_type: VarType::F32,
					};
					self.func_body.push(Box::new(convert));
					Value::Temp(new_temp)
				}
				_ => unreachable!(),
			}
		}
		fn get_kind(op: &CompOp) -> CompKind {
			match op.oprand_type() {
				VarType::I32 => CompKind::Icmp,
				VarType::F32 => CompKind::Fcmp,
				_ => unreachable!(),
			}
		}
		let target = self.temp_mgr.new_temp(op.oprand_type());
		let instr = CompInstr {
			kind: get_kind(&op),
			target: target.clone(),
			var_type: op.oprand_type(),
			lhs,
			op,
			rhs,
		};
		self.func_body.push(Box::new(instr));
		target
	}

	pub fn visit_jump_instr(&mut self, target: Label) {
		let instr = JumpInstr { target };
		self.func_body.push(Box::new(instr));
	}

	pub fn visit_jump_cond_instr(
		&mut self,
		cond: Value,
		target_true: Label,
		target_false: Label,
	) {
		let instr = JumpCondInstr {
			var_type: VarType::I32,
			cond,
			target_true,
			target_false,
		};
		self.func_body.push(Box::new(instr));
	}

	pub fn visit_phi_instr(
		&mut self,
		var_type: VarType,
		source: Vec<(Value, Label)>,
	) -> Temp {
		let target = self.temp_mgr.new_temp(var_type);
		let instr = PhiInstr {
			target: target.clone(),
			var_type,
			source,
		};
		self.func_body.push(Box::new(instr));
		target
	}

	pub fn visit_ret(&mut self, value: Option<Value>) {
		let instr = RetInstr { value };
		self.func_body.push(Box::new(instr));
	}

	pub fn visit_alloc_instr(
		&mut self,
		var_type: VarType,
		length: Value,
	) -> Temp {
		let target = self.temp_mgr.new_temp(var_type);
		let instr = AllocInstr {
			target: target.clone(),
			var_type,
			length,
		};
		self.func_body.push(Box::new(instr));
		target
	}

	pub fn visit_store_instr(&mut self, mut value: Value, addr: Value) {
		if value.get_type() == VarType::F32 && addr.get_type() == VarType::I32Ptr {
			value = match value {
				Value::Float(v) => Value::Int(v as i32),
				Value::Temp(t) => {
					let new_temp = self.temp_mgr.new_temp(VarType::I32);
					let convert = ConvertInstr {
						target: new_temp.clone(),
						op: ConvertOp::Int2Float,
						from_type: VarType::F32,
						lhs: Value::Temp(t),
						to_type: VarType::I32,
					};
					self.func_body.push(Box::new(convert));
					Value::Temp(new_temp)
				}
				_ => unreachable!(),
			};
		}
		if value.get_type() == VarType::I32 && addr.get_type() == VarType::F32Ptr {
			value = match value {
				Value::Int(v) => Value::Float(v as f32),
				Value::Temp(t) => {
					let new_temp = self.temp_mgr.new_temp(VarType::F32);
					let convert = ConvertInstr {
						target: new_temp.clone(),
						op: ConvertOp::Int2Float,
						from_type: VarType::I32,
						lhs: Value::Temp(t),
						to_type: VarType::F32,
					};
					self.func_body.push(Box::new(convert));
					Value::Temp(new_temp)
				}
				_ => unreachable!(),
			};
		}
		let instr = StoreInstr { value, addr };
		self.func_body.push(Box::new(instr));
	}

	pub fn visit_load_instr(&mut self, addr: Value) -> Temp {
		let var_type = ptr2type(addr.get_type());
		let target = self.temp_mgr.new_temp(var_type);
		let instr = LoadInstr {
			target: target.clone(),
			var_type,
			addr,
		};
		self.func_body.push(Box::new(instr));
		target
	}

	pub fn visit_gep_instr(&mut self, addr: Value, offset: Value) -> Temp {
		let var_type = addr.get_type();
		let target = self.temp_mgr.new_temp(var_type);
		let instr = GEPInstr {
			target: target.clone(),
			var_type,
			addr,
			offset,
		};
		self.func_body.push(Box::new(instr));
		target
	}

	pub fn visit_call_instr(
		&mut self,
		var_type: VarType,
		func_name: String,
		params: Vec<Value>,
	) -> Temp {
		let target = self.temp_mgr.new_temp(var_type);
		let func_label = Label::new(format!("Function<{}>", func_name));
		let instr = CallInstr {
			target: target.clone(),
			var_type,
			func: func_label,
			params: params.into_iter().map(|v| (v.get_type(), v)).collect(),
		};
		self.func_body.push(Box::new(instr));
		target
	}

	pub fn visit_formal_param(&mut self, param_type: VarType) -> Temp {
		let tmp = self.temp_mgr.new_temp(param_type);
		self.params.push(tmp.clone());
		tmp
	}

	pub fn visit_convert_instr(
		&mut self,
		op: ConvertOp,
		from_type: VarType,
		value: Value,
		to_type: VarType,
	) -> Temp {
		let target = self.temp_mgr.new_temp(to_type);
		let instr = ConvertInstr {
			target: target.clone(),
			op,
			from_type,
			lhs: value,
			to_type,
		};
		self.func_body.push(Box::new(instr));
		target
	}

	pub fn visit_end(mut self) -> LlvmFunc {
		fn get_default_value(ret_type: VarType) -> Option<Value> {
			match ret_type {
				VarType::F32 => Some(Value::Float(0.0)),
				VarType::I32 => Some(Value::Int(0)),
				VarType::Void => None,
				_ => unreachable!(),
			}
		}
		if self.func_body.last().map_or(true, |v| !v.is_ret()) {
			self.visit_ret(get_default_value(self.ret_type));
		}
		LlvmFunc {
			label: self.label,
			params: self.params,
			ret_type: self.ret_type,
			body: self.func_body,
		}
	}
}
