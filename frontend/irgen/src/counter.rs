use ast::*;
use attr::Attrs;
use rrvm_symbol::VarSymbol;
use utils::errors::Result;

pub struct Counter {
	pub symbols: Vec<i32>,
}

impl Counter {
	pub fn new() -> Self {
		Self {
			symbols: Vec::new(),
		}
	}
}

impl Visitor for Counter {
	fn visit_var_def(&mut self, node: &mut VarDef) -> Result<()> {
		if let Some(init) = node.init.as_mut() {
			init.accept(self)?;
		}
		Ok(())
	}
	fn visit_var_decl(&mut self, node: &mut VarDecl) -> Result<()> {
		for var_def in node.defs.iter_mut() {
			var_def.accept(self)?;
		}
		Ok(())
	}
	fn visit_init_val_list(&mut self, node: &mut InitValList) -> Result<()> {
		for val in node.val_list.iter_mut() {
			val.accept(self)?;
		}
		Ok(())
	}
	fn visit_variable(&mut self, node: &mut Variable) -> Result<()> {
		let symbol: VarSymbol = node.get_attr("symbol").unwrap().into();
		self.symbols.push(symbol.id);
		Ok(())
	}
	fn visit_literal_int(&mut self, _node: &mut LiteralInt) -> Result<()> {
		Ok(())
	}
	fn visit_literal_float(&mut self, _node: &mut LiteralFloat) -> Result<()> {
		Ok(())
	}
	fn visit_binary_expr(&mut self, node: &mut BinaryExpr) -> Result<()> {
		node.lhs.accept(self)?;
		node.rhs.accept(self)
	}
	fn visit_unary_expr(&mut self, node: &mut UnaryExpr) -> Result<()> {
		node.rhs.accept(self)
	}
	fn visit_func_call(&mut self, node: &mut FuncCall) -> Result<()> {
		for param in node.params.iter_mut() {
			param.accept(self)?;
		}
		Ok(())
	}
	fn visit_block(&mut self, node: &mut Block) -> Result<()> {
		for stmt in node.stmts.iter_mut().take_while(|v| !v.is_end()) {
			stmt.accept(self)?;
		}
		Ok(())
	}
	fn visit_if(&mut self, node: &mut If) -> Result<()> {
		node.cond.accept(self)
	}
	fn visit_while(&mut self, node: &mut While) -> Result<()> {
		node.cond.accept(self)?;
		node.body.accept(self)
	}
	fn visit_return(&mut self, node: &mut Return) -> Result<()> {
		if let Some(val) = &mut node.value {
			val.accept(self)?;
		}
		Ok(())
	}
}
