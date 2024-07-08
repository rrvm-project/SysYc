use crate::scope::Scope;
use rrvm_symbol::{manager::SymbolManager, FuncSymbol, VarSymbol};
use utils::{
	errors::Result,
	SysycError::{FatalError, SyntaxError},
};
use value::{FuncRetType, FuncType, Value, VarType};

#[derive(Default)]
pub struct ScopeStack {
	func_scope: Scope<FuncType>,
	scopes: Vec<Scope<VarType>>,
}

const UNDERFLOW_ERR_MSG: &str = "stack of scopes underFlow";

impl ScopeStack {
	fn top(&mut self) -> Result<&mut Scope<VarType>> {
		self.scopes.last_mut().ok_or(FatalError(UNDERFLOW_ERR_MSG.to_owned()))
	}
	pub fn push(&mut self) {
		self.scopes.push(Scope::new())
	}
	pub fn pop(&mut self) -> Result<()> {
		if self.scopes.pop().is_none() {
			Err(FatalError(UNDERFLOW_ERR_MSG.to_owned()))
		} else {
			Ok(())
		}
	}
	pub fn set_val(&mut self, ident: &str, symbol: VarSymbol) -> Result<()> {
		self.top()?.new_symbol(ident, symbol)
	}
	pub fn set_func(&mut self, ident: &str, symbol: FuncSymbol) -> Result<()> {
		self.func_scope.new_symbol(ident, symbol)
	}
	pub fn get_val(&self, ident: &str) -> Result<&VarSymbol> {
		self
			.scopes
			.iter()
			.rev()
			.find_map(|v| v.get_symbol(ident))
			.ok_or(SyntaxError(format!("{} is not found", ident)))
	}
	pub fn get_func(&self, ident: &str) -> Result<&FuncSymbol> {
		self
			.func_scope
			.get_symbol(ident)
			.ok_or(SyntaxError(format!("{} is not found", ident)))
	}
	pub fn set_constant(&mut self, id: i32, value: Value) -> Result<()> {
		self.top()?.set_constant(id, value);
		Ok(())
	}
	pub fn get_constant(&mut self, id: i32) -> Option<&Value> {
		self.scopes.iter().rev().find_map(|v| v.get_constant(id))
	}
	pub fn is_global(&self) -> bool {
		self.scopes.len() == 1
	}
	pub fn extern_init(&mut self, mgr: &mut SymbolManager) -> Result<()> {
		let getint_t = (FuncRetType::Int, Vec::new());
		self.set_func("getint", mgr.new_func_symbol("getint", getint_t))?;
		let getch_t = (FuncRetType::Int, Vec::new());
		self.set_func("getch", mgr.new_func_symbol("getch", getch_t))?;
		let getfloat_t = (FuncRetType::Float, Vec::new());
		self.set_func("getfloat", mgr.new_func_symbol("getfloat", getfloat_t))?;
		let getarray_t = (
			FuncRetType::Int,
			vec![VarType {
				is_lval: true,
				type_t: value::BType::Int,
				dims: vec![0],
			}],
		);
		self.set_func("getarray", mgr.new_func_symbol("getarray", getarray_t))?;
		let getfrray_t = (
			FuncRetType::Int,
			vec![VarType {
				is_lval: true,
				type_t: value::BType::Float,
				dims: vec![0],
			}],
		);
		self.set_func("getfarray", mgr.new_func_symbol("getfarray", getfrray_t))?;
		let putint_t = (FuncRetType::Void, vec![VarType::new_int()]);
		self.set_func("putint", mgr.new_func_symbol("putint", putint_t))?;
		let putch_t = (FuncRetType::Void, vec![VarType::new_int()]);
		self.set_func("putch", mgr.new_func_symbol("putch", putch_t))?;
		let putarray_t = (
			FuncRetType::Void,
			vec![
				VarType::new_int(),
				VarType {
					is_lval: true,
					type_t: value::BType::Int,
					dims: vec![0],
				},
			],
		);
		self.set_func("putarray", mgr.new_func_symbol("putarray", putarray_t))?;
		let putfloat_t = (FuncRetType::Void, vec![VarType::new_float()]);
		self.set_func("putfloat", mgr.new_func_symbol("putfloat", putfloat_t))?;
		let putfarray_t = (
			FuncRetType::Void,
			vec![
				VarType::new_int(),
				VarType {
					is_lval: true,
					type_t: value::BType::Float,
					dims: vec![0],
				},
			],
		);
		self
			.set_func("putfarray", mgr.new_func_symbol("putfarray", putfarray_t))?;
		let putf_t = (
			FuncRetType::Void,
			vec![VarType {
				is_lval: true,
				type_t: value::BType::Int,
				dims: vec![0],
			}],
		);
		self.set_func("putf", mgr.new_func_symbol("putf", putf_t))?;
		let void_t = (FuncRetType::Void, vec![]);
		self.set_func(
			"before_main",
			mgr.new_func_symbol("before_main", void_t.clone()),
		)?;
		self.set_func(
			"after_main",
			mgr.new_func_symbol("after_main", void_t.clone()),
		)?;
		self.set_func(
			"starttime",
			mgr.new_func_symbol("starttime", void_t.clone()),
		)?;
		self.set_func("stoptime", mgr.new_func_symbol("stoptime", void_t))?;
		self.set_func(
			"_sysy_starttime",
			mgr.new_func_symbol(
				"_sysy_starttime",
				(FuncRetType::Void, vec![VarType::new_int()]),
			),
		)?;
		self.set_func(
			"_sysy_stoptime",
			mgr.new_func_symbol(
				"_sysy_stoptime",
				(FuncRetType::Void, vec![VarType::new_int()]),
			),
		)?;
		Ok(())
	}
}
