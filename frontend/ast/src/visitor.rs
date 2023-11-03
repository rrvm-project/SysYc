use scope::Scope;

use crate::tree::*;

pub trait Visitor {
	fn visit_program(&self, program: &mut Program, ctx: &mut dyn Scope);
	fn visit_var_def(&self, val_decl: &mut VarDef, ctx: &mut dyn Scope);
	fn visit_var_decl(&self, val_decl: &mut VarDecl, ctx: &mut dyn Scope);
	fn visit_func_decl(&self, val_decl: &mut FuncDecl, ctx: &mut dyn Scope);
	fn visit_init_val_list(&self, val_decl: &mut InitValList, ctx: &mut dyn Scope);
}

// pub struct PrintVisitor {

// }

// impl Visitor for PrintVisitor {
//   fn visitProgram(&self, program: &mut Program, ctx: &mut dyn Scope) {
//     program.accept(self, ctx)
//   }
// }
