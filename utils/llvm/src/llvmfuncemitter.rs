use crate::{
	func::LlvmFunc,
	label::Label,
	llvminstr::*,
	llvmop::*,
	llvmvar::VarType,
	temp::{Temp, TempManager},
};

pub struct LlvmFuncEmitter {
	label: Label,
	ret_type: VarType,
	temp_mgr: TempManager,
	func_body: Vec<Box<dyn LlvmInstr>>,
}

impl LlvmFuncEmitter {
	pub fn new(name: String, ret_type: VarType) -> Self {
		LlvmFuncEmitter {
			label: Label::new(name),
			ret_type,
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
			ret_type: self.ret_type,
			body: self.func_body,
		}
	}
}
