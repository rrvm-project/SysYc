use crate::scope::Scope;
use rrvm_symbol::{FuncSymbol, VarSymbol};
use utils::{
	errors::Result,
	SysycError::{FatalError, SyntaxError},
};
use value::{FuncType, VarType};

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
}
