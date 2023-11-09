use std::collections::HashMap;

use crate::symbol::{FuncSymbol, VarSymbol};

// pub enum ScopeKind{
//     Global,
//     Local,
// }

#[derive(Debug)]
pub struct Scope {
	// TODO: scope需要专门有个 kind 域来区分全局作用域和局部作用域吗
	// pub kind: ScopeKind,
	pub varsymbols: HashMap<String, VarSymbol>,
	pub funcsymbols: HashMap<String, FuncSymbol>,
}

impl Default for Scope {
	fn default() -> Self {
		Self::new()
	}
}

impl Scope {
	pub fn new() -> Self {
		Scope {
			// kind,
			varsymbols: HashMap::new(),
			funcsymbols: HashMap::new(),
		}
	}
	pub fn declare_var(&mut self, var: &VarSymbol) {
		self.varsymbols.insert(var.name.clone(), var.clone());
	}
	pub fn declare_func(&mut self, func: &FuncSymbol) {
		self.funcsymbols.insert(func.name.clone(), func.clone());
	}
	pub fn lookup_var(&self, name: &str) -> Option<&VarSymbol> {
		self.varsymbols.get(name)
	}
	pub fn lookup_func(&self, name: &str) -> Option<&FuncSymbol> {
		self.funcsymbols.get(name)
	}
}
#[derive(Debug)]
pub struct ScopeStack {
	pub scopes: Vec<Scope>,
}

impl Default for ScopeStack {
	fn default() -> Self {
		Self::new()
	}
}

impl ScopeStack {
	pub fn current_is_global(&self) -> bool {
		self.scopes.len() == 1
	}

	pub fn new() -> Self {
		ScopeStack {
			scopes: vec![Scope::new()],
		}
	}
	pub fn push(&mut self) {
		self.scopes.push(Scope::new());
	}
	pub fn pop(&mut self) {
		if !self.is_empty() {
			self.scopes.pop();
		} else {
			unreachable!("scope stack should not be empty")
		}
	}
	pub fn current_scope(&self) -> &Scope {
		if self.is_empty() {
			unreachable!("scope stack should not be empty")
		} else {
			self.scopes.last().unwrap()
		}
	}
	pub fn current_scope_mut(&mut self) -> &mut Scope {
		if self.is_empty() {
			unreachable!("scope stack should not be empty")
		} else {
			self.scopes.last_mut().unwrap()
		}
	}
	// it should always return false
	pub fn is_empty(&self) -> bool {
		self.scopes.is_empty()
	}
	pub fn declare_var(&mut self, var: &VarSymbol) {
		self.current_scope_mut().declare_var(var);
	}
	pub fn declare_func(&mut self, func: &FuncSymbol) {
		self.current_scope_mut().declare_func(func);
	}
	pub fn declare_func_at_global(&mut self, func: &FuncSymbol) {
		self.scopes[0].declare_func(func);
	}
	pub fn lookup_var(&self, name: &str) -> Option<&VarSymbol> {
		for scope in self.scopes.iter().rev() {
			if let Some(var) = scope.lookup_var(name) {
				return Some(var);
			}
		}
		None
	}
	pub fn lookup_func(&self, name: &str) -> Option<&FuncSymbol> {
		for scope in self.scopes.iter().rev() {
			if let Some(func) = scope.lookup_func(name) {
				return Some(func);
			}
		}
		None
	}
}
