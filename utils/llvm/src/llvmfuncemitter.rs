use utils::{Label, LabelManager};

use crate::{
	basicblock::BasicBlock,
	cfg::CFG,
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
	break_label: Vec<usize>,
	continue_label: Vec<usize>,
	cfg: CFG,
	cur_basicblock: usize,
	// func_body: Vec<Box<dyn LlvmInstr>>,
}

impl LlvmFuncEmitter {
	pub fn new(
		name: String,
		ret_type: VarType,
		params: Vec<Temp>,
		entry: BasicBlock,
		exit: BasicBlock,
	) -> Self {
		LlvmFuncEmitter {
			label: Label::new(format!("Function<{}>", name)),
			ret_type,
			params,
			temp_mgr: TempManager::new(),
			label_mgr: LabelManager::new(),
			break_label: Vec::new(),
			continue_label: Vec::new(),
			cur_basicblock: entry.id,
			cfg: CFG::new(entry, exit),
		}
	}

	pub fn get_cur_basicblock(&mut self) -> &mut BasicBlock {
		self.cfg.basic_blocks.get_mut(&self.cur_basicblock).unwrap()
	}

	// 这里可能需要创建temp，将新的temp total更新到 temp manager
	// 传usize是因为succ已经在cfg内了
	pub fn add_succ_to_cur_basicblock(&mut self, succ_id: usize) {
		let symbol2temp = self.get_cur_basicblock().symbol2temp.clone();
		let cur_label = self.get_cur_basicblock().label.clone();
		let mut cur_temp_total = self.temp_mgr.cur_total();
		let cur_id = self.cur_basicblock;
		{
			let succ = self.get_basicblock(succ_id);
			succ.pred.push(cur_id);
			for (k, v) in symbol2temp.iter() {
				if succ.symbol2temp.contains_key(k) {
					let succ_value = succ.symbol2temp.get(k).unwrap();
					succ
						.phi_instrs
						.get_mut(succ_value)
						.unwrap()
						.push((cur_label.clone(), v.clone()));
				} else {
					cur_temp_total += 1;
					let new_temp = Temp::new(cur_temp_total, v.var_type);
					succ.symbol2temp.insert(*k, new_temp.clone());
					succ
						.phi_instrs
						.insert(new_temp.clone(), vec![(cur_label.clone(), v.clone())]);
				}
			}
		}
		self.temp_mgr.set_total(cur_temp_total);
		self.get_cur_basicblock().succ.push(succ_id);
	}

	pub fn get_basicblock(&mut self, id: usize) -> &mut BasicBlock {
		self.cfg.basic_blocks.get_mut(&id).unwrap()
	}

	// 一个label对应一个BasicBlock，所以这里创建一个新的BasicBlock
	// 这里直接将它放入cfg中，返回id
	pub fn fresh_label(&mut self) -> (usize, Label) {
		let label = self.label_mgr.new_label();
		let id = self.cfg.basic_blocks.len();
		self
			.cfg
			.basic_blocks
			.insert(id, BasicBlock::new(id, label.clone(), Vec::new()));
		(id, label)
	}

	pub fn fresh_temp(&mut self, var_type: VarType) -> Temp {
		self.temp_mgr.new_temp(var_type)
	}

	pub fn cur_temp_total(&self) -> u32 {
		self.temp_mgr.cur_total()
	}

	pub fn set_temp_total(&mut self, total: u32) {
		self.temp_mgr.set_total(total);
	}

	// 这里传basicblock的id
	pub fn openloop(&mut self, break_bb_id: usize, continue_bb_id: usize) {
		self.break_label.push(break_bb_id);
		self.continue_label.push(continue_bb_id);
	}

	pub fn closeloop(&mut self) {
		self.break_label.pop();
		self.continue_label.pop();
	}

	pub fn get_break_label(&self) -> usize {
		*self.break_label.last().unwrap()
	}

	pub fn get_continue_label(&self) -> usize {
		*self.continue_label.last().unwrap()
	}

	pub fn visit_label(&mut self, label: usize) {
		self.cur_basicblock = label;
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
					self.get_cur_basicblock().add(Box::new(convert));
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
					self.get_cur_basicblock().add(Box::new(convert));
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
		self.get_cur_basicblock().add(Box::new(instr));
		target
	}

	pub fn visit_assign_instr(&mut self, target: Temp, value: Value) {
		let instr = ArithInstr {
			target: target.clone(),
			var_type: target.var_type,
			lhs: value,
			op: if target.var_type == VarType::I32 {
				ArithOp::Add
			} else {
				ArithOp::Fadd
			},
			rhs: if target.var_type == VarType::I32 {
				Value::Int(0)
			} else {
				Value::Float(0.0)
			},
		};
		self.get_cur_basicblock().add(Box::new(instr));
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
					self.get_cur_basicblock().add(Box::new(convert));
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
					self.get_cur_basicblock().add(Box::new(convert));
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
		self.get_cur_basicblock().add(Box::new(instr));
		target
	}

	pub fn visit_jump_instr(&mut self, target: Label, id: usize) {
		// 如果当前基本块最后一条语句已经是跳转了，则不添加跳转语句
		// TODO: is_seq() == false 就一定是跳转语句嘛？
		if self.get_cur_basicblock().instrs.last().map_or(false, |v| !v.is_seq()) {
			return;
		}
		// 否则添加
		let instr = JumpInstr { target };
		self.get_cur_basicblock().add(Box::new(instr));

		self.add_succ_to_cur_basicblock(id);
	}

	pub fn visit_jump_cond_instr(
		&mut self,
		cond: Value,
		target_true: Label,
		target_false: Label,
		target_true_id: usize,
		target_false_id: usize,
	) {
		if self.get_cur_basicblock().instrs.last().map_or(false, |v| !v.is_seq()) {
			return;
		}

		let instr = JumpCondInstr {
			var_type: VarType::I32,
			cond,
			target_true,
			target_false,
		};
		self.get_cur_basicblock().add(Box::new(instr));

		self.add_succ_to_cur_basicblock(target_true_id);
		self.add_succ_to_cur_basicblock(target_false_id);
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
		self.get_cur_basicblock().add(Box::new(instr));
		target
	}

	pub fn visit_ret(&mut self, value: Option<Value>) {
		if self.get_cur_basicblock().instrs.last().map_or(false, |v| !v.is_seq()) {
			return;
		}

		let instr = RetInstr { value };
		self.get_cur_basicblock().add(Box::new(instr));
		// exit basicblock 的 id 固定为 1
		self.add_succ_to_cur_basicblock(1);
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
		self.get_cur_basicblock().add(Box::new(instr));
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
					self.get_cur_basicblock().add(Box::new(convert));
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
					self.get_cur_basicblock().add(Box::new(convert));
					Value::Temp(new_temp)
				}
				_ => unreachable!(),
			};
		}
		let instr = StoreInstr { value, addr };
		self.get_cur_basicblock().add(Box::new(instr));
	}

	pub fn visit_load_instr(&mut self, addr: Value) -> Temp {
		let var_type = ptr2type(addr.get_type());
		let target = self.temp_mgr.new_temp(var_type);
		let instr = LoadInstr {
			target: target.clone(),
			var_type,
			addr,
		};
		self.get_cur_basicblock().add(Box::new(instr));
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
		self.get_cur_basicblock().add(Box::new(instr));
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
		self.get_cur_basicblock().add(Box::new(instr));
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
		self.get_cur_basicblock().add(Box::new(instr));
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
		if self.get_cur_basicblock().instrs.last().map_or(true, |v| !v.is_ret()) {
			self.visit_ret(get_default_value(self.ret_type));
		}
		// 给每一个 basicblock 添上 phi 语句, 去掉只有一项的 phi 语句
		for basicblock in self.cfg.basic_blocks.values_mut() {
			for (k, v) in basicblock.phi_instrs.iter() {
				if v.len() == 1 {
					for instr in &mut basicblock.instrs {
						instr.swap_temp(k.clone(), v[0].1.clone());
					}
					continue;
				}
				let phi = PhiInstr {
					target: k.clone(),
					var_type: k.var_type,
					source: v
						.iter()
						.map(|(l, t)| (Value::Temp(t.clone()), l.clone()))
						.collect(),
				};
				basicblock.instrs.insert(0, Box::new(phi));
			}
		}
		LlvmFunc {
			label: self.label,
			params: self.params,
			ret_type: self.ret_type,
			// body: self.func_body,
			body: Vec::new(),
			cfg: self.cfg,
		}
	}
}
