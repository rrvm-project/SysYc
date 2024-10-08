use ast::{tree::*, Visitor};
use attr::Attrs;
use rrvm_symbol::FuncSymbol;
use utils::{errors::Result, SysycError::TypeError};
use value::{
	calc_type::{to_rval, type_binaryop},
	BType, UnaryOp, VarType,
};

use crate::FUNC_IGNORE_TYPE_CHECK;

#[derive(Default)]
pub struct Typer {}

impl Typer {
	pub fn transform(&mut self, program: &mut Program) -> Result<()> {
		program.accept(self)
	}
}

impl Visitor for Typer {
	fn visit_program(&mut self, node: &mut Program) -> Result<()> {
		for v in node.functions.iter_mut() {
			v.accept(self)?;
		}
		Ok(())
	}

	fn visit_func_decl(&mut self, node: &mut FuncDecl) -> Result<()> {
		node.block.accept(self)
	}

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

	fn visit_literal_int(&mut self, node: &mut LiteralInt) -> Result<()> {
		node.set_attr("type", VarType::new_int().into());
		Ok(())
	}

	fn visit_literal_float(&mut self, node: &mut LiteralFloat) -> Result<()> {
		node.set_attr("type", VarType::new_float().into());
		Ok(())
	}

	fn visit_binary_expr(&mut self, node: &mut BinaryExpr) -> Result<()> {
		node.lhs.accept(self)?;
		node.rhs.accept(self)?;
		let lhs = node.lhs.get_attr("type").ok_or(TypeError(
			" void value not ignored as it ought to be".to_string(),
		))?;
		let rhs = node.rhs.get_attr("type").ok_or(TypeError(
			" void value not ignored as it ought to be".to_string(),
		))?;
		let type_t = type_binaryop(&lhs.into(), node.op, &rhs.into())?;
		node.set_attr("type", type_t.into());
		Ok(())
	}

	fn visit_unary_expr(&mut self, node: &mut UnaryExpr) -> Result<()> {
		node.rhs.accept(self)?;
		let rhs = node.rhs.get_attr("type").ok_or(TypeError(
			" void value not ignored as it ought to be".to_string(),
		))?;
		let type_t = to_rval(&rhs.into());
		if type_t.type_t == BType::Float && node.op == UnaryOp::Not {
			return Err(TypeError("Only integer can use operator '!'".to_string()));
		}
		node.set_attr("type", type_t.into());
		Ok(())
	}

	fn visit_func_call(&mut self, node: &mut FuncCall) -> Result<()> {
		for param in node.params.iter_mut() {
			param.accept(self)?;
		}

		let symbol: FuncSymbol = node.get_attr("func_symbol").unwrap().into();
		let (_, params) = symbol.var_type;

		if node.params.len() != params.len() {
			return Err(TypeError(format!(
				"unmatch numbers of params for function {}",
				node.ident
			)));
		}

		// XXX: testcase derich.sy pass pass incorrect type of arguments to the function getfarray.
		if !FUNC_IGNORE_TYPE_CHECK.contains(&node.ident.as_str()) {
			for (x_t, y) in params.iter().zip(node.params.iter()) {
				let y_t: VarType = y.get_attr("type").unwrap().into();
				let err_msg =
					format!("expected `{}` but argument is of type `{}`", x_t, y_t);
				if x_t.dims.len() != y_t.dims.iter().len()
					|| x_t.dims.iter().skip(1).ne(y_t.dims.iter().skip(1))
				{
					return Err(TypeError(err_msg));
				}
			}
		}

		Ok(())
	}

	fn visit_variable(&mut self, _node: &mut Variable) -> Result<()> {
		Ok(())
	}

	fn visit_block(&mut self, node: &mut Block) -> Result<()> {
		for stmt in node.stmts.iter_mut() {
			stmt.accept(self)?;
		}
		Ok(())
	}

	fn visit_if(&mut self, node: &mut If) -> Result<()> {
		node.cond.accept(self)?;
		node.body.accept(self)?;
		if let Some(then) = &mut node.then {
			then.accept(self)?;
		}
		Ok(())
	}

	fn visit_while(&mut self, node: &mut While) -> Result<()> {
		node.cond.accept(self)?;
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
			let val_t: VarType = val.get_attr("type").unwrap().into();
			if val_t.is_array() {
				return Err(TypeError("function cannot return an array".to_string()));
			}
		}
		Ok(())
	}
}
