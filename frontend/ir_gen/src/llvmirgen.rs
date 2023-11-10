use llvm::{LlvmProgram, llvmfuncemitter::LlvmFuncEmitter, func::LlvmFunc};
use attr::{Attrs, Attr};
use namer::utils::DataFromNamer;
use namer::namer::SYMBOL_NUMBER;
use ast::{tree::*, visitor::Visitor};
use utils::SysycError;
use llvm::llvmvar::VarType;

static VALUE: &str = "value";
pub struct LlvmIrGen {
    pub funcemitter: LlvmFuncEmitter,
    pub funcs: Vec<LlvmFunc>,
    pub data: DataFromNamer,
}

impl LlvmIrGen {
    fn transform(&self, program: Program) -> Result<LlvmProgram, SysycError>{
        program.comp_units.iter().for_each(|comp_unit| {
            comp_unit.accept(self);
        });
        Ok(LlvmProgram{
            funcs: self.funcs,
            // funcs: vec![self.funcemitter.emit_func()],
            global_vars: vec![],
        
        })
    }
}

impl Visitor for LlvmIrGen {
    fn visit_program(&mut self, program: &mut Program) -> Result<(), SysycError> {
        // TODO: 这个 for 循环如果改成迭代器访问的话，不知道如何传出错误
        for comp_unit in &mut program.comp_units {
            comp_unit.accept(self)?;
        };
        Ok(())
    }
    fn visit_func_decl(&mut self, val_decl: &mut FuncDecl) -> Result<(), SysycError> {
        let ret_type = match val_decl.func_type {
            ast::FuncType::Int => llvm::llvmvar::VarType::I32,
            ast::FuncType::Float => llvm::llvmvar::VarType::F32,
            ast::FuncType::Void => llvm::llvmvar::VarType::Void,
        };
        self.funcemitter = LlvmFuncEmitter::new(val_decl.ident.clone(), ret_type, vec![]);
        for param in &mut val_decl.formal_params {
            param.accept(self)?;
        };
        val_decl.block.accept(self)?;
        Ok(())
    }
    fn visit_formal_param(&mut self, val_decl: &mut FormalParam) -> Result<(), SysycError> {
        let var_type = match val_decl.type_t {
            ast::VarType::Int => if val_decl.dim_list.is_none() {llvm::llvmvar::VarType::I32} else {llvm::llvmvar::VarType::I32Ptr},
            ast::VarType::Float => if val_decl.dim_list.is_none() {llvm::llvmvar::VarType::F32} else {llvm::llvmvar::VarType::F32Ptr},
            _ => unreachable!(),
        };
        self.funcemitter.visit_formal_param(var_type);
        Ok(())
    }
    fn visit_block(&mut self,	val_decl: &mut Block) -> Result<(), SysycError> {
        for stmt in &mut val_decl.stmts {
            stmt.accept(self)?;
        };
        Ok(())
    }
    fn visit_func_call(&mut self,	val_decl: &mut FuncCall) -> Result<(), SysycError> {
        let mut params = vec![];
        for param in &mut val_decl.params {
            param.accept(self)?;
            if let Some(Attr::Value(v)) = param.get_attr(VALUE) {
                params.push(v.clone());
            } else {
                return Err(SysycError::LlvmSyntexError(format!("param of call {} has no value", val_decl.ident.clone())));
            }
        };
        let funcsymbol_id = match val_decl.get_attr(SYMBOL_NUMBER) {
            Some(Attr::FuncSymbol(id)) => *id,
            _ => return Err(SysycError::LlvmSyntexError(format!("call {} has no funcsymbol", val_decl.ident.clone()))),
        };
        let funcsymbol = &self.data.func_symbols[funcsymbol_id];
        let var_type = match funcsymbol.ret_t.base_type {
            ir_type::builtin_type::BaseType::Int => if funcsymbol.ret_t.dims.len() == 0 {VarType::I32} else {VarType::I32Ptr},
            ir_type::builtin_type::BaseType::Float => if funcsymbol.ret_t.dims.len() == 0 {VarType::F32} else {VarType::F32Ptr},
            ir_type::builtin_type::BaseType::Void => VarType::Void,
        };
        let target = self.funcemitter.visit_call_instr(var_type, val_decl.ident.clone(), params);
        val_decl.set_attr(VALUE, Attr::Value(llvm::llvmop::Value::Temp(target)));
        Ok(())
    }
    fn visit_binary_expr(&mut self,	val_decl: &mut BinaryExpr) -> Result<(), SysycError> {
        val_decl.lhs.accept(self)?;
        val_decl.rhs.accept(self)?;
        let lhs = match val_decl.lhs.get_attr(VALUE) {
            Some(Attr::Value(v)) => v.clone(),
            _ => return Err(SysycError::LlvmSyntexError(format!("lhs of binary expr has no value"))),
        };
        let rhs = match val_decl.rhs.get_attr(VALUE) {
            Some(Attr::Value(v)) => v.clone(),
            _ => return Err(SysycError::LlvmSyntexError(format!("rhs of binary expr has no value"))),
        };
        let op = match val_decl.op {
            ast::BinaryOp::Add => llvm::llvmop::ArithOp::Add,
            ast::BinaryOp::Sub => llvm::llvmop::ArithOp::Sub,
            ast::BinaryOp::Mul => llvm::llvmop::ArithOp::Mul,
        }

    }
}