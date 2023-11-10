use crate::{
	func::LlvmFunc,
	label::Label,
	llvminstr::*,
	llvmop::*,
	llvmvar::VarType,
	temp::{Temp, TempManager},
	utils::ptr2type,
};

pub struct LlvmFuncEmitter {
	label: Label,
	params: Vec<Temp>,
	ret_type: VarType,
	temp_mgr: TempManager,
	func_body: Vec<Box<dyn LlvmInstr>>,
}

impl LlvmFuncEmitter {
	pub fn new(name: String, ret_type: VarType, params: Vec<Temp>) -> Self {
		LlvmFuncEmitter {
			label: Label::new(format!("Function<{}>", name)),
			ret_type,
			params,
			temp_mgr: TempManager::new(),
			func_body: Vec::new(),
		}
	}

	pub fn visit_label(&mut self, label: Label) {
		self.func_body.push(Box::new(LabelInstr { label }))
	}

	pub fn visit_arith_instr(
		&mut self,
		lhs: Value,
		op: ArithOp,
		rhs: Value,
	) -> Temp {
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
		lhs: Value,
		op: CompOp,
		rhs: Value,
	) -> Temp {
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

	pub fn visit_ret(&mut self, value: Value) {
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

	pub fn visit_store_instr(&mut self, value: Value, addr: Value) {
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
		let var_type = ptr2type(addr.get_type());
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
		func_label: Label,
		params: Vec<Value>,
	) -> Temp {
		let target = self.temp_mgr.new_temp(var_type);
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

	pub fn visit_end(mut self) -> LlvmFunc {
		fn get_default_value(ret_type: VarType) -> Value {
			match ret_type {
				VarType::F32 => Value::Float(0.0),
				VarType::I32 => Value::Int(0),
				VarType::Void => Value::Void,
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
