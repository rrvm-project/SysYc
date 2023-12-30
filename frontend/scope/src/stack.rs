use crate::scope::Scope;
use rrvm_symbol::{manager::SymbolManager, FuncSymbol, VarSymbol};
use utils::{
	errors::Result,
	SysycError::{FatalError, SyntaxError},
};
use value::{FuncRetType, FuncType, VarType};

#[derive(Default)]
pub struct ScopeStack {
	func_scope: Scope<FuncType>,
	scopes: Vec<Scope<VarType>>,
}

const UNDERFLOW_ERR_MSG: &str = "stack of scopes underFlow";

impl ScopeStack {
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
		if let Some(scope) = self.scopes.last_mut() {
			scope.new_symbol(ident, symbol)
		} else {
			Err(FatalError(UNDERFLOW_ERR_MSG.to_owned()))
		}
	}
	pub fn set_func(&mut self, ident: &str, symbol: FuncSymbol) -> Result<()> {
		self.func_scope.new_symbol(ident, symbol)
	}
	pub fn find_val(&self, ident: &str) -> Result<&VarSymbol> {
		self
			.scopes
			.iter()
			.rev()
			.find_map(|v| v.get_symbol(ident))
			.ok_or(SyntaxError(format!("{} is not found", ident)))
	}
	pub fn find_func(&self, ident: &str) -> Result<&FuncSymbol> {
		self
			.func_scope
			.get_symbol(ident)
			.ok_or(SyntaxError(format!("{} is not found", ident)))
	}
	pub fn extern_init(&mut self, mgr: &mut SymbolManager) {
		let intvoid: FuncType = (FuncRetType::Int, Vec::new());
		let _ =
			self.set_func("getint", mgr.new_func_symbol("getint", intvoid.clone()));
		let _ = self.set_func("getch", mgr.new_func_symbol("getch", intvoid));
		let floatvoid: FuncType = (FuncRetType::Float, Vec::new());
		let _ =
			self.set_func("getfloat", mgr.new_func_symbol("getfloat", floatvoid));
		let getarray: FuncType = (
			FuncRetType::Int,
			vec![VarType {
				is_lval: true,
				type_t: value::BType::Int,
				dims: vec![0],
			}],
		);
		let _ =
			self.set_func("getarray", mgr.new_func_symbol("getarray", getarray));
		let getfrray: FuncType = (
			FuncRetType::Float,
			vec![VarType {
				is_lval: true,
				type_t: value::BType::Float,
				dims: vec![0],
			}],
		);
		let _ =
			self.set_func("getfarray", mgr.new_func_symbol("getfarray", getfrray));
		let putint: FuncType = (FuncRetType::Void, vec![VarType::new_int()]);
		let _ =
			self.set_func("putint", mgr.new_func_symbol("putint", putint.clone()));
		let _ =
			self.set_func("putch", mgr.new_func_symbol("putch", putint.clone()));
		let putarray: FuncType = (
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
		let _ =
			self.set_func("putarray", mgr.new_func_symbol("putarray", putarray));
		let putfloat = (FuncRetType::Void, vec![VarType::new_float()]);
		let _ =
			self.set_func("putfloat", mgr.new_func_symbol("putfloat", putfloat));
		let putfarray: FuncType = (
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
		let _ =
			self.set_func("putfarray", mgr.new_func_symbol("putfarray", putfarray));
		let putf: FuncType = (
			FuncRetType::Void,
			vec![VarType {
				is_lval: true,
				type_t: value::BType::Int,
				dims: vec![0],
			}],
		);
		let _ = self.set_func("putf", mgr.new_func_symbol("putf", putf));
		let void: FuncType = (FuncRetType::Void, vec![]);
		let _ = self.set_func(
			"before_main",
			mgr.new_func_symbol("before_main", void.clone()),
		);
		let _ = self.set_func(
			"after_main",
			mgr.new_func_symbol("after_main", void.clone()),
		);
		let _ = self
			.set_func("starttime", mgr.new_func_symbol("starttime", void.clone()));
		let _ = self.set_func("stoptime", mgr.new_func_symbol("stoptime", void));
		let _ = self.set_func(
			"_sysy_starttime",
			mgr.new_func_symbol("_sysy_starttime", putint.clone()),
		);
		let _ = self.set_func(
			"_sysy_stoptime",
			mgr.new_func_symbol("_sysy_stoptime", putint.clone()),
		);
	}
}
