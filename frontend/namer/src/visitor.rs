use ast::{shirink, tree::*, Visitor};
use attr::Attrs;
use rrvm_symbol::manager::SymbolManager;
use scope::stack::ScopeStack;
use utils::{
	errors::Result,
	SysycError::{SemanticError, TypeError},
};
use value::{
	calc::{exec_binaryop, exec_unaryop},
	BType, BinaryOp, FuncType, Value, VarType,
};

#[derive(Default)]
pub struct Namer {
	mgr: SymbolManager,
	ctx: ScopeStack,
	cur_type: Option<(bool, BType)>,
}

impl Namer {
	pub fn transform(&mut self, program: &mut Program) -> Result<()> {
		program.accept(self)
	}
}

impl Namer {
	fn visit_dim_list(&mut self, node_list: &mut NodeList) -> Result<Vec<usize>> {
		let mut dim_list: Vec<usize> = Vec::new();
		for dim in node_list.iter_mut() {
			dim.accept(self)?;
			let value: Value = dim
				.get_attr("value")
				.ok_or(TypeError(
					"The length of array must be constant integer".to_string(),
				))?
				.into();
			shirink(dim);
			dim_list.push(value.to_int()? as usize);
		}
		Ok(dim_list)
	}
}

impl Visitor for Namer {
	fn visit_program(&mut self, node: &mut Program) -> Result<()> {
		self.ctx.push();
		self.ctx.extern_init(&mut self.mgr);
		self.is_global = true;
		self.init_values.clear();
		for v in node.global_vars.iter_mut() {
			v.accept(self)?;
		}
		self.is_global = false;

		for v in node.functions.iter_mut() {
			v.accept(self)?;
		}
		self.ctx.pop()?;
		Ok(())
	}
	fn visit_func_decl(&mut self, node: &mut FuncDecl) -> Result<()> {
		self.ctx.push();
		let mut func_type = Vec::new();
		for param in node.formal_params.iter_mut() {
			param.accept(self)?;
			func_type.push(param.get_attr("type").unwrap().into());
		}
		let func_type: FuncType = (node.ret_type, func_type);
		let symbol = self.mgr.new_func_symbol(&node.ident, func_type);
		self.ctx.set_func(&node.ident, symbol)?;
		node.block.accept(self)?;
		self.ctx.pop()?;
		Ok(())
	}
	fn visit_var_def(&mut self, node: &mut VarDef) -> Result<()> {
		let dim_list = self.visit_dim_list(&mut node.dim_list)?;
		let (is_const, btype) = self.cur_type.unwrap();
		let var_type: VarType = (!is_const, btype, dim_list).into();
		let symbol = self.mgr.new_var_symbol(&node.ident, var_type);
		node.set_attr("symbol", symbol.clone().into());
		self.ctx.set_val(&node.ident, symbol)?;
		if let Some(init) = node.init.as_mut() {
			init.accept(self)?;
		} else if is_const {
			return Err(SemanticError(format!(
				"uninitialized 'const {}'",
				node.ident
			)))?;
		}
		Ok(())
	}
	fn visit_var_decl(&mut self, node: &mut VarDecl) -> Result<()> {
		self.cur_type = Some((node.is_const, node.type_t));
		for var_def in node.defs.iter_mut() {
			var_def.accept(self)?;
		}
		self.cur_type = None;
		Ok(())
	}
	//TODO: solve init value list
	fn visit_init_val_list(&mut self, node: &mut InitValList) -> Result<()> {
		for val in node.val_list.iter_mut() {
			val.accept(self)?;
		}
		Ok(())
	}
	fn visit_literal_int(&mut self, node: &mut LiteralInt) -> Result<()> {
		let value: Value = node.value.into();
		node.set_attr("value", value.into());
		Ok(())
	}
	fn visit_literal_float(&mut self, node: &mut LiteralFloat) -> Result<()> {
		let value: Value = node.value.into();
		node.set_attr("value", value.into());
		Ok(())
	}
	fn visit_binary_expr(&mut self, node: &mut BinaryExpr) -> Result<()> {
		node.lhs.accept(self)?;
		node.rhs.accept(self)?;
		shirink(&mut node.lhs);
		shirink(&mut node.rhs);
		if node.op != BinaryOp::Assign {
			let lhs = node.lhs.get_attr("value");
			let rhs = node.rhs.get_attr("value");
			if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
				let value = exec_binaryop(&lhs.into(), node.op, &rhs.into())?;
				node.set_attr("value", value.into());
			}
		} else if let Some(symbol) = node.lhs.get_attr("symbol") {
			node.set_attr("symbol", symbol.clone());
		}
		Ok(())
	}
	fn visit_unary_expr(&mut self, node: &mut UnaryExpr) -> Result<()> {
		node.rhs.accept(self)?;
		shirink(&mut node.rhs);
		if let Some(rhs) = node.rhs.get_attr("value") {
			let value = exec_unaryop(node.op, &rhs.into())?;
			node.set_attr("value", value.into());
		}
		Ok(())
	}
	fn visit_func_call(&mut self, node: &mut FuncCall) -> Result<()> {
		let symbol = self.ctx.find_func(&node.ident)?.clone();
		let v: Option<VarType> = symbol.var_type.0.into();
		if let Some(v) = v {
			node.set_attr("type", v.into());
		}
		node.set_attr("func_symbol", symbol.into());
		for param in node.params.iter_mut() {
			param.accept(self)?;
			shirink(param);
		}
		Ok(())
	}
	fn visit_formal_param(&mut self, node: &mut FormalParam) -> Result<()> {
		let dim_list = self.visit_dim_list(&mut node.dim_list)?;
		let var_type: VarType = (false, node.type_t, dim_list).into();
		let symbol = self.mgr.new_var_symbol(&node.ident, var_type.clone());
		node.set_attr("symbol", symbol.clone().into());
		self.ctx.set_val(&node.ident, symbol)?;
		node.set_attr("type", var_type.into());
		Ok(())
	}
	fn visit_variable(&mut self, node: &mut Variable) -> Result<()> {
		let symbol = self.ctx.find_val(&node.ident)?.clone();
		node.set_attr("symbol", symbol.clone().into());
		node.set_attr("type", symbol.var_type.into());
		Ok(())
	}
	fn visit_block(&mut self, node: &mut Block) -> Result<()> {
		self.ctx.push();
		for stmt in node.stmts.iter_mut() {
			stmt.accept(self)?;
		}
		self.ctx.pop()?;
		Ok(())
	}
	fn visit_if(&mut self, node: &mut If) -> Result<()> {
		node.cond.accept(self)?;
		shirink(&mut node.cond);
		node.body.accept(self)?;
		if let Some(then) = &mut node.then {
			then.accept(self)?;
		}
		Ok(())
	}
	fn visit_while(&mut self, node: &mut While) -> Result<()> {
		node.cond.accept(self)?;
		shirink(&mut node.cond);
		node.body.accept(self)
	}
	fn visit_continue(&mut self, _node: &mut Continue) -> Result<()> {
		Ok(())
	}
	fn visit_break(&mut self, _node: &mut Break) -> Result<()> {
		Ok(())
	}
	fn visit_return(&mut self, node: &mut Return) -> Result<()> {
		if let Some(val) = &mut node.value {
			val.accept(self)?;
			shirink(val);
		}
		Ok(())
	}
}
