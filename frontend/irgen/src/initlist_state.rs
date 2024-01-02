use llvm::{GEPInstr, StoreInstr, Temp, TempManager, Value, VarType};
use rrvm::LlvmCFG;

pub struct InitlistState {
	pub var_type: VarType,
	pub init_items: Vec<Value>,
	pub decl_dims: Vec<usize>,
	pub target: Temp,
	pub depth: usize,
	pub cnt: usize,
}

impl InitlistState {
	pub fn new(var_type: VarType, decl_dims: Vec<usize>, target: Temp) -> Self {
		Self {
			var_type,
			decl_dims,
			init_items: Vec::new(),
			target,
			depth: 0,
			cnt: 0,
		}
	}
	pub fn cur_size(&self) -> usize {
		self.decl_dims.iter().skip(self.depth).product()
	}
	pub fn push(&mut self) {
		self.depth += 1;
	}
	pub fn pop(&mut self) {
		self.depth -= 1;
	}
	pub fn store(
		&mut self,
		value: Value,
		cfg: &mut LlvmCFG,
		mgr: &mut TempManager,
	) {
		self.cnt += 1;
		let instr = Box::new(StoreInstr {
			addr: self.target.clone().into(),
			value,
		});
		cfg.get_exit().borrow_mut().push(instr);
		let new_temp = mgr.new_temp(self.var_type, false);
		let instr = Box::new(GEPInstr {
			var_type: self.var_type,
			target: new_temp.clone(),
			addr: self.target.clone().into(),
			offset: self.var_type.deref_type().get_size().into(),
		});
		cfg.get_exit().borrow_mut().push(instr);
		self.target = new_temp;
	}
	pub fn assign(
		&mut self,
		size: usize,
		cfg: &mut LlvmCFG,
		mgr: &mut TempManager,
	) {
		while self.cnt % size != 0 {
			self.store(self.var_type.default_value(), cfg, mgr)
		}
	}
}
