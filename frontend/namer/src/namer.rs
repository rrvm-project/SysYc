use ast::{tree::*, visitor::Visitor, FuncType};
use utils::SysycError;
use scope::{scope::ScopeStack, symbol::FuncSymbol};

#[derive(Debug)]
pub struct Namer{// namer 需要的context在这里存！
	pub loop_num: i32,
}

impl Default for Namer {
	fn default() -> Self {
		Namer {
			loop_num: 0,
		}
	}	
}

impl Namer {
	pub fn transform(&self, mut program : Program) -> Result<Program, SysycError>{
		let mut ctx = ScopeStack::new();
		// program.accept(self);
		// 给每个ident，对应根据可见性规则的VarSymbol或者FuncSymbol
		Ok(program)	
	}
}

impl Visitor for Namer {
	fn visit_program(&self, program: &mut Program) -> Result<(), SysycError>{
		// program.comp_units.iter_mut().for_each(|unit| unit.accept(self, ctx));
		// if let Some(f) = ctx.lookup_func("main") {
		// 	if f.ret_t != FuncType::Int {
		// 		return Err(SysycError::NamerError("main function should return int".to_string()));
		// 	}
		// 	if f.param_num() != 0 {
		// 		return Err(SysycError::NamerError("main function should not have any parameter".to_string()));
		// 	}
		// } else {
		// 	return Err(SysycError::NamerError("main function not found".to_string()));
		// }
		Ok(())
	}
	fn visit_var_def(&self, val_decl: &mut VarDef) -> Result<(), SysycError>{
		val_decl.init.as_mut().map(|init| init.accept(self));
		Ok(())
	}
	fn visit_var_decl(&self, val_decl: &mut VarDecl) -> Result<(), SysycError>{
		todo!()
	}
	fn visit_func_decl(&self, val_decl: &mut FuncDecl) -> Result<(), SysycError>{
		// if ctx.current_scope().lookup_func(val_decl.ident.as_str()).is_some() || ctx.current_scope().lookup_var(val_decl.ident.as_str()).is_some(){
		// 	return Err(SysycError::NamerError(format!("Identifier {} already declared", val_decl.ident)));
		// }
		// let symbol = FuncSymbol {
		// 	name: val_decl.ident.clone(),
		// 	ret_t: val_decl.func_type.clone(),
		// 	params: vec![],
		// };
		// ctx.push();
		// val_decl.formal_params.iter_mut().for_each(|param| {
		// 	param.accept(self, ctx); 
		// 	// symbol.add_param(param.get_attr("symbol"));
		// });		
		// val_decl.block.accept(self, ctx);
		// ctx.pop();
		Ok(())
	}
	fn visit_init_val_list(&self,	val_decl: &mut InitValList) -> Result<(), SysycError>{
		todo!()
	}
	fn visit_literal_int(&self,	val_decl: &mut LiteralInt) -> Result<(), SysycError>{
		todo!()
	}
	fn visit_literal_float(&self,	val_decl: &mut LiteralFloat) -> Result<(), SysycError>{
		todo!()
	}
	fn visit_binary_expr(&self,	val_decl: &mut BinaryExpr) -> Result<(), SysycError>{
		todo!()
	}
	fn visit_unary_expr(&self, val_decl: &mut UnaryExpr) -> Result<(), SysycError>{
		todo!()
	}
	fn visit_func_call(&self,	val_decl: &mut FuncCall) -> Result<(), SysycError>{
		todo!()
	}
	fn visit_formal_param(&self,	val_decl: &mut FormalParam) -> Result<(), SysycError>{
		todo!()
	}
	fn visit_lval(&self, val_decl: &mut Lval) -> Result<(), SysycError>{
		todo!()
	}
	fn visit_block(&self,	val_decl: &mut Block) -> Result<(), SysycError>{
		todo!()
	}
	fn visit_if(&self, val_decl: &mut If) -> Result<(), SysycError>{
		todo!()
	}
	fn visit_while(&self,	val_decl: &mut While) -> Result<(), SysycError>{
		todo!()
	}
	fn visit_continue(&self, val_decl: &mut Continue) -> Result<(), SysycError>{
		todo!()
	}
	fn visit_break(&self,	val_decl: &mut Break) -> Result<(), SysycError>{
		todo!()
	}
	fn visit_return(&self, val_decl: &mut Return) -> Result<(), SysycError>{
		todo!()
	}
}