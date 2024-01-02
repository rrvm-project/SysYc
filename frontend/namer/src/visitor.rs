use crate::utils::*;
use ast::{shirink, tree::*, Visitor};
use attr::Attrs;
use rrvm_symbol::manager::SymbolManager;
use scope::stack::ScopeStack;
use utils::errors::Result;
use value::{
	calc::{exec_binaryop, exec_unaryop},
	BType, BinaryOp, FuncType, Value, VarType,
};

#[derive(Default)]
pub struct Namer {
	mgr: SymbolManager,
	ctx: ScopeStack,
	decl_type: Option<(bool, BType)>,
	decl_dims: Vec<usize>,
	depth: usize,
}

impl Namer {
	pub fn transform(&mut self, program: &mut Program) -> Result<()> {
		program.accept(self)
	}
	fn cur_size(&self) -> usize {
		self.decl_dims.iter().skip(self.depth).product()
	}
	fn cur_dims(&self) -> Vec<usize> {
		self.decl_dims.iter().skip(self.depth).cloned().collect()
	}
}

impl Namer {
	fn visit_dim_list(&mut self, node_list: &mut NodeList) -> Result<Vec<usize>> {
		let mut dim_list: Vec<usize> = Vec::new();
		for dim in node_list.iter_mut() {
			dim.accept(self)?;
			let value: Value =
				dim.get_attr("value").ok_or_else(array_dims_error)?.into();
			shirink(dim);
			if let Value::Int(v) = value {
				if v <= 0 {
					return Err(non_positive_dim_length());
				}
				dim_list.push(v as usize);
			} else {
				return Err(array_dims_error());
			}
		}
		Ok(dim_list)
	}
}

impl Visitor for Namer {
	fn visit_program(&mut self, node: &mut Program) -> Result<()> {
		self.ctx.push();
		for v in node.global_vars.iter_mut() {
			v.accept(self)?;
		}
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
		let (is_const, btype) = self.decl_type.unwrap();
		let var_type: VarType = (!is_const, btype, &dim_list).into();
		let symbol = self.mgr.new_var_symbol(&node.ident, var_type);
		node.set_attr("symbol", symbol.clone().into());
		self.ctx.set_val(&node.ident, symbol.clone())?;
		if let Some(init) = node.init.as_mut() {
			self.decl_dims = dim_list.clone();
			init.accept(self)?;
			shirink(init);
		}
		if is_const {
			let value = node
				.init
				.as_ref()
				.ok_or_else(|| uninitialized(&node.ident))?
				.get_attr("value")
				.ok_or_else(|| initialize_by_none(&node.ident))?;
			self.ctx.set_constant(symbol.id, value.into())?;
			node.init = None;
		}
		Ok(())
	}

	fn visit_var_decl(&mut self, node: &mut VarDecl) -> Result<()> {
		self.decl_type = Some((node.is_const, node.type_t));
		for var_def in node.defs.iter_mut() {
			var_def.accept(self)?;
		}
		self.decl_type = None;
		Ok(())
	}

	fn visit_init_val_list(&mut self, node: &mut InitValList) -> Result<()> {
		self.depth += 1;
		for val in node.val_list.iter_mut() {
			val.accept(self)?;
			shirink(val);
		}
		let len = self.cur_size();
		self.depth -= 1;

		let (is_const, btype) = self.decl_type.unwrap();
		if is_const || self.ctx.is_global() {
			let mut array: Vec<Value> = Vec::new();
			for val in node.val_list.iter_mut() {
				let value: Value = val
					.get_attr("value")
					.ok_or_else(|| initialize_by_none("array"))?
					.into();
				if let Value::Array((_, val_array)) = value {
					let size = (len - array.len() % len) % len;
					array.extend((0..size).map(|_| btype.to_value()));
					array.extend(val_array);
				} else {
					array.push(value.to_type(btype)?);
				};
			}
			let size = self.cur_size() - array.len();
			array.extend((0..size).map(|_| btype.to_value()));
			let value: Value = (self.cur_dims(), array).into();
			node.set_attr("value", value.into());
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
		let symbol = self.ctx.get_func(&node.ident)?.clone();
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
		let var_type: VarType = (true, node.type_t, &dim_list).into();
		let symbol = self.mgr.new_var_symbol(&node.ident, var_type.clone());
		node.set_attr("symbol", symbol.clone().into());
		self.ctx.set_val(&node.ident, symbol)?;
		node.set_attr("type", var_type.into());
		Ok(())
	}

	fn visit_variable(&mut self, node: &mut Variable) -> Result<()> {
		let symbol = self.ctx.get_val(&node.ident)?.clone();
		node.set_attr("symbol", symbol.clone().into());
		node.set_attr("type", symbol.var_type.into());
		if let Some(value) = self.ctx.get_constant(symbol.id) {
			node.set_attr("value", value.clone().into())
		}
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
