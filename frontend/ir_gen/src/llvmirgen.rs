use llvm::{LlvmProgram, llvmfuncemitter::LlvmFuncEmitter};
use utils::SysycError;
use ast::{tree::Program, visitor::Visitor};
pub struct LlvmIrGen {
    pub funcemitter: LlvmFuncEmitter,
}

impl LlvmIrGen {
    fn transform(&self, program: &Program) -> Result<LlvmProgram, SysycError>{
        program.comp_units.iter().for_each(|comp_unit| {
            comp_unit.accept(self);
        });
        Ok(LlvmProgram{
            funcs: vec![],
            // funcs: vec![self.funcemitter.emit_func()],
            global_vars: vec![],
        
        })
    }
}

impl Visitor for LlvmIrGen {
    
}